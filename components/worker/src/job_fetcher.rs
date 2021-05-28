use super::Result;
use crossbeam::channel::Sender;
use fastjob_components_storage::Storage;
use fastjob_components_utils::sched_pool::JobHandle;

/// A thread that periodically pulls job information from the database.
#[derive(Clone)]
pub struct JobFetcher<S: Storage> {
    server_id: i64,
    sender: Sender<()>,
    job_handler: JobHandle,
    storage: S,
}

impl<S: Storage> JobFetcher<S> {
    pub fn new(server_id: i64, sender: Sender<()>, storage: S) -> Self {
        Self {
            server_id,
            sender,
            job_handler: Default::default(),
            storage,
        }
    }

    pub fn fetch(&self) -> Result<()> {
        self.storage.fetch_page();
        self.sender.send(()).unwrap();
        Ok(())
    }

    #[inline]
    pub fn set_handler(&mut self, job_handler: JobHandle) {
        self.job_handler = job_handler;
    }

    #[inline]
    pub fn canceled(&self) {
        self.job_handler.cancel();
    }
}
