pub mod option;

use crate::option::Opt;
use fastjob_components_core::gossip::GossipConfig;
use fastjob_components_core::{gossip, server, ListenAddr};
use fastjob_components_error::Error;
use fastjob_components_utils::id_generator::GeneratorTyp;
use fastjob_components_utils::{drain, id_generator, signal_handler};
use fastjob_components_worker::worker_manager;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use tokio::{
    sync::mpsc,
    time::{self, Duration},
};
use fastjob_components_core::server::Server;

#[derive(Clone, Debug)]
pub struct Config {
    pub server: server::ServiceConfig,
    pub worker_manager: worker_manager::WorkerManagerConfig,
}

pub struct App {
    pub server: server::Server,
    pub worker_manager: worker_manager::WorkerManager,
}

impl Config {
    /// Only build all components equivalent to initialization, and will not start.
    pub fn build(self, shutdown_tx: mpsc::UnboundedSender<()>) -> Result<App, Error> {
        let server = server::Server::build(
            id_generator::generator_id(GeneratorTyp::Server),
            &self.server,
        );

        let worker_manager = worker_manager::WorkerManager::build(
            id_generator::generator_id(GeneratorTyp::WorkerManager),
            &self.worker_manager,
        );

        Ok(App {
            server,
            worker_manager,
        })
    }
}

impl App {
    /// Run a FastJob server include scheduler 、workerManager 、gossip 、admin components etc.
    pub fn spawn(self) {
        let App {
            mut server,
            mut worker_manager,
        } = self;

        // setup the global logger.
        initial_logger();

        server.run().unwrap_or_else(|e| tracing::error!("FastJob Server start failure, cause: {}", e));
        worker_manager.run().unwrap_or_else(|e| tracing::error!("FastJob WorkerManager start failure, cause: {}", e));

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

#[allow(dead_code)]
pub fn initial_logger() {}