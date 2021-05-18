use crate::Worker;
use fastjob_components_error::Error;
// use crate::worker_manager::;
use crossbeam::channel::Receiver;
use fastjob_components_storage::model::task::Task;
use std::collections::HashMap;
use fastjob_proto::fastjob::{WorkerManagerScope, WorkerManagerScope::ServerSide, WorkerManagerConfig};

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum WorkerManagerScope {
//     /// Belongs to `FastJobServer`.
//     ServerSide,
//     /// Belongs to `ApplicationSDK`.
//     ApplicationSide,
//     EMPTY,
// }

#[derive(Clone, Debug)]
pub enum WorkerManagerStatus {
    Ready,
    Starting,
}

// #[derive(Clone, Debug)]
// pub struct WorkerManagerConfig {}

#[derive(Clone, Debug)]
pub struct WorkerManager {
    id: u64,
    status: WorkerManagerStatus,
    config: WorkerManagerConfig,
    scope: WorkerManagerScope,
    workers: Vec<Worker>,
    tasks: HashMap<u64, Task>,
}

impl WorkerManager {
    pub fn builder(config: &WorkerManagerConfig) -> Self {
        Self {
            id: 0,
            status: WorkerManagerStatus::Ready,
            config: config.clone(),
            scope: WorkerManagerScope::EMPTY,
            workers: vec![],
            tasks: HashMap::new(),
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

    pub fn build(self) -> Self {
        // init worker pool.
        self
    }

    pub fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn register_task(&mut self, task: Task) -> Result<(), Error> {
        if !self.tasks.contains_key(&task.task_id.unwrap()) {
            self.tasks.insert(task.task_id.unwrap().clone(), task);
        }
        Ok(())
    }

    pub fn unregister_task(&mut self, task_id: &u64) -> Result<(), Error> {
        if self.tasks.contains_key(task_id) {
            self.tasks.remove(task_id);
        }
        Ok(())
    }

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
}