use fastjob_components_utils::yatp_pool::future_pool::FuturePool;
use fastjob_components_utils::yatp_pool::{PoolTicker, YatpPoolBuilder};

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
            .name_prefix(name_prefix)
            .build_future_pool();
        Self { pool }
    }
}