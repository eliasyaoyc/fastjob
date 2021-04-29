use crate::meta::MetaManager;
use fastjob_components_scheduler::SchedulerManger;
use fastjob_components_worker::worker_manager::WorkerManager;

/// Service handles the RPC messages for the `FastJob` service.
pub struct Service {
    // storage: Storage,
    meta_mgr: MetaManager,
    sched_mgr: SchedulerManger,
    work_mgrs: Vec<WorkerManager>,
}

impl Service {
    pub fn new(meta_mgr: MetaManager) -> Self {
        Self {
            meta_mgr,
            sched_mgr: SchedulerManger::new(),
            work_mgrs: vec![],
        }
    }
}