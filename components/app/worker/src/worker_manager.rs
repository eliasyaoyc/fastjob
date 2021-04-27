use crate::Worker;
use fastjob_components_error::Error;
use crate::worker_manager::WorkerManagerScope::ServerSide;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerManagerScope {
    /// Belongs to `FastJobServer`.
    ServerSide,
    /// Belongs to `ApplicationSDK`.
    ApplicationSide,
}

#[derive(Clone, Debug)]
pub struct WorkerManagerConfig {}

#[derive(Clone, Debug)]
pub struct WorkerManager {
    id: usize,
    config: WorkerManagerConfig,
    scope: WorkerManagerScope,
    workers: Vec<Worker>,
}

impl WorkerManager {
    pub fn build(id: usize, config: &WorkerManagerConfig) -> Self {
        Self {
            id,
            config: config.clone(),
            scope: WorkerManagerScope::ServerSide,
            workers: vec![],
        }
    }

    fn pre_start(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn manual_sched(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn sched(&mut self) -> Result<(), Error> {
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