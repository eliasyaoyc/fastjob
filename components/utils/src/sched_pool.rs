use crate::yatp_pool::future_pool::FuturePool;
use crate::yatp_pool::{PoolTicker, YatpPoolBuilder};
use std::time::Duration;

#[derive(Clone)]
pub struct SchedPool {
    pub pool: FuturePool,
}

#[derive(Clone)]
pub struct SchedTicker;

impl PoolTicker for SchedTicker {
    fn on_tick(&mut self) {}
}

impl SchedPool {
    pub fn new(pool_size: usize, name_prefix: &str) -> Self {
        let pool = YatpPoolBuilder::new(SchedTicker {})
            .thread_count(pool_size, pool_size)
            .max_tasks(1024)
            .name_prefix(name_prefix)
            .build_future_pool();
        Self { pool }
    }

    /// Creates and executes a periodic action that becomes enabled first after the given initial delay,
    /// and subsequently with the given period; that is executions will commence after initialDelay then
    /// initialDelay+period,then initialDelay + 2 * period, and so on. If any execution of the task
    /// encounters an exception, subsequent executions are suppressed.Otherwise, the task will only
    /// terminate via cancellation or termination of the executor. If any execution of this task takes
    /// longer than its period,then subsequent executions may start late, but will not concurrently execute
    pub fn schedule_at_fixed_rate<F>(&self, mut f: F, initial_delay: Duration, period: Duration)
        where
            F: FnMut() + Sync + Send + 'static
    {
        self.pool.spawn(async move {
            f()
        });
    }

    /// Creates and executes a periodic action that becomes enabled first after the given initial delay,
    /// and subsequently with the given delay between the termination of one execution and the commencement
    /// of the next. If any execution of the task encounters an exception, subsequent executions are suppressed.
    /// Otherwise, the task will only terminat
    /// e via cancellation or termination of the executor.
    pub fn schedule_at_fixed_delay<F>(&self, mut f: F, initial_delay: Duration, period: Duration)
        where
            F: FnMut() + Sync + Send + 'static
    {
        self.pool.spawn(async move {
            f()
        });
    }
}
