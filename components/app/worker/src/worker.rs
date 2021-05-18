use fastjob_components_storage::model::task::Task;
use tokio::sync::mpsc::Receiver;

pub struct Worker {
    pub sequence_queue: Receiver<Task>,
    pub completed_queue: Receiver<Task>,
}
