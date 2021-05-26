mod job_fetcher;
mod worker;
pub mod worker_manager;
mod error;

pub use error::Result;

#[derive(Debug, Clone)]
pub struct Worker {
    id: usize,
}
