use fastjob_components_storage::model::job_info::JobInfo;
use fastjob_components_storage::Storage;
use fastjob_components_storage::model::instance_info::{InstanceInfo, InstanceStatus};
use std::convert::TryFrom;

pub struct Dispatch<S: Storage> {
    task_receiver: async_channel::Receiver<(JobInfo, u64)>,
    storage: S,
}

impl<S: Storage> Dispatch<S> {
    pub fn new(
        task_receiver: async_channel::Receiver<(JobInfo, u64)>,
        storage: S,
    ) -> Self {
        Self { task_receiver, storage }
    }

    pub async fn event_loop(&self) {
        while let Ok(task) = self.task_receiver.recv().await {
            info!("[Dispatch Event-Loop] receive task id: {}", task.0.id.unwrap());
            self.dispatch(task).await;
        }
    }
    pub async fn dispatch(&self, task: (JobInfo, u64)) {
        info!("[Dispatch] start dispatch: {}", task.0.id.unwrap());
        // 1. check the current instance whether canceled.
        if let Some(instance_info) = self.storage.find_instance_by_id(task.1).unwrap() {
            match InstanceStatus::try_from(instance_info.status.unwrap()) {
                Some(InstanceStatus::Canceled) => {
                    info!("[Dispatch] The instance: {} is canceled", task.1);
                    return;
                }
                Err(_) => { error!("[Dispatch] invalid instance: {} status:{}", task.1, instance_info.status.unwrap()) }
            }
        }
    }

    fn redispatch(&self) {}
}

#[cfg(test)]
mod tests {}