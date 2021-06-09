use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

pub use error::Result;
use fastjob_proto::fastjob::*;
use std::collections::HashMap;
use std::cmp::Ordering;
use fastjob_components_storage::model::job_info::JobInfo;

mod alarm_controller;
mod dispatch;
mod error;
mod instance_status_checker;
pub mod worker_manager;
mod event;

#[macro_use]
extern crate fastjob_components_log;

struct WorkerClusterHolder {
    app_name: &'static str,
    // all worker in the cluster.
    workers: HashMap<&'static str, Worker>,
    containers: HashMap<u64, HashMap<&'static str, DeployContainerInfo>>,
}

struct Worker {
    address: &'static str,
    last_active_time: i64,
    client: Option<grpcio::Client>,
    tag: &'static str,
    indicators: WorkerIndicators,
}

impl Eq for Worker {}

impl PartialEq for Worker {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

impl Ord for Worker {
    fn cmp(&self, other: &Self) -> Ordering {
        let available_memory = self.indicators.get_jvmMaxMemory() - self.indicators.get_jvmUsedMemory() as f32;
        let available_cpu = self.indicators.get_cpuProcessors() - self.indicators.get_cpuLoad();
        let o_available_memory = other.indicators.get_jvmMaxMemory() - other.indicators.get_jvmUsedMemory() as f32;
        let o_available_cpu = other.indicators.get_cpuProcessors() - other.indicators.get_cpuLoad();
        (available_memory + available_cpu).cmp(&(o_available_memory - o_available_cpu))
    }
}

impl PartialOrd for Worker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Worker {
    pub fn new() -> Self {
        Self {
            address: "",
            last_active_time: 0,
            client: None,
            tag: "",
            indicators: WorkerIndicators::new(),
        }
    }

    fn refresh(&mut self, heartbeat: &HeartBeatRequest) {
        self.address = heartbeat.get_workerAddress();
        self.last_active_time = heartbeat.get_heartbeatTime();
        self.tag = heartbeat.get_tag();
        self.indicators = heartbeat.get_indicators();
    }

    fn is_available(&self, job_info: &JobInfo) -> bool {
        // 1. determine job whether specified the worker.
        if let Some(d_workers) = &job_info.designated_workers {
            let workers: Vec<_> = d_workers.split(',').collect();
            if workers.contains(&self.address) || workers.contains(&self.tag) {
                return true;
            }
        }

        // 2. determine the worker whether expired.
        if chrono::Local::now().timestamp_millis() < self.last_active_time {
            warn!("[Worker - {}] unreported heartbeat for a long time.", self.address);
            return true;
        }
        // 3. determine the worker indicators whether is satisfied.
        let available_memory = self.indicators.get_jvmMaxMemory() - self.indicators.get_jvmUsedMemory();
        let available_disk = self.indicators.get_diskTotal() - self.indicators.get_diskUsed();
        let available_core = self.indicators.get_cpuProcessors() - self.indicators.get_cpuLoad();
        if available_memory < job_info.min_memory_space.unwrap()
            || available_disk < job_info.min_disk_space.unwrap()
            || available_core < job_info.min_cpu_cores.unwrap() {
            return true;
        }
        false
    }

    fn worker_clean(&self) {

    }

    #[inline]
    fn last_active_time(&self) -> i64 {
        self.last_active_time
    }
}

impl WorkerClusterHolder {
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            workers: Default::default(),
            containers: Default::default(),
        }
    }

    /// Update the worker status through HeartBeatRequest
    pub fn update_worker_status(&mut self, heartbeat: &HeartBeatRequest) {
        let address = heartbeat.get_workerAddress();
        let heartbeat_time = heartbeat.get_heartbeatTime();
        let worker = self.workers.entry(address).or_insert_with(|| {
            let mut worker = Worker::new();
            worker.refresh(heartbeat);
            worker
        });
        if heartbeat_time < worker.last_active_time() {
            warn!("[WorkerClusterHolder] receive the expired heartbeat from {}, server time: {}, heart time: {}",
                  self.app_name,
                  address,
                  chrono::Local::now().timestamp_millis(),
                  heartbeat_time,
            );
            return;
        }

        worker.refresh(heartbeat);

        let container_infos = heartbeat.get_deployContainerInfo();
        if !container_infos.is_empty() {
            for container in container_infos {
                self.containers
                    .entry(container.getr_containerId())
                    .or_insert_with(|| {
                        let mut map = HashMap::new();
                        map.insert(address, container);
                        map
                    });
            }
        }
    }

    /// Returns the most suitable worker and if have worker unavailable that will remove it.
    pub fn get_suitable_worker(&mut self, job_info: &JobInfo) -> Option<&[Worker]> {
        if self.workers.is_empty() {
            return None;
        }
        let mut worker: &[_] = self.workers.clone().values().collect();
        worker.retain(|_, &v| v.is_available(job_info));
        worker.sort();


        if worker.is_empty() {
            return None;
        }

        if let Some(count) = job_info.max_worker_count {
            if count > 0 && worker.len() > count {
                worker.split_at(count);
            }
        }
        Some(worker)
    }

    /// Returns all containers deployment status in worker.
    pub fn get_container_infos(&self, contain_id: u64) -> Option<&[DeployContainerInfo]> {
        if let Some(infos) = self.containers.get(&contain_id).take() {
            let ret: &[DeployContainerInfo] = infos.iter().map(|(k, v)| v.set_workerAddress(k)).collect();
            Some(ret)
        }
        None
    }

    /// Release all container meta.
    pub fn release_container(&mut self) {

    }

    #[inline]
    fn app_name(&self) -> &str {
        self.app_name
    }
}

fn init_grpc_client(addr: &str) -> Result<::grpcio::Client> {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(addr);
    let client = FastJobClient::new(ch);
    Ok(client)
    // let mut req = HelloRequest::default();
    // req.set_name("world".to_owned());
    // let reply = client.say_hello(&req).expect("rpc");
    // info!("Greeter received: {}", reply.get_message());
}
