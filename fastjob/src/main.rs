mod rt;

use fastjob_components_app::{Config, option};
use fastjob_components_utils::signal;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use structopt::StructOpt;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use fastjob_components_core::{service, ListenAddr};
use fastjob_components_core::gossip::GossipConfig;
use fastjob_components_worker::worker_manager;
use std::io::Error;

const EX_USAGE: i32 = 64;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short = "d", long)]
    debug: bool,
    #[structopt(short = "p", default_value = "3000")]
    serve_port: u16,
    #[structopt(short = "gp", default_value = "3001")]
    gossip_port: u16,
    #[structopt(short = "ll", default_value = "info")]
    log_level: String,
}

fn main() {
    // Command-Line Highest Priority.
    let opt = Opt::from_args();
    tracing::debug!("recv command-line param {:#?}", opt);

    let config = match config_from_yaml(opt) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Invalid configuration: {}", e);
            std::process::exit(EX_USAGE);
        }
    };

    rt::build().block_on(async move {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let app = match config.build(shutdown_tx).await {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Initialization failure: {}", e);
                std::process::exit(1);
            }
        };

        app.spawn();

        tokio::select! {
            _ = signal::shutdown() => {
                info!("Received shutdown signal")
            }
            _ = shutdown_rx.recv() => {
                info!("Received shutdown via admin interface.")
            }
        }
        // drain.drain().await;
    });
}

pub fn config_from_yaml(opt: Opt) -> Result<Config, Error> {
    Ok(Config {
        server: service::ServiceConfig {
            addr: ListenAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)),
            gossip: GossipConfig {},
            log_level: "".to_string(),
        },
        worker_manager: worker_manager::WorkerManagerConfig {},
    })
}