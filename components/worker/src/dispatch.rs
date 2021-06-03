use crate::error::Result;
use crossbeam::channel::Receiver;
use crossbeam::utils::CachePadded;
use dashmap::DashMap;
use fastjob_components_storage::model::job_info::JobInfo;
use fastjob_components_utils::pair::PairCond;
use fastjob_components_utils::sched_pool::SchedPool;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const SCHED_PENDING_TASK_THRESHOLD: usize = 10;

pub enum Event {
    AddClientEvent(String, ::grpcio::Client),
    RemoveClientEvent(String),
}

/// Sender that send the RPC messages to the 'Worker'.
pub struct Sender {
    inner: SenderInner,
    recv: Receiver<JobInfo>,
    shutdown: AtomicBool,
    pair: Arc<PairCond>,
}

impl Debug for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("sender")
            .field(&self.inner.connects)
            .field(&self.inner.task_slots)
            .field(&self.shutdown)
            .finish()
    }
}

struct SenderInner {
    /// Manager all worker clients.
    connects: DashMap<String, ::grpcio::Client>,
    /// Ready to send.
    task_slots: Vec<CachePadded<DashMap<u64, TaskContext>>>,
    worker_pool: SchedPool,
    failure_pool: SchedPool,
}

impl SenderInner {
    fn new(clients: DashMap<String, ::grpcio::Client>) -> Self {
        Self {
            connects: clients,
            task_slots: vec![],
            worker_pool: SchedPool::new(num_cpus::get(), "sender-worker-pool"),
            failure_pool: SchedPool::new(
                std::cmp::max(1, num_cpus::get() / 2),
                "sender-failure-pool",
            ),
        }
    }
}

/// Stores context of a task.
struct TaskContext {}

impl Sender {
    pub fn new(
        clients: DashMap<String, ::grpcio::Client>,
        recv: Receiver<JobInfo>,
        pair: Arc<PairCond>,
    ) -> Self {
        Self {
            inner: SenderInner::new(clients),
            recv,
            shutdown: Default::default(),
            pair,
        }
    }
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::AddClientEvent(key, client) => self.get_connects().insert(key, client),
            Event::RemoveClientEvent(key) => self.get_connects().remove(&key),
        };
    }

    fn get_connects(&mut self) -> &mut DashMap<String, ::grpcio::Client> {
        &mut self.inner.connects
    }

    pub fn run(&self) {
        info!("Sender run.");
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }
            match self.recv.recv_timeout(Duration::from_millis(500)).as_mut() {
                Ok(job) => {
                    info!("Sender start trigger sched-task, id: {}", job.id.unwrap());
                }
                Err(_) => {
                    warn!("Sender timeout recv, need to sleep.");
                    self.pair.wait();
                }
            }
        }
    }

    pub fn stop(&self) {}
}

#[cfg(test)]
mod tests {}