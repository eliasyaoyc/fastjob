use crossbeam::channel::Sender;

/// A thread that periodically pulls job information from the database.
#[derive(Clone)]
pub struct JobFetcher {
    sender: Sender<()>,
}

impl JobFetcher {
    pub fn new(sender: Sender<()>) -> Self {
        Self { sender }
    }

    pub fn fetch(&self) {}
}