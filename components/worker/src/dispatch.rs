use crate::error::Result;
use crate::WorkerClusterHolder;
use dashmap::DashMap;
use fastjob_components_storage::model::instance_info::{InstanceInfo, InstanceStatus};
use fastjob_components_storage::model::job_info::{JobInfo, JobStatus, JobTimeExpressionType};
use fastjob_components_storage::model::task::TimeExpressionType;
use fastjob_components_storage::Storage;
use snafu::ResultExt;
use std::cell::RefCell;
use std::convert::TryFrom;
use tokio::sync::mpsc::Receiver;

pub struct Dispatch<S: Storage> {
    task_receiver: Receiver<(JobInfo, u64)>,
    storage: S,
    workers: RefCell<DashMap<u64, WorkerClusterHolder>>,
}

impl<S: Storage> Dispatch<S> {
    pub fn new(
        task_receiver: Receiver<(JobInfo, u64)>,
        storage: S,
        workers: RefCell<DashMap<u64, WorkerClusterHolder>>,
    ) -> Self {
        Self {
            task_receiver,
            storage,
            workers,
        }
    }

    pub async fn event_loop(&mut self) {
        while let Some(task) = self.task_receiver.recv().await {
            info!(
                "[Dispatch Event-Loop] receive task id: {}",
                task.0.id.unwrap()
            );
            self.dispatch(task).await;
        }
    }
    pub async fn dispatch(&self, task: (JobInfo, u64)) -> Result<()> {
        debug!("[Dispatch] start dispatch: {}", task.0.id.unwrap());
        // 1. check the current instance whether canceled.
        if let Some(instance_info) = self.storage.find_instance_by_id(task.1).context("")? {
            match InstanceStatus::try_from(instance_info.status.unwrap()) {
                Some(InstanceStatus::Canceled) => {
                    warn!("[Dispatcher] The instance: {} is canceled.", task.1);
                    return Ok(());
                }
                // if it has been dispatched, it will not be dispatched.
                Some(status) if status != InstanceStatus::WaitingDispatch => {
                    warn!("[Dispatcher] The instance: {} has been dispatched.", task.1);
                    return Ok(());
                }
                Err(_) => {
                    error!(
                        "[Dispatcher] invalid instance: {} status:{}.",
                        task.1,
                        instance_info.status.unwrap()
                    )
                }
            }
            // scope for workflow job.
            if task.0.id.is_none() {
                warn!(
                    "[Dispatcher] The job: {} has been deleted.",
                    instance_info.job_id.unwrap()
                );
                // process this finished instance.
                self.process_completed_instance(
                    task.1,
                    InstanceStatus::Failed,
                    format!("can't find job by id {}", instance_info.job_id.unwrap()).as_str(),
                    instance_info.wf_instance_id,
                );
                return Ok(());
            }

            // 2. Determine the instance number whether the limit is exceeded.
            info!("[Dispatcher] Start to dispatch job: {}", task.0);
            match task.0.max_instance_num {
                Some(n) if n > 0 => {
                    let running_count = self
                        .storage
                        .count_instance_by_status(
                            instance_info.job_id.unwrap(),
                            vec![
                                InstanceStatus::WaitingWorkerReceive.into(),
                                InstanceStatus::Running.into(),
                            ],
                        )
                        .context()?;
                    if running_count >= n as u64 {
                        self.update_instance_trigger_failed(instance_info.clone());

                        // process this finished instance.
                        self.process_completed_instance(
                            task.1,
                            InstanceStatus::Failed,
                            format!("Too many instances, exceed max instance num: {}", n).as_str(),
                            instance_info.wf_instance_id,
                        );
                        return Ok(());
                    }
                }
                _ => {}
            }
            // 3. Choose the most suitable worker.
            self.choose_suitable_worker(&task.0)?;

            // 4. Construct the schedule task request.
            self.construct_schedule_job()?;

            // 5. Send request(unreliable，so need background thread to poll the status periodically).
            self.send()?;
            info!("[Dispatcher] Send schedule request( job id: {}, instance id: {} ) to worker address: {} successfully.");
            self.update_instance_trigger_success(instance_info.clone(), "");
        }
        Ok(())
    }

    /// Process the completed instance.
    fn process_completed_instance(
        &self,
        instance_id: u64,
        status: InstanceStatus,
        result: &'static str,
        wf_instance_id: Option<u64>,
    ) {
        info!(
            "[Dispatcher] Instance {} process finished, final status: {}.",
            instance_id, status
        );

        // alarm
        if status == InstanceStatus::Failed {}
    }

    fn update_instance_trigger_failed(&self, mut instance: InstanceInfo) -> Result<()> {
        let now = chrono::Local::now().timestamp_millis();
        instance.result =
            Some(format!("Too many instances, exceed max instance num: {}", n).as_str());
        instance.actual_trigger_time = Some(now);
        instance.finished_time = Some(now);
        instance.status = Some(InstanceStatus::Failed.into());
        self.storage.update(instance.as_mut_ref())?;
        Ok(())
    }

    fn update_instance_trigger_success(
        &self,
        mut instance: InstanceInfo,
        worker_address: &str,
    ) -> Result<()> {
        let now = chrono::Local::now().timestamp_millis();
        instance.actual_trigger_time = Some(now);
        instance.task_tracker_address = Some(worker_address);
        instance.status = Some(InstanceStatus::WaitingWorkerReceive.into());
        self.storage.update(instance.as_mut_ref())?;
        Ok(())
    }

    /// Choose the most suitable worker.
    fn choose_suitable_worker(&self, job_info: &JobInfo) -> Result<()> {
        Ok(())
    }

    fn construct_schedule_job(&self) -> Result<()> {
        Ok(())
    }

    fn send(&self) -> Result<()> {
        Ok(())
    }

    pub async fn redispatch(&self, mut instance: InstanceInfo) -> Result<()> {
        if let Some(job) = self
            .storage
            .find_job_info_by_instance_id(instance.instance_id.unwrap())
            .context(error::WorkerStorageError)?
        {
            if !job.status.unwrap() == JobStatus::Running
                || JobTimeExpressionType::try_from(job.time_expression_type)?.is_frequent()
                || instance.running_times.unwrap() >= job.instance_retry_num.unwrap()
            {
                instance.status = Some(InstanceStatus::Failed.into());
                instance.finished_time = Some(chrono::Local::now().timestamp_millis());
                // only this case will happen.
                instance.result = Some("worker report timeout, maybe Worker down");
                instance.running_times = Some(instance.running_times.unwrap().wrapping_add(1));
                self.storage.save(instance);
                return Ok(());
            }
            // redispatch.
            self.dispatch((job, instance.instance_id.unwrap())).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
