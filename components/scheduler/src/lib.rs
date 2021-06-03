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
use fastjob_components_storage::model::app_info::AppInfo;
use fastjob_components_storage::model::job_info::{JobInfo, JobStatus, JobTimeExpressionType, JobType};
use fastjob_components_storage::Storage;
use fastjob_components_utils::component::Component;
use fastjob_components_utils::pair::PairCond;
use fastjob_components_storage::model::instance_info::InstanceInfo;
use crate::dispatch::Dispatch;

pub mod error;
mod dispatch;
mod instance_status_checker;
mod container;
mod mapreduce;
mod workflow;
mod rt;

pub const SCHEDULE_INTERVAL: Duration = Duration::from_millis(10000);

pub struct Scheduler<S: Storage> {
    delay_timer: DelayTimer,
    storage: S,
    dispatch: Dispatch,
}

impl<S: Storage> Scheduler<S> {
    pub fn new(
        storage: S,
    ) -> Self {
        Self {
            delay_timer: DelayTimerBuilder::default().enable_status_report().build(),
            storage,
            dispatch: Dispatch {},
        }
    }

    /// Schedule tasks of type CRON expressions.
    pub fn schedule_cron_job(&self, ids: &[u64]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let now = chrono::Local::now().timestamp_millis();
        let mut chunks = ids.chunks(10);
        for chunk in chunks {
            rt::build().block_on(async move {
                let job_infos = self.storage.find_cron_jobs(chunk, now + SCHEDULE_INTERVAL.as_millis() * 2).context("")?;
                if job_infos.is_empty() {
                    return;
                }
                info!("[Cron Scheduler] all cron jobs ready to scheduled: {}", job_infos);

                // 1. write record to instance table.
                let instances: Vec<(u64, u64)> = vec![];
                let mut job_instance_map = std::collections::HashMap::with_capacity(job_infos.len());
                for job in job_infos {
                    let instance_info = InstanceInfo::create(
                        job.get_id(),
                        job.get_app_id(),
                        job.get_job_params(),
                        None,
                        None,
                        job.get_next_trigger_time());
                    job_instance_map.insert(job.id.unwrap(), instance_info.id.unwrap());
                }
                self.storage.save_batch(instances.as_slice());

                // 2. push to timing wheel, then waiting be triggered.
                for job in job_infos {
                    let instance_id = job_instance_map.get(&job.id.unwrap());
                    let target_trigger_time = job.get_next_trigger_time().unwrap();
                    let delay = if target_trigger_time < now {
                        warn!("[Job-{}] schedule delay, expect: {}, current: {}", job.id.unwrap(), target_trigger_time, chrono::Local::now().timestamp_millis());
                        0
                    } else {
                        target_trigger_time - now
                    };
                    // push to timing wheel, consider use tokio library?.

                }

                // 3. calculate job the next trigger time.(ignore repeat execute in 5s, i.e.
                // the minimum continuous execution interval in cron mode is SCHEDULE_INTERVAL ms).
                self.refresh_job(job_infos)?;
            })
        }
        Ok(())
    }

    /// Schedule second-level task.
    pub fn schedule_frequent_job(&self, ids: &[u64]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut chunks = ids.chunks(10);
        for chunk in chunks {
            rt::build().block_on(async move {
                let mut job_infos = self.storage.find_frequent_jobs(chunk).context("")?;
                if job_infos.is_empty() {
                    return;
                }

                let job_ids = self.storage.find_frequent_instance_by_job_id(chunk).context("")?;

                job_infos.retain(|job| !job_ids.contains(&job.id.unwrap()));

                if job_infos.is_empty() {
                    return Ok(());
                }

                info!("[Frequent Scheduler] all frequent jobs ready to scheduled: {}", job_infos);

                for job in job_infos {
                    let instance_info = InstanceInfo::create(
                        job.get_id(),
                        job.get_app_id(),
                        job.get_job_params(),
                        None,
                        None,
                        Some(chrono::Local::now().timestamp_millis()));
                    self.storage.save(instance_info)?;
                    self.dispatch.dispatch(job.clone(), instance_info.id.unwrap());
                }
                Ok(())
            })
        }
        Ok(())
    }

    /// Schedule tasks of type worker-flow.
    pub fn schedule_worker_flow(&self, ids: &[u64]) -> Result<()> {
        //todo
        Ok(())
    }

    fn refresh_job(&self, jobs: Vec<JobInfo>) {}

    fn refresh_workflow(&self) {}

    fn calculate_next_trigger_time(&self) {}

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
    }
}
