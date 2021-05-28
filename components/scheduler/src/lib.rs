//! The dispatcher component is core component that responsible dispatch task to concrete scheduler.
//!
//! Below is a flow diagram for a task.
//! Task -> FastJobServer
//!                       -> WorkerManager
//!                                         -> Dispatcher  -> Scheduler
//!
//! Dispatcher runs a single-thread event loop, but task execution are delegated to Scheduler.
use crate::scheduler::Scheduler;
use crossbeam::channel::Receiver;
use delay_timer::entity::{DelayTimer, DelayTimerBuilder};
use fastjob_components_utils::component::Component;
use std::sync::Arc;

mod algo;
mod error;
mod scheduler;

pub struct Dispatcher {
    scheduler: Arc<Scheduler>,
    receiver: Receiver<()>,
    delay_timer: DelayTimer,
}

impl Dispatcher {
    pub fn new(receiver: Receiver<()>) -> Self {
        Self {
            scheduler: Arc::new(Scheduler::new(2)),
            receiver,
            delay_timer: DelayTimerBuilder::default().enable_status_report().build(),
        }
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
