use std::sync::Arc;
use yatp::task::future;

pub type ThreadPool = yatp::ThreadPool<future::TaskCell>;

#[derive(Clone)]
pub struct FuturePool {
    pool: Arc<ThreadPool>,
    pool_size: usize,
    max_task: usize,
}

impl std::fmt::Debug for FuturePool {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "FuturePool")
    }
}

unsafe impl Send for FuturePool {}

unsafe impl Sync for FuturePool {}


impl FuturePool {
    pub fn from_pool(pool: ThreadPool, name: &str, pool_usize: usize, max_tasks: usize) -> Self {
        FuturePool {
            pool: Arc::new(pool),
            pool_size: pool_usize,
            max_task: max_tasks,
        }
    }
}