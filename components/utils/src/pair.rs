use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::fmt::{Debug, Formatter};

pub struct PairCond {
    pub mu: Mutex<bool>,
    pub cond: Condvar,
}

impl Debug for PairCond {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("pair_cond")
            .field(&self.mu)
            .field(&self.cond)
            .finish()
    }
}

impl PairCond {
    pub fn new() -> Arc<PairCond> {
        Arc::new(PairCond {
            mu: Mutex::new(false),
            cond: Condvar::default(),
        })
    }

    pub fn wait(&self) {
        let mut started = self.mu.lock();
        self.cond.wait(&mut started);
    }

    pub fn notify(&self) {
        let mut started = self.mu.lock();
        *started = true;
        self.cond.notify_all();
    }
}


