use super::Result;
use crossbeam::channel::Sender;
use fastjob_components_storage::{Storage, Wrapper};
use fastjob_components_utils::sched_pool::JobHandle;
use fastjob_components_storage::model::job_info::JobInfo;
use snafu::ResultExt;

/// A thread that periodically pulls job information from the database.
#[derive(Clone)]
pub struct JobFetcher<S: Storage> {
    worker_manager_id: i64,
    sender: Sender<Vec<JobInfo>>,
    job_handler: JobHandle,
    storage: S,
}

impl<S: Storage> JobFetcher<S> {
    pub fn new(worker_manager_id: i64, sender: Sender<Vec<JobInfo>>, storage: S) -> Self {
        Self {
            worker_manager_id,
            sender,
            job_handler: Default::default(),
            storage,
        }
    }

    pub fn fetch(&self) -> Result<()> {
        let mut page_no = 0;
        let wrapper = self.storage.get_wrapper()
            .eq("designated_workers", self.worker_manager_id);

        while let Ok(v) = self.storage.fetch_page(&wrapper, page_no, 10) {
            if v.records.is_empty() {
                break;
            }
            page_no = v.page_no * v.page_size;
            self.sender.send(v.records).context("job fetcher send failed.")?;
        }
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
