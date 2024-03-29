pub use self::imp::wait_for_signal;

#[cfg(unix)]
mod imp {
    use libc::c_int;
    use nix::sys::signal::{SIGHUP, SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
    use signal::trap::Trap;

    #[allow(dead_code)]
    pub fn wait_for_signal() {
        let trap = Trap::trap(&[SIGTERM, SIGINT, SIGHUP, SIGUSR1, SIGUSR2]);
        for sig in trap {
            match sig {
                SIGTERM | SIGINT | SIGHUP => {
                    tracing::info!("receive signal {}, stopping server...", sig as c_int);
                    break;
                }
                // SIGUSR1 => {
                //     // Use SIGUSR1 to log metrics.
                //     info!("{}", metrics::dump());
                //     if let Some(ref engines) = engines {
                //         info!("{:?}", MiscExt::dump_stats(&engines.kv));
                //         info!("{:?}", RaftEngine::dump_stats(&engines.raft));
                //     }
                // }
                // TODO: handle more signal
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use engine_rocks::RocksEngine;
    use engine_traits::Engines;

    pub fn wait_for_signal(_: Option<Engines<RocksEngine, RocksEngine>>) {}
}
