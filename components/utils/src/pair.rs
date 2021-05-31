use parking_lot::{Condvar, Mutex};
use std::sync::Arc;

pub struct Pair(pub Mutex<bool>, pub Condvar);

#[inline]
pub fn condvar_pair() -> Arc<Pair> {
    Arc::new(Pair(Mutex::new(false), Condvar::new()))
}
