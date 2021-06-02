use std::sync::Arc;

use dashmap::DashMap;
use grpcio::{ChannelBuilder, EnvBuilder};

pub use error::Result;

mod error;
mod job_fetcher;
pub mod worker_manager;
mod sender;
mod dispatch;

#[macro_use]
extern crate fastjob_components_log;


struct Worker {
    inner: WorkerInner,
}

struct WorkerInner {
    client: ::grpcio::Client,
    status: WorkerStatus,
}

impl WorkerInner {
    fn new(client: ::grpcio::Client) -> Self {
        Self { client, status: WorkerStatus::new() }
    }
}

struct WorkerStatus {}

impl WorkerStatus {
    fn new() -> Self {
        Self {}
    }
}

impl Worker {
    pub fn build(address: &str) -> Self {
        let client = init_grpc_client(address)?;
        Self { inner: WorkerInner { client, status: WorkerStatus {} } }
    }

    pub fn transform_status(&self) {}
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
