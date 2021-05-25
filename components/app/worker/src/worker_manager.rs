//! Worker Manager only stores tasks and is not going to scheduling.
//! If worker manager checks itself is free, it will start the steal thread
//! that steals the task from another server in FastJob Cluster. However,
//! if don't have enough space that will reject task request and respond a full error message.
//! so client will retry this request that send to another server util success unless achieved
//! the maximum retry numbers and send has failed.
use crate::Worker;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Receiver;
use fastjob_components_error::Error;
use fastjob_components_storage::model::task::Task;
use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_proto::fastjob::{
    WorkerManagerConfig, WorkerManagerScope, WorkerManagerScope::ServerSide,
};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::Ordering::SeqCst;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum WorkerManagerScope {
//     /// Belongs to `FastJobServer`.
//     ServerSide,
//     /// Belongs to `ApplicationSDK`.
//     ApplicationSide,
//     EMPTY,
// }

// #[derive(Clone, Debug)]
// pub enum WorkerManagerStatus {
//     Initialized,
//     Ready,
//     Started,
//     Terminating,
//     Shutdown,
// }

// #[derive(Clone, Debug)]
// pub struct WorkerManagerConfig {}

pub struct WorkerManager {
    id: u64,
    status: AtomicCell<ComponentStatus>,
    config: WorkerManagerConfig,
    scope: WorkerManagerScope,

    wait_queue: HashMap<u64, Task>,
    // todo: instead of yatp scheduler pool
    workers: Vec<Worker>,
    // execute_queue:
}

impl Clone for WorkerManager {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Debug for WorkerManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct WorkerManagerBuilder {
    id: u64,
    status: AtomicCell<ComponentStatus>,
    config: WorkerManagerConfig,
    scope: WorkerManagerScope,
    workers: Vec<Worker>,
}

impl WorkerManagerBuilder {
    pub fn builder(config: WorkerManagerConfig) -> Self {
        Self {
            id: 0,
            status: AtomicCell::new(ComponentStatus::Initialized),
            config,
            scope: WorkerManagerScope::EMPTY,
            workers: vec![],
        }
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    pub fn scope(mut self, scope: WorkerManagerScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn build(self) -> WorkerManager {
        WorkerManager {
            id: self.id,
            status: self.status,
            config: self.config,
            scope: self.scope,
            workers: self.workers,
            wait_queue: Default::default(),
        }
    }
}

impl Component for WorkerManager {
    fn prepare(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Initialized);

        // 1. Prepare yatp schedule pool.

        // 2. Prepare checker thread.

        // 3. Prepare fetch job thread.
        self.status.store(ComponentStatus::Ready);
    }

    fn start(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Ready);

        // 1. Start yatp schedule pool

        // 2. Start checker thread that will check workloads and server itself whether have task,
        //    if not or less than the minimum threshold, it will steal from another server.

        // 3. Start fetch job thread.
        self.status.store(ComponentStatus::Starting);
        // code.

        self.status.store(ComponentStatus::Running);
    }

    fn stop(&mut self) {
        assert_eq!(self.status.load(), ComponentStatus::Running);
        self.status.store(ComponentStatus::Terminating);
        // code

        self.status.store(ComponentStatus::Shutdown);
    }
}

impl WorkerManager {
    // pub fn register_task(&mut self, task: Task) -> Result<(), Error> {
    //     if !self.wait_queue.contains_key(&task.task_id.unwrap()) {
    //         self.wait_queue.insert(task.task_id.unwrap().clone(), task);
    //     }
    //     Ok(())
    // }
    //
    // pub fn unregister_task(&mut self, task_id: &u64) -> Result<(), Error> {
    //     if self.wait_queue.contains_key(task_id) {
    //         self.wait_queue.remove(task_id);
    //     }
    //     Ok(())
    // }

    /// Manually perform a schedule.
    pub fn manual_sched(&mut self) -> Result<(), Error> {
        self.sched()
    }

    fn sched(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn add_worker(&mut self) {}

    fn stop_worker(&mut self) {}

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
