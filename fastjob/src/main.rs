use fastjob::Result;
use fastjob::{server, Config};
use fastjob_components_log::{get_level_by_string, LogFormat};
use fastjob_components_storage::StorageConfig;
use fastjob_components_utils::signal;
use fastjob_proto::fastjob::WorkerManagerConfig;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use structopt::StructOpt;
use tracing::{debug, error, info, warn};

const EX_USAGE: i32 = 64;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short = "d", long)]
    debug: bool,
    #[structopt(short = "addr", default_value = "127.0.0.1:3000")]
    addr: String,
    #[structopt(short = "gp", default_value = "3001")]
    gossip_addr: u16,
    #[structopt(short = "ll", default_value = "info")]
    log_level: String,
    #[structopt(short = "cp", default_value = "../fastjob.toml")]
    config_path: String,
}

fn main() {
    // tracing_subscriber::fmt::init();

    // Command-Line Highest Priority.
    let opt = Opt::from_args();
    println!("recv command-line param {:#?}", opt);

    let config = match overwrite_config_with_cmd_args(opt) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Invalid configuration: {}", e);
            std::process::exit(EX_USAGE);
        }
    };

    let (shutdown_tx, mut shutdown_rx) = crossbeam::channel::unbounded();
    let app = match config.build(shutdown_tx) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Initialization failure: {}", e);
            std::process::exit(1);
        }
    };

    // Run server.
    app.spawn();

    // rt::build().block_on(async move {
    //     let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
    //     let app = match config.build(shutdown_tx).await {
    //         Ok(app) => app,
    //         Err(e) => {
    //             eprintln!("Initialization failure: {}", e);
    //             std::process::exit(1);
    //         }
    //     };
    //
    //     // Run server.
    //     app.spawn();
    //
    //     tokio::select! {
    //     _ = signal::shutdown() => {
    //         info!("Received shutdown signal")
    //     }
    //     _ = shutdown_rx.recv() => {
    //         info!("Received shutdown via admin interface.")
    //     }
    // }
    //     // drain.drain().await;
    // });
}

pub fn overwrite_config_with_cmd_args(opt: Opt) -> Result<Config> {
    let config = StorageConfig {
        addr: "localhost:3306".to_string(),
        username: "root".to_string(),
        password: "yaoyichen52".to_string(),
        database: "neptune".to_string(),
        max_connections: 20,
        min_connections: 5,
        connect_timeout: 5,
        idle_timeout: 5,
    };

    Ok(Config {
        server: server::ServiceConfig {
            addr: opt.addr,
            log_level: get_level_by_string(&opt.log_level).unwrap(),
            log_file: "".to_string(),
            log_format: LogFormat::Text,
            slow_log_file: "".to_string(),
            slow_log_threshold: Duration::from_secs(1),
            log_rotation_timespan: Duration::from_secs(86400),
            storage_config: config,
            log_rotation_size: 300,
        },
    })
}
