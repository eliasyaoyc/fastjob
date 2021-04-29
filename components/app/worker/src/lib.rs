pub mod worker_manager;
mod worker;

#[derive(Debug, Clone)]
pub struct Worker {
    id: usize,
}