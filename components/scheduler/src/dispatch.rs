// //! The dispatcher component is core component that responsible dispatch task to concrete scheduler.
// //!
// //! Below is a flow diagram for a task.
// //! Task -> FastJobServer
// //!                       -> WorkerManager
// //!                                         -> Dispatcher  -> Scheduler
// //!
// //! Dispatcher runs a single-thread event loop, but task execution are delegated to Scheduler.
//
// use crate::algo::Algorithm;
// use crate::sched_pool::SchedPool;
// use crossbeam::utils::CachePadded;
// use crossbeam_utils::CachePadded;
// use fastjob_components_utils::component::Component;
// use fastjob_components_utils::sched_pool::SchedPool;
// use std::collections::HashMap;
// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::Mutex;
//
// /// Stores context of a task.
// struct TaskContext {}
//
// const SCHED_PENDING_TASK_THRESHOLD: usize = 10;
//
// #[derive(Clone)]
// struct SchedulerInner {
//     task_slots: Vec<CachePadded<Mutex<HashMap<u64, TaskContext>>>>,
//     worker_pool: SchedPool,
//     high_priority_pool: SchedPool,
//     // used to control flow.
//     running_task_num: AtomicUsize,
// }
//
// impl SchedulerInner {
//     fn too_busy(&self) -> bool {
//         self.running_task_num.load(Ordering::Acquire) >= SCHED_PENDING_TASK_THRESHOLD
//     }
// }
//
// #[derive(Clone)]
// pub struct Dispatch {
//     id: u64,
//     inner: SchedulerInner,
// }
//
// impl Dispatch {
//     /// Creates a scheduler.
//     pub fn new(worker_pool_size: usize) -> Self {
//         Self {
//             id: 0,
//             inner: SchedulerInner {
//                 task_slots: vec![],
//                 worker_pool: SchedPool::new(worker_pool_size, "sched-worker-pool"),
//                 high_priority_pool: SchedPool::new(
//                     std::cmp::max(1, worker_pool_size / 2),
//                     "sched-high-pri-pool",
//                 ),
//                 running_task_num: Default::default(),
//             },
//         }
//     }
//
//     /// Determine Scheduler whether processing flow control,true that return TooBusy error
//     /// otherwise invoke `schedule_task`.
//     pub fn run_task(&self) {
//         // flow control
//         if self.inner.too_busy() {
//             return;
//         }
//         self.schedule_task()
//     }
//
//     /// Schedule task that chooses correct algorithm and sched pool.
//     fn schedule_task(&self) {
//         self.choose_algo();
//     }
//
//     /// Execute task and report to `WorkerManager`.
//     fn execute(&self) {
//         self.report_task_msg();
//     }
//
//     /// Report the current task msg to `WorkerManager`.
//     fn report_task_msg(&self) {}
// }