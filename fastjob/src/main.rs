mod rt;

use fastjob_components_app::{Config, option::Opt};
use fastjob_components_utils::signal;
use tokio::sync::mpsc;
pub use tracing::{debug, error, info, warn};

const EX_USAGE: i32 = 64;

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
        let app = match config.build(shutdown_rx).await {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Initialization failure: {}", e);
                std::process::exit(1);
            }
        };

        let drain = app.spawn();
        tokio::select! {
            _ = signal::shutdown() => {
                info!("Received shutdown signal")
            }
            _ = shutdown_rx.recv() => {
                info!("Received shutdown via admin interface.")
            }
        }
        drain.drain().await;
    });
}

fn config_from_yaml(opt: Opt) -> Result<Config, ()> {
    Ok(Config { worker: 0, gossip: () })
}
