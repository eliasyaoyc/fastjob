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
    pub fn scheduler(&self) -> Result<(), Error> {
        Ok(())
    }

    fn add_worker(&self) {}

    fn stop_worker(&self) {}

    /// Determine whether the current `WorkerManager` is service-side.
    #[inline]
    fn is_server_side(&self) -> bool {
        self.scope == ServerSide
    }
}