use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

pub use error::Result;
use fastjob_proto::fastjob::*;
use std::collections::HashMap;
use std::cmp::Ordering;

mod alarm_controller;
mod dispatch;
mod error;
mod instance_status_checker;
pub mod worker_manager;
mod event;

#[macro_use]
extern crate fastjob_components_log;

struct WorkerClusterHolder {
    app_name: String,
    // all worker in the cluster.
    workers: HashMap<String, Worker>,
    containers: HashMap<u64, HashMap<String, DeployContainerInfo>>,
}

struct Worker {
    address: String,
    last_active_time: i64,
    client: Option<grpcio::Client>,
    tag: String,
}

impl Eq for Worker {}

impl PartialEq for Worker {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

impl Ord for Worker {
    fn cmp(&self, other: &Self) -> Ordering {

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
            address: "".to_string(),
            last_active_time: 0,
            client: None,
            tag: "".to_string(),
        }
    }

    pub fn refresh(&self, heartbeat: &HeartBeatRequest) {}

    #[inline]
    pub fn last_active_time(&self) -> i64 {
        self.last_active_time
    }
}

impl WorkerClusterHolder {
    pub fn new(app_name: String) -> Self {
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
            let worker = Worker::new();
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
    pub fn get_suitable_worker(&self) -> Option<&[Worker]> {
        None
    }

    pub fn get_container_infos(&self, contain_id: u64) -> Option<&[DeployContainerInfo]> {
        if let Some(v) = self.containers.get(&contain_id).take() {
            // return Some(v.as_slice());
        }
        None
    }

    #[inline]
    fn app_name(&self) -> String {
        self.app_name.clone()
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
