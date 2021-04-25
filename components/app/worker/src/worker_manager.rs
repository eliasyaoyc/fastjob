use crate::Worker;
use fastjob_components_error::Error;
use crate::worker_manager::WorkerManagerScope::ServerSide;

pub enum WorkerManagerScope {
    /// Belongs to `FastJobServer`.
    ServerSide,
    /// Belongs to `ApplicationSDK`.
    ApplicationSide,
}

#[derive(Copy, Clone)]
pub struct WorkerManager {
    id: usize,
    scope: WorkerManagerScope,
    workers: Vec<Worker>,
}

impl WorkerManager {
    pub fn build() -> Self {
        Self { id: 0, scope: WorkerManagerScope::ServerSide, workers: vec![] }
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