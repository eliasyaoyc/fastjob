mod worker;
pub mod worker_manager;
mod job_fetcher;

#[derive(Debug, Clone)]
pub struct Worker {
    id: usize,
}
