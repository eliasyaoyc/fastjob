use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

pub use error::Result;
use std::collections::HashMap;

mod alarm_controller;
mod dispatch;
mod error;
mod instance_status_checker;
mod task;
pub mod worker_manager;

#[macro_use]
extern crate fastjob_components_log;

struct WorkerClusterHolder {
    app_name: String,
    // all worker in the cluster.
    workers: HashMap<String, Worker>,
}

struct Worker {
    address: String,
    last_active_time: i64,
    client: Option<grpcio::Client>,
    tag: String,
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

    pub fn refresh(&self) {}
}

impl WorkerClusterHolder {
    pub fn new(app_name: String) -> Self {
        Self {
            app_name,
            workers: Default::default(),
        }
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
