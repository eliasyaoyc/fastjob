pub use error::Result;
use std::sync::Arc;
use grpcio::{EnvBuilder, ChannelBuilder};

mod error;
mod job_fetcher;
pub mod worker_manager;
mod health_checker;
mod sender;

pub fn init_grpc_client(addr: &str) -> Result<::grpcio::Client> {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(addr);
    let client = FastJobClient::new(ch);
    Ok(client)
    // let mut req = HelloRequest::default();
    // req.set_name("world".to_owned());
    // let reply = client.say_hello(&req).expect("rpc");
    // info!("Greeter received: {}", reply.get_message());
}
