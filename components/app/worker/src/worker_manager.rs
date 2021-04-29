use crate::Worker;
use fastjob_components_error::Error;
use crate::worker_manager::WorkerManagerScope::ServerSide;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerManagerScope {
    /// Belongs to `FastJobServer`.
    ServerSide,
    /// Belongs to `ApplicationSDK`.
    ApplicationSide,
    EMPTY,
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
    pub fn builder(config: WorkerManagerConfig) -> Self {
        Self {
            id: 0,
            config,
            scope: WorkerManagerScope::EMPTY,
            workers: vec![],
        }
    }

    pub fn id(mut self, id: usize) -> Self {
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