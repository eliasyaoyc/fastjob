use tokio::sync::mpsc::Receiver;

pub struct Worker {
    pub sequence_queue: Receiver<Task>,
    pub completed_queue: Receiver<Task>,
}

#[derive(Clone, Debug)]
pub struct Task {

}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}