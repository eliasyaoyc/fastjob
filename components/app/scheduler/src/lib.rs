//! The dispatcher component is core component that responsible dispatch task to concrete scheduler.
//!
//! Below is a flow diagram for a task.
//! Task -> FastJobServer
//!                       -> WorkerManager
//!                                         -> Dispatcher  -> Scheduler
//!
//! Dispatcher runs a single-thread event loop, but task execution are delegated to Scheduler.
use std::sync::Arc;
use fastjob_components_utils::component::Component;
use crate::scheduler::Scheduler;

mod algo;
mod error;
mod scheduler;
mod sched_pool;

pub struct Dispatcher {
    scheduler: Arc<Scheduler>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let dispatcher = Self { scheduler: Arc::new(Scheduler::new()) };
    }
}

impl Component for Dispatcher {
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