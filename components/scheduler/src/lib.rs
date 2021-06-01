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
use delay_timer::prelude::*;
use fastjob_components_storage::model::job_info::{JobInfo, JobTimeExpressionType, JobType};
use fastjob_components_utils::component::Component;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

mod algo;
mod error;

use delay_timer::prelude::{unblock_process_task_fn, DelayTaskHandler};
use error::Result;
use snafu::ResultExt;
use std::convert::TryFrom;
use std::mem::MaybeUninit;
use fastjob_components_utils::pair::PairCond;

mod scheduler;

#[macro_use]
extern crate fastjob_components_log;

pub struct Dispatcher {
    scheduler: Arc<Scheduler>,
    receiver: Receiver<Vec<JobInfo>>,
    delay_timer: DelayTimer,
    pair: Arc<PairCond>,
}

impl Dispatcher {
    pub fn new(receiver: Receiver<Vec<JobInfo>>, pair: Arc<PairCond>) -> Self {
        Self {
            scheduler: Arc::new(Scheduler::new(2)),
            receiver,
            delay_timer: DelayTimerBuilder::default().enable_status_report().build(),
            pair,
        }
    }

    fn dispatcher(&self) {
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }
            match self
                .receiver
                .recv_timeout(Duration::from_millis(500))
                .as_mut()
            {
                Ok(jobs) => {
                    if jobs.is_empty() {
                        warn!("scheduler dispatcher recv empty, need to sleep.");
                        self.condvar.wait(&mut started);
                    } else {
                        jobs.iter_mut().map(|job| {
                            if let Some(task) = self::build_task(job) {
                                self.delay_timer.add_task(task).context("")?;
                            }
                        });
                    }
                }
                Err(_) => {
                    warn!("scheduler dispatcher timeout recv, need to sleep.");
                    self.pair.wait();
                }
            }
        }
    }

    pub fn filter_task_record_id<P>(&self, predicate: P) -> Option<i64>
        where
            P: FnMut(&PublicEvent) -> bool,
    {
        let mut public_events = Vec::<PublicEvent>::new();

        while let Ok(public_event) = self.delay_timer.get_public_event() {
            public_events.push(public_event);
        }

        let public_event = public_events.into_iter().find(predicate)?;
        public_event.get_record_id()
    }

    pub fn update_task(&self, task: Task) -> Result<()> {
        self.delay_timer.update_task(task).context("")
    }

    pub fn cancel_task(&self, task_id: u64, record_id: i64) -> Result<()> {
        self.delay_timer.cancel_task(task_id, record_id).context("")
    }

    pub fn remove_task(&self, task_id: u64) -> Result<()> {
        self.delay_timer.remove_task(task_id).context("")
    }
}

pub(crate) fn build_task<F>(job: &mut JobInfo) -> Option<Task>
    where
        F: Fn(TaskContext) -> Box<dyn DelayTaskHandler> + 'static + Send + Sync,
{
    let body = match JobType::try_from(job.processor_type.unwrap()) {
        Some(JobType::Shell) => unblock_process_task_fn(job.processor_info.into()),
        Some(JobType::Java) => {
            create_async_fn_body!({})
        }
        None => {
            error!("Unknown atomic number: {}", job.processor_type.unwrap());
            return None;
        }
    };

    let frequency = match JobTimeExpressionType::try_from(job.time_expression_type.unwrap()) {
        Some(_) => CandyFrequency::Once(CandyCronStr("0/1 * * * * * *".to_string())),
        Some(JobTimeExpressionType::CRON) => {
            CandyFrequency::Repeated(CandyCronStr(job.time_expression.unwrap()))
        }
        None => {
            error!("Unknown atomic number: {}", job.processor_type.unwrap());
            return None;
        }
    };

    let task = TaskBuilder::default()
        .set_task_id(job.id.unwrap())
        .set_frequency_by_candy(frequency)
        .set_maximun_parallel_runable_num(job.concurrency.unwrap_or_else(1) as u64)
        .spawn(body)
        .context("")?;

    Some(task)
}

impl Component for Dispatcher {
    fn start(&mut self) {
        info!("startup dispatcher.");
        self.dispatcher()
    }

    fn stop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.pair.notify();
    }
}
