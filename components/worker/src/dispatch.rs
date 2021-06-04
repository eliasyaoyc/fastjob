use fastjob_components_storage::model::instance_info::{InstanceInfo, InstanceStatus};
use fastjob_components_storage::model::job_info::JobInfo;
use fastjob_components_storage::Storage;
use std::convert::TryFrom;
use crate::error::Result;
use snafu::ResultExt;

pub struct Dispatch<S: Storage> {
    task_receiver: async_channel::Receiver<(JobInfo, u64)>,
    storage: S,
}

impl<S: Storage> Dispatch<S> {
    pub fn new(task_receiver: async_channel::Receiver<(JobInfo, u64)>, storage: S) -> Self {
        Self {
            task_receiver,
            storage,
        }
    }

    pub async fn event_loop(&self) {
        while let Ok(task) = self.task_receiver.recv().await {
            info!(
                "[Dispatch Event-Loop] receive task id: {}",
                task.0.id.unwrap()
            );
            self.dispatch(&task).await;
        }
    }
    pub async fn dispatch(&self, task: &(JobInfo, u64)) -> Result<()> {
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
                self.process_completed_instance();
                return Ok(());
            }

            // 2. Determine the instance number whether the limit is exceeded.
            info!("[Dispatcher] Start to dispatch job: {}", task.0);
            match task.0.max_instance_num {
                Some(n) if n > 0 => {
                    let running_count = self.storage.count_instance_by_status(
                        instance_info.job_id.unwrap(),
                        vec![
                            InstanceStatus::WaitingWorkerReceive.into(),
                            InstanceStatus::Running.into(),
                        ],
                    ).context()?;
                    if running_count >= n {
                        self.update_instance_trigger_failed(instance_info.clone());
                        // process this finished instance.
                        self.process_completed_instance();
                        return Ok(());
                    }
                }
                _ => {}
            }
            // 3. Choose the most suitable worker.
            self.choose_suitable_worker()?;

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
    fn process_completed_instance(&self) {}

    fn update_instance_trigger_failed(&self, mut instance: InstanceInfo) -> Result<()> {
        let now = chrono::Local::now().timestamp_millis();
        instance.result = Some(format!("Too many instances, exceed max instance num: {}", n));
        instance.actual_trigger_time = Some(now);
        instance.finished_time = Some(now);
        instance.status = Some(InstanceStatus::Failed.into());
        self.storage.update(instance.as_mut_ref())?;
        Ok(())
    }

    fn update_instance_trigger_success(&self, mut instance: InstanceInfo, worker_address: &str) -> Result<()> {
        let now = chrono::Local::now().timestamp_millis();
        instance.actual_trigger_time = Some(now);
        instance.task_tracker_address = Some(worker_address.to_string());
        instance.status = Some(InstanceStatus::WaitingWorkerReceive.into());
        self.storage.update(instance.as_mut_ref())?;
        Ok(())
    }

    fn choose_suitable_worker(&self) -> Result<()> {
        Ok(())
    }

    fn construct_schedule_job(&self) -> Result<()> {
        Ok(())
    }

    fn send(&self) -> Result<()> {
        Ok(())
    }

    fn redispatch(&self) {}
}

#[cfg(test)]
mod tests {}
