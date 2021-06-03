//! Scheduler which schedules the execution of `Task`. It receives commands from `WorkManager`.
//!
//! Scheduler keeps track of all the running task status and reports to `WorkerManager`.

#[macro_use]
extern crate fastjob_components_log;

use std::convert::TryFrom;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};
use delay_timer::prelude::*;
use delay_timer::prelude::{DelayTaskHandler, unblock_process_task_fn};
use snafu::ResultExt;

use error::Result;
use fastjob_components_storage::model::job_info::{JobInfo, JobStatus, JobTimeExpressionType, JobType};
use fastjob_components_utils::component::Component;
use fastjob_components_utils::pair::PairCond;
use fastjob_components_storage::Storage;
use fastjob_components_storage::model::app_info::AppInfo;

mod error;
mod dispatch;
mod instance_status_checker;

const SCHEDULE_RATE: usize = 15000;

pub struct Scheduler<S: Storage> {
    // receiver: Receiver<Vec<JobInfo>>,
    delay_timer: DelayTimer,
    // pair: Arc<PairCond>,
    // sender: Sender<JobInfo>,
    storage: S,
}

impl<S: Storage> Scheduler<S> {
    pub fn new(
        // receiver: Receiver<Vec<JobInfo>>,
        // sender: Sender<JobInfo>,
        storage: S,
    ) -> Self {
        Self {
            // receiver,
            delay_timer: DelayTimerBuilder::default().enable_status_report().build(),
            // pair,
            // sender,
            storage,
        }
    }

    pub fn schedule(&self) {
        info!("Schedule task start.");
        // let app_ids: Option<Vec<AppInfo>> = self.storage.find_all_by_current_server().context("")?;
        // let app_ids: Option<Vec<AppInfo>> = self.storage.find_all_by_current_server();
        // if app_ids.is_none() {
        //     info!("[JobScheduler] current server has no app's job to schedule.");
        //     return;
        // }


        // loop {
        //     if self.shutdown.load(Ordering::Relaxed) {
        //         break;
        //     }
        //     match self
        //         .receiver
        //         .recv_timeout(Duration::from_millis(500))
        //         .as_mut()
        //     {
        //         Ok(jobs) => {
        //             if jobs.is_empty() {
        //                 warn!("scheduler dispatcher recv empty, need to sleep.");
        //                 self.pair.wait();
        //             } else {
        //                 jobs.iter_mut().map(|job| {
        //                     if let Some(task) = self::build_task(job) {
        //                         self.delay_timer.add_task(task).context("")?;
        //                     }
        //                 });
        //             }
        //         }
        //         Err(_) => {
        //             warn!("scheduler dispatcher timeout recv, need to sleep.");
        //             self.pair.wait();
        //         }
        //     }
        // }
    }

    /// Schedule tasks of type CRON expressions.
    pub fn schedule_cron_job(&self, ids: Vec<&str>) -> Result<()> {
        Ok(())
    }

    /// Schedule second-level task.
    pub fn schedule_frequent_job(&self, ids: Vec<&str>) -> Result<()> {
        Ok(())
    }

    /// Schedule tasks of type worker-flow.
    pub fn schedule_worker_flow(&self, ids: Vec<&str>) -> Result<()> {
        //todo
        Ok(())
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
        self.delay_timer.update_task(task);
        Ok(())
    }

    pub fn cancel_task(&self, task_id: u64, record_id: i64) -> Result<()> {
        self.delay_timer.cancel_task(task_id, record_id);
        Ok(())
    }

    pub fn remove_task(&self, task_id: u64) -> Result<()> {
        self.delay_timer.remove_task(task_id);
        Ok(())
    }

    pub(crate) fn build_task<F>(&self, job: &mut JobInfo) -> Option<Task>
        where
            F: Fn(TaskContext) -> Box<dyn DelayTaskHandler> + 'static + Send + Sync,
    {
        if !job.is_running() {
            return None;
        }

        let body = match JobType::try_from(job.processor_type.unwrap()) {
            Ok(_) => {
                create_async_fn_body!({
                    // self.sender.send(job);
                })
            }
            Err(_) => {
                error!("Unknown atomic number: {}", job.processor_type.unwrap());
                return None;
            }
        };

        let frequency = match JobTimeExpressionType::try_from(job.time_expression_type.unwrap()) {
            Ok(_) => {
                // let expression = job.time_expression.unwrap();
                CandyFrequency::Repeated(CandyCronStr("".to_string()))
            }
            Err(_) => {
                error!("Unknown atomic number: {}", job.processor_type.unwrap());
                return None;
            }
        };

        let task = TaskBuilder::default()
            .set_task_id(job.id.unwrap())
            .set_frequency_by_candy(frequency)
            .set_maximun_parallel_runable_num(job.concurrency.unwrap_or(1) as u64)
            .spawn(body);
        // .context("")?;

        Some(task.unwrap())
    }

    pub fn shutdown(&self) {
        info!("Scheduler stop.");
        // self.pair.notify();
    }
}
