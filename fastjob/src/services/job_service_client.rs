use grpcio::{ChannelBuilder, EnvBuilder};
use std::sync::Arc;

/// Client send the RPC messages to the 'Worker'.
#[derive(Clone)]
pub struct Client {
    client: ClientInner,
}

#[derive(Clone)]
struct ClientInner {
    inner: dashmap::DashMap<String, ::grpcio::Client>,
}

impl ClientInner {

}

impl client {
    pub fn start(&self) {
        // let env = Arc::new(EnvBuilder::new().build());
        // let ch = ChannelBuilder::new(env).connect("localhost:50051");
        // let client = FastJobClient::new(ch);
        //
        // let mut req = HelloRequest::default();
        // req.set_name("world".to_owned());
        // let reply = client.say_hello(&req).expect("rpc");
        // info!("Greeter received: {}", reply.get_message());
    }
    pub fn stop(&self) {}
}
