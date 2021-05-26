use super::Result;
use crate::server;
use crossbeam::channel::Sender;
use fastjob_components_utils::id_generator::GeneratorTyp;
use fastjob_components_utils::{id_generator, signal_handler};
use fastjob_components_worker::worker_manager;
use fastjob_proto::fastjob::{
    WorkerManagerConfig, WorkerManagerScope, WorkerManagerScope::ServerSide,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use tokio::time::{self, Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub server: server::ServiceConfig,
}

pub struct App {
    pub server: server::Server,
}

impl Config {
    /// Only build all components equivalent to initialization, and will not start.
    pub fn build(self, shutdown_tx: Sender<()>) -> Result<App> {
        let server = server::Server::build(
            id_generator::generator_id(GeneratorTyp::Server),
            &self.server,
        );

        Ok(App {
            server
        })
    }
}

impl App {
    /// Run a FastJob server include scheduler 、workerManager 、gossip 、admin components etc.
    pub fn spawn(self) {
        let App {
            mut server
        } = self;

        server
            .run()
            .unwrap_or_else(|e| tracing::error!("FastJob Server start failure, cause: {}", e));

        // worker_manager.run().unwrap_or_else(|e| {
        //     tracing::error!("FastJob WorkerManager start failure, cause: {}", e)
        // });

        signal_handler::wait_for_signal();

        // std::thread::Builder::new()
        //     .name("fastjob-server".into())
        //     .spawn(move || {
        //         let rt = tokio::runtime::Builder::new_current_thread()
        //             .enable_all()
        //             .build()
        //             .expect("building failed");
        //         rt.block_on(async move {
        //             tokio::spawn(FastJobServe::serve().await.expect("a"));
        //         })
        //     }).expect("fastjob-server");

        // std::thread::Builder::new()
        //     .name("workermanger".into())
        //     .spawn(move || {});
    }

    pub fn stop(self) {}
}
