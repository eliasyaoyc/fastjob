//! Scheduler which schedules the execution of `Task`. It receives commands from `WorkManager`.
//!
//! Scheduler runs in a single-thread event loop, but task executions are delegated to a pool of
//! worker thread.
//!
//! Scheduler keeps track of all the running task status and reports to `WorkerManager`.
use fastjob_components_utils::component::Component;
use crate::algo::Algorithm;

mod algo;
mod error;

struct SchedulerInner {}

impl SchedulerInner {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct Scheduler<A: Algorithm> {
    algo: Option<A>,
    inner: SchedulerInner,
}

impl<A: Algorithm> Scheduler<A> {
    /// Creates a scheduler.
    pub fn new(
        algo: A,
    ) -> Self {
        Self {
            algo,
            inner: SchedulerInner::new(),
        }
    }
}

impl<A: Algorithm> Component for Scheduler<A> {
    fn prepare(&mut self) {
        todo!()
    }

    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
