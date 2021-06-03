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
use fastjob_components_scheduler::Scheduler;
use fastjob_components_storage::model::{app_info::AppInfo, job_info::JobInfo, lock::Lock};
use fastjob_components_storage::{BatisError, Storage};
use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_components_utils::pair::PairCond;
use fastjob_components_utils::sched_pool::{JobHandle, SchedPool};
use fastjob_components_utils::time::duration_to_ms;
use fastjob_proto::fastjob::{
    WorkerManagerConfig, WorkerManagerScope, WorkerManagerScope::ServerSide,
};
use snafu::ResultExt;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::process::id;
use std::ops::Sub;

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
    storage: Arc<S>,
    // sender_t: SenderT,
    workers: DashMap<String, Worker>,
    scheduler: Scheduler<S>,
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
            .field(&self.storage)
            // .field(&self.sender_t)
            .field(&self.scheduler)
            .finish()
    }
}

pub struct WorkerManagerBuilder<S: Storage> {
    id: i64,
    status: AtomicCell<ComponentStatus>,
    config: WorkerManagerConfig,
    // sender: Sender<Vec<JobInfo>>,
    // pair: Arc<PairCond>,
}

impl<S: Storage> WorkerManagerBuilder<S> {
    pub fn builder(
        config: WorkerManagerConfig,
        // sender: Sender<Vec<JobInfo>>,
        // pair: Arc<PairCond>,
    ) -> Self {
        Self {
            id: 0,
            status: AtomicCell::new(ComponentStatus::Initialized),
            config,
            // sender,
            // pair,
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
            // job_fetcher: JobFetcher::new(self.id, self.sender.clone(), S, self.pair.clone()),
            storage: Arc::new(S),
            // sender_t: SenderT::new(
            //     DashMap::default(),
            // ),
            workers: DashMap::default(),
            scheduler: Scheduler::new(S),
        }
    }
}

impl<S: Storage> Component for WorkerManager<S> {
    fn start(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Ready);

        // Change status.
        self.status.store(ComponentStatus::Starting);

        // Start scheduler thread.
        self.sched_pool.schedule_at_fixed_rate(
            self.sched(),
            WORKER_MANAGER_FETCH_INIT_TIME,
            WORKER_MANAGER_FETCH_FIXED_TIME,
        );

        // self.job_fetcher.set_handler(handler);

        self.status.store(ComponentStatus::Running);
    }

    fn stop(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Running);
        self.status.store(ComponentStatus::Terminating);

        // self.scheduler.shutdown();

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
            let lock = Lock::new(String::from(app_id), 30000, String::from(current_server));
            if !self.lock(lock) {
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

    fn lock(&self, lock: Lock) -> bool {
        let r = self.storage.save(lock);
        if r.is_ok() {
            return true;
        }
        false
    }

    fn release(&self) {}

    /// Manually perform a schedule.
    pub fn manual_sched(&mut self) -> Result<()> {
        self.sched();
        Ok(())
    }

    fn sched(&mut self) {
        info!("Schedule task start.");
        let instant = Instant::now();
        let app_ids: Option<Vec<AppInfo>> = self.storage.find_all_by_current_server().context("")?;
        if app_ids.is_none() {
            info!("[JobScheduler] current server has no app's job to schedule.");
            return;
        }
        let ids: &Vec<&str> = app_ids
            .unwrap()
            .iter()
            .map(|app| app.id)
            .collect();

        self.clean_useless_worker(ids);

        self.scheduler.schedule_cron_job(ids)?;
        let cron_cost = instant.elapsed();
        self.scheduler.schedule_worker_flow(ids)?;
        let worker_flow_cost = instant.elapsed().sub(cron_cost);
        self.scheduler.schedule_frequent_job(ids)?;
        let frequent_cost = instant.elapsed().sub(worker_flow_cost + cron_cost);
    }


    fn clean_useless_worker(&mut self, app_ids: &Vec<&str>) {}

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
mod tests {
    use std::time::{Instant, Duration};
    use std::ops::Sub;

    #[test]
    fn t_elapsed() {
        let instant = Instant::now();
        std::thread::sleep(Duration::from_millis(500));
        let cost1 = instant.elapsed().as_millis();
        std::thread::sleep(Duration::from_millis(500));
        let cost2 = instant.elapsed().as_millis().sub(cost1);
        std::thread::sleep(Duration::from_millis(500));
        let cost3 = instant.elapsed().as_millis().sub(cost1 + cost2);

        println!("{},{},{}", cost1, cost2, cost3)
    }
}
