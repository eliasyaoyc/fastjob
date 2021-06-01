//! Worker Manager only stores tasks and is not going to scheduling.
//! If worker manager checks itself is free, it will start the steal thread
//! that steals the task from another server in FastJob Cluster. However,
//! if don't have enough space that will reject task request and respond a full error message.
//! so client will retry this request that send to another server util success unless achieved
//! the maximum retry numbers and send has failed.
use super::Result;
use crate::job_fetcher::JobFetcher;
use crate::{Worker, init_grpc_client};
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender};
use fastjob_components_storage::model::job_info::JobInfo;
use fastjob_components_storage::model::task::Task;
use fastjob_components_storage::Storage;
use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_components_utils::sched_pool::{JobHandle, SchedPool};
use fastjob_components_utils::timing_wheel::TimingWheel;
use fastjob_proto::fastjob::{
    WorkerManagerConfig, WorkerManagerScope, WorkerManagerScope::ServerSide,
};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use fastjob_components_utils::pair::PairCond;
use std::sync::Arc;
use dashmap::DashMap;
use crate::sender::Sender as SenderT;
use fastjob_components_scheduler::Dispatcher;
use crate::health_checker::HealthChecker;

const WORKER_MANAGER_SCHED_POOL_NUM_SIZE: usize = 2;
const WORKER_MANAGER_SCHED_POOL_NAME: &str = "worker-manager";
const WORKER_MANAGER_FETCH_INIT_TIME: Duration = Duration::from_secs(2);
const WORKER_MANAGER_FETCH_FIXED_TIME: Duration = Duration::from_secs(5);

pub struct WorkerManager<S: Storage> {
    id: i64,
    status: AtomicCell<ComponentStatus>,
    config: WorkerManagerConfig,
    scope: WorkerManagerScope,
    sched_pool: SchedPool,
    job_fetcher: JobFetcher<S>,
    storage: S,
    // sender_t: SenderT,
    workers: DashMap<String, ::grpcio::Client>,
    dispatcher: Dispatcher,
    health_checker: HealthChecker,
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
            .field(&self.config)
            .field(&self.scope)
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
    scope: WorkerManagerScope,
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
            scope: WorkerManagerScope::EMPTY,
            sender,
            pair,
        }
    }

    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn scope(mut self, scope: WorkerManagerScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn build(self) -> WorkerManager<S> {
        WorkerManager {
            id: self.id,
            status: AtomicCell::new(ComponentStatus::Ready),
            config: self.config,
            scope: self.scope,
            sched_pool: SchedPool::new(
                WORKER_MANAGER_SCHED_POOL_NUM_SIZE,
                WORKER_MANAGER_SCHED_POOL_NAME,
            ),
            job_fetcher: JobFetcher::new(
                self.id,
                self.sender.clone(),
                S,
                self.pair.clone()),
            storage: S,
            // sender_t: SenderT::new(
            //     DashMap::default(),
            // ),
            workers: DashMap::default(),
            dispatcher: Dispatcher::new(),
            health_checker: HealthChecker::new(),
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
        let client = init_grpc_client(addr)?;
        self.workers.insert(addr.into(), client);
        Ok(())
    }

    pub fn register_task(&mut self, task: Task) -> Result<()> {
        Ok(())
    }

    pub fn unregister_task(&mut self, task_id: &u64) -> Result<()> {
        Ok(())
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

    /// Determine whether the current `WorkerManager` is service-side.
    #[inline]
    fn is_server_side(&self) -> bool {
        self.scope == ServerSide
    }

    /// Returns the status of `WorkerManager`.
    #[inline]
    pub fn get_status(&self) -> ComponentStatus {
        self.status.load()
    }
}
