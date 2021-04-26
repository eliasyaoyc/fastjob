pub mod option;

use crate::option::Opt;
use fastjob_components_core::gossip::GossipConfig;
use fastjob_components_core::{gossip, service, ListenAddr};
use fastjob_components_error::Error;
use fastjob_components_utils::id_generator::GeneratorTyp;
use fastjob_components_utils::{drain, id_generator};
use fastjob_components_worker::worker_manager;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use tokio::{
    sync::mpsc,
    time::{self, Duration},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub server: service::ServiceConfig,
    pub worker_manager: worker_manager::WorkerManagerConfig,
}

pub struct App {
    pub server: service::FastJobServe,
    pub worker_manager: worker_manager::WorkerManager,
}

impl Config {
    /// Only build all components equivalent to initialization, and will not start.
    pub async fn build(self, shutdown_tx: mpsc::UnboundedSender<()>) -> Result<App, Error> {
        let server = service::FastJobServe::build(
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
    /// start all components.
    pub fn spawn(self){
        let App {
            server,
            worker_manager,
        } = self;

        std::thread::Builder::new()
            .name("fastjob-server".into())
            .spawn(move || {
                // let rt = tokio::runtime::Builder::new_current_thread()
                //     .enable_all()
                //     .build()
                //     .expect("building failed");
                // rt.block_on(async move {
                //     tokio::spawn(server.serve());
                // })
            }).expect("fastjob-server");

        std::thread::Builder::new()
            .name("workermanger".into())
            .spawn(move || {});
    }
}
