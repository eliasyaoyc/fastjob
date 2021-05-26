//! Scheduler which schedules the execution of `Task`. It receives commands from `WorkManager`.
//!
//! Scheduler keeps track of all the running task status and reports to `WorkerManager`.
use fastjob_components_utils::component::Component;
use crate::algo::Algorithm;
use crossbeam_utils::CachePadded;
use std::sync::Mutex;
use std::collections::HashMap;
use crate::sched_pool::SchedPool;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Stores context of a task.
struct TaskContext {}

const SCHED_PENDING_TASK_THRESHOLD: usize = 10;


#[derive(Clone)]
struct SchedulerInner {
    task_slots: Vec<CachePadded<Mutex<HashMap<u64, TaskContext>>>>,
    worker_pool: SchedPool,
    high_priority_pool: SchedPool,
    // used to control flow.
    running_task_num: AtomicUsize,
}

impl SchedulerInner {
    fn too_busy(&self) -> bool {
        self.running_task_num.load(Ordering::Acquire) >= SCHED_PENDING_TASK_THRESHOLD
    }
}

#[derive(Clone)]
pub struct Scheduler {
    id: u64,
    inner: SchedulerInner,
}

impl Scheduler {
    /// Creates a scheduler.
    pub fn new(
        worker_pool_size: usize,
    ) -> Self {
        Self {
            id: 0,
            inner: SchedulerInner {
                task_slots: vec![],
                worker_pool: SchedPool::new(worker_pool_size, "sched-worker-pool"),
                high_priority_pool: SchedPool::new(
                    std::cmp::max(1, worker_pool_size / 2),
                    "sched-high-pri-pool"),
                running_task_num: Default::default(),
            },
        }
    }

    /// Determine Scheduler whether processing flow control,true that return TooBusy error
    /// otherwise invoke `schedule_task`.
    pub fn run_task(&self) {
        // flow control
        if self.inner.too_busy() {
            return;
        }
        self.schedule_task()
    }

    /// Schedule task that chooses correct algorithm and sched pool.
    fn schedule_task(&self) {
        self.choose_algo();
    }

    /// Execute task and report to `WorkerManager`.
    fn execute(&self) {
        self.report_task_msg();
    }

    /// Report the current task msg to `WorkerManager`.
    fn report_task_msg(&self) {}

    /// Select the correct scheduling algorithm for task.
    fn choose_algo(&self) {}
}