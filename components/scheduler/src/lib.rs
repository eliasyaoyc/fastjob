//! Scheduler which schedules the execution of `Task`. It receives commands from `WorkManager`.
//!
//! Scheduler keeps track of all the running task status and reports to `WorkerManager`.

#[macro_use]
extern crate fastjob_components_log;

use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cron::Schedule;
use delay_timer::prelude::*;
use snafu::ResultExt;

use error::Result;
use fastjob_components_storage::model::{
    app_info::AppInfo,
    instance_info::InstanceInfo,
    job_info::{JobInfo, JobStatus, JobTimeExpressionType, JobType},
};
use fastjob_components_storage::Storage;
use fastjob_components_utils::component::Component;

use crate::dispatch::Dispatch;
use std::fmt::{Debug, Formatter};

mod container;
pub mod error;
mod mapreduce;
mod rt;
mod workflow;

pub const SCHEDULE_INTERVAL: Duration = Duration::from_millis(10000);

pub struct Scheduler<S: Storage> {
    delay_timer: DelayTimer,
    storage: S,
    task_sender: async_channel::Sender<(JobInfo, u64)>,
}

impl<S: Storage> Debug for Scheduler<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("scheduler")
            .field("delay_timer",&self.delay_timer)
            .finish()
    }
}

impl<S: Storage> Scheduler<S> {
    pub fn new(storage: S, task_sender: async_channel::Sender<(JobInfo, u64)>) -> Self {
        Self {
            delay_timer: DelayTimerBuilder::default().enable_status_report().build(),
            storage,
            task_sender,
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
                let job_infos = self
                    .storage
                    .find_cron_jobs(chunk, now + SCHEDULE_INTERVAL.as_millis() * 2)
                    .context("")?;
                if job_infos.is_empty() {
                    return;
                }
                info!(
                    "[Cron Scheduler] all cron jobs ready to scheduled: {}",
                    job_infos
                );

                // 1. write record to instance table.
                let instances: Vec<(u64, u64)> = vec![];
                let mut job_instance_map =
                    std::collections::HashMap::with_capacity(job_infos.len());
                for job in job_infos {
                    let instance_info = InstanceInfo::create(
                        job.get_id(),
                        job.get_app_id(),
                        job.get_job_params(),
                        None,
                        None,
                        job.get_next_trigger_time(),
                    );
                    job_instance_map.insert(job.id.unwrap(), instance_info.id.unwrap());
                }
                self.storage.save_batch(instances.as_slice());

                // 2. push to timing wheel, then waiting be triggered.
                for job in job_infos {
                    let instance_id = job_instance_map.get(&job.id.unwrap());
                    let target_trigger_time = job.get_next_trigger_time().unwrap();

                    // let delay = if target_trigger_time < now {
                    //     warn!("[Job-{}] schedule delay, expect: {}, current: {}", job.id.unwrap(), target_trigger_time, chrono::Local::now().timestamp_millis());
                    //     0
                    // } else {
                    //     target_trigger_time - now
                    // };

                    // push to timing wheel, consider use tokio library?.
                    if instance_id.is_none() {
                        return Ok(());
                    }
                    if let Some(task) = self.build_task(job.clone(), instance_id.unwrap().clone()) {
                        self.delay_timer.add_task(task);
                    }
                    // 3. calculate job the next trigger time.(ignore repeat execute in 5s, i.e.
                    // the minimum continuous execution interval in cron mode is SCHEDULE_INTERVAL ms).
                    self.refresh_job(job.clone())?;
                }
                Ok(())
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

                let job_ids = self
                    .storage
                    .find_frequent_instance_by_job_id(chunk)
                    .context("")?;

                job_infos.retain(|job| !job_ids.contains(&job.id.unwrap()));

                if job_infos.is_empty() {
                    return Ok(());
                }

                info!(
                    "[Frequent Scheduler] all frequent jobs ready to scheduled: {}",
                    job_infos
                );

                for job in job_infos {
                    let instance_info = InstanceInfo::create(
                        job.get_id(),
                        job.get_app_id(),
                        job.get_job_params(),
                        None,
                        None,
                        Some(chrono::Local::now().timestamp_millis()),
                    );
                    self.storage.save(instance_info)?;
                    self.dispatch
                        .dispatch(job.clone(), instance_info.id.unwrap());
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

    fn refresh_job(&self, mut job: JobInfo) -> Result<()> {
        match job.time_expression {
            Some(express) => {
                let next_trigger_time = self.calculate_next_trigger_time(express.as_str())?;
                job.next_trigger_time = Some(next_trigger_time);
            }
            None => {
                job.status = Some(JobStatus::DISABLED.into());
            }
        }
        self.storage.update(&mut job)?;
        Ok(())
    }

    fn refresh_workflow(&self) {}

    fn calculate_next_trigger_time(&self, expression: &str) -> Result<i64> {
        let next_trigger_time = Schedule::from_str(expression)?
            .upcoming(Local)
            .next()
            .unwrap();
        Ok(next_trigger_time.timestamp_millis())
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

    // pub(crate) fn build_task<F>(&self, job: JobInfo, delay: i64, instance_id: u64) -> Result<Option<Task>>
    pub(crate) fn build_task<F>(&self, job: JobInfo, instance_id: u64) -> Result<Option<Task>>
        where
            F: Fn(TaskContext) -> Box<dyn DelayTaskHandler> + 'static + Send + Sync,
    {
        let body = match JobType::try_from(job.processor_type.unwrap()) {
            Ok(_) => {
                create_async_fn_body!({
                    info!("Job {} start running", job.id.unwrap());
                    self.task_sender.send((job.clone(), instance_id));
                })
            }
            Err(_) => {
                error!("Unknown atomic number: {}", job.processor_type.unwrap());
                // i think unimportant for this error, so just log
                return Ok(None);
            }
        };

        let frequency = match JobTimeExpressionType::try_from(job.time_expression_type.unwrap()) {
            Ok(_) => CandyFrequency::Once(CandyCronStr("".to_string())),
            Err(_) => {
                error!(
                    "Unknown atomic number: {}",
                    job.time_expression_type.unwrap()
                );
                // i think unimportant for this error, so just log
                return Ok(None);
            }
        };

        let task = TaskBuilder::default()
            .set_task_id(job.id.unwrap())
            .set_frequency_by_candy(CandyFrequency::Once(job.time_expression.unwrap()))
            .set_maximun_parallel_runable_num(job.concurrency.unwrap_or(1) as u64)
            .spawn(body)
            .context(error::ConstructorTaskFailed {
                task_id: job.id.unwrap(),
            })?;

        Ok(Some(task))
    }

    pub fn shutdown(&self) {
        info!("Scheduler stop.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t_cron() {
        let expression = "0   30   9,12,15     1,15       May-Aug  Mon,Wed,Fri  2018/2";
        let schedule = Schedule::from_str(expression).unwrap();
        println!("Upcoming fire times:");
        let a = schedule.upcoming(Utc).next().unwrap();
        let b = a.max(chrono::Utc::now());
        println!("-> {}", b);
    }
}
