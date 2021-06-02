//! Worker Manager only stores tasks and is not going to scheduling.
//! If worker manager checks itself is free, it will start the steal thread
//! that steals the task from another server in FastJob Cluster. However,
//! if don't have enough space that will reject task request and respond a full error message.
//! so client will retry this request that send to another server util success unless achieved
//! the maximum retry numbers and send has failed.
use super::{error, Result};
use crate::job_fetcher::JobFetcher;
use crate::sender::Sender as SenderT;
use crate::{init_grpc_client, Worker};
use chrono::Local;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender};
use dashmap::DashMap;
use fastjob::services::health_checker::HealthChecker;
use fastjob_components_scheduler::Dispatcher;
use fastjob_components_storage::model::app_info::AppInfo;
use fastjob_components_storage::model::job_info::JobInfo;
use fastjob_components_storage::model::lock::Lock;
use fastjob_components_storage::model::task::Task;
use fastjob_components_storage::{BatisError, Storage};
use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_components_utils::pair::PairCond;
use fastjob_components_utils::sched_pool::{JobHandle, SchedPool};
use fastjob_components_utils::time::duration_to_ms;
use fastjob_components_utils::timing_wheel::TimingWheel;
use fastjob_proto::fastjob::{
    WorkerManagerConfig, WorkerManagerScope, WorkerManagerScope::ServerSide,
};
use snafu::ResultExt;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::{Duration, Instant};

const WORKER_MANAGER_SCHED_POOL_NUM_SIZE: usize = 2;
const WORKER_MANAGER_SCHED_POOL_NAME: &str = "worker-manager";
const WORKER_MANAGER_FETCH_INIT_TIME: Duration = Duration::from_secs(2);
const WORKER_MANAGER_FETCH_FIXED_TIME: Duration = Duration::from_secs(5);
const RETRY_TIMES: u32 = 3;

pub struct WorkerManager<S: Storage> {
    id: i64,
    address: String,
    status: AtomicCell<ComponentStatus>,
    sched_pool: SchedPool,
    job_fetcher: JobFetcher<S>,
    storage: S,
    // sender_t: SenderT,
    workers: DashMap<String, Worker>,
    dispatcher: Dispatcher,
}

impl<S: Storage> Clone for WorkerManager<S> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<S: Storage> Debug for WorkerManager<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.id)
            .field(&self.status)
            .field(&self.sched_pool)
            .field(&self.job_fetcher)
            .field(&self.storage)
            // .field(&self.sender_t)
            .field(&self.dispatcher)
            .finish()
    }
}

pub struct WorkerManagerBuilder {
    id: i64,
    status: AtomicCell<ComponentStatus>,
    config: WorkerManagerConfig,
    sender: Sender<Vec<JobInfo>>,
    pair: Arc<PairCond>,
}

impl<S: Storage> WorkerManagerBuilder {
    pub fn builder(
        config: WorkerManagerConfig,
        sender: Sender<Vec<JobInfo>>,
        pair: Arc<PairCond>,
    ) -> Self {
        Self {
            id: 0,
            status: AtomicCell::new(ComponentStatus::Initialized),
            config,
            sender,
            pair,
        }
    }

    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn build(self) -> WorkerManager<S> {
        WorkerManager {
            id: self.id,
            address: "".to_string(),
            status: AtomicCell::new(ComponentStatus::Ready),
            sched_pool: SchedPool::new(
                WORKER_MANAGER_SCHED_POOL_NUM_SIZE,
                WORKER_MANAGER_SCHED_POOL_NAME,
            ),
            job_fetcher: JobFetcher::new(self.id, self.sender.clone(), S, self.pair.clone()),
            storage: S,
            // sender_t: SenderT::new(
            //     DashMap::default(),
            // ),
            workers: DashMap::default(),
            dispatcher: Dispatcher::new(),
        }
    }
}

impl<S: Storage> Component for WorkerManager<S> {
    fn start(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Ready);

        // Change status.
        self.status.store(ComponentStatus::Starting);

        self.health_checker.run();

        self.dispatcher.dispatcher();

        // Start fetch job thread.
        let handler = self.sched_pool.schedule_at_fixed_rate(
            self.job_fetcher.fetch(),
            WORKER_MANAGER_FETCH_INIT_TIME,
            WORKER_MANAGER_FETCH_FIXED_TIME,
        );

        self.job_fetcher.set_handler(handler);

        self.status.store(ComponentStatus::Running);
    }

    fn stop(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Running);
        self.status.store(ComponentStatus::Terminating);

        self.job_fetcher.shutdown();

        self.dispatcher.shutdown();

        self.health_checker.shutdown();

        self.status.store(ComponentStatus::Shutdown);
    }
}

impl<S: Storage> WorkerManager<S> {
    /// Connect to worker grpc client.
    pub fn connect(&self, addr: &str) -> Result<()> {
        self.workers.insert(addr.into(), Worker::build(addr))?;
        Ok(())
    }

    /// Validate worker is effective when worker init.
    pub fn validate_worker(&self, app_name: &str) -> Result<()> {
        let wrapper = self.storage.get_wrapper().eq("app_name", app_name);
        let rs: std::result::Result<AppInfo, BatisError> = self.storage.fetch(&wrapper);

        if rs.is_ok() {
            return Ok(());
        }

        Err(error::WorkerNotRegistered {
            app_name_or_id: String::from(app_name),
        })
    }

    /// Select the appropriate server according to the appName sent by the worker
    /// And check it whether alive,if dead the current service tries to usurp the throne.
    ///
    /// Thread Safety: Distributed-Lock.
    pub fn lookup(&self, current_server: &str, app_id: u64) -> Result<&str> {
        let cache = &Vec::<String>::new();
        if self.address.eq(current_server) {
            return Ok(current_server);
        }
        let wrapper = &self.storage.get_wrapper().eq("id", app_id);
        for _ in 0..RETRY_TIMES {
            let rs: Option<AppInfo> = self
                .storage
                .fetch(wrapper)
                .context(error::WorkerStorageError)?;

            if rs.is_none() {
                return Err(error::WorkerNotRegistered {
                    app_name_or_id: app_id.to_string(),
                });
            }
            let name = rs.as_ref().unwrap().app_name.unwrap();
            let origin_server = rs.as_ref().unwrap().current_server.unwrap().as_str();
            if self.is_active(origin_server, cache) {
                return Ok(origin_server);
            }

            // Server is not available, try server election again, need to lock.
            let lock = &Lock::new(String::from(app_id), 30000, String::from(current_server));
            if !lock.lock() {
                std::thread::sleep(Duration::from_millis(500));
            }

            // It is possible that a machine has already completed the Server election and needs to be judged again.
            let mut rs: Option<AppInfo> = self
                .storage
                .fetch(wrapper)
                .context(error::WorkerStorageError)?;
            let cur = rs.as_ref().unwrap().current_server.unwrap().as_str();
            if self.is_active(cur, cache) {
                return Ok(cur);
            }
            // Usurpation, native as current server.
            rs.take().unwrap().current_server = Some(current_server.to_string());
            rs.take().unwrap().gmt_modified = Some(Local::now().timestamp_millis());
            self.storage.save(rs.unwrap());
            info!(
                "[Election] server {} become the new server fo appId {}",
                current_server.to_string(),
                app_id
            )
        }
        Err(error::LookupFail {
            server_ip: self.address.clone(),
        })
    }

    fn is_active(&self, target_server: &str, cache: &Vec<String>) -> bool {
        if cache.contains(&target_server.to_string()) {
            return false;
        }
        // send hello request to target server.
        let client = init_grpc_client(target_server);
        let mut req = Ping::default();
        let reply = client.ping(&req).expect("Ping failed");
        if reply.get_code() == 200 {
            return true;
        }
        false
    }

    /// Manually perform a schedule.
    pub fn manual_sched(&mut self) -> Result<()> {
        self.sched()
    }

    fn sched(&mut self) -> Result<()> {
        Ok(())
    }

    /// Determine if the worker is to be removed.
    #[inline]
    fn is_need_remove(&self) -> bool {
        true
    }

    /// Returns the status of `WorkerManager`.
    #[inline]
    pub fn get_status(&self) -> ComponentStatus {
        self.status.load()
    }
}

#[cfg(test)]
mod tests {}
