use tokio::runtime::{Builder, Runtime};

pub fn build() -> Runtime {
    let mut cores = std::env::var("FASTJOB_CORES")
        .ok()
        .and_then(|v| {
            let opt = v.parse::<usize>().ok().filter(|n| *n > 0);
            if opt.is_none() {
                warn!(FASTJOB_CORES = %v, "Ignoring invalid configuration");
            }
            opt
        })
        .unwrap_or(0);

    let cpus = num_cpus::get();
    debug_assert!(cpus > 0, "At least one CPU must be available");
    if cores > cpus {
        warn!(
            cpus,
            FASTJOB_CORES = cores,
            "Ignoring configuration due to insufficient resources"
        );
        cores = cpus;
    }

    match cores {
        0 | 1 => {
            info!("Using single-threaded runtime");
            Builder::new_current_thread()
                .enable_all()
                .thread_name("proxy")
                .build()
                .expect("failed to build basic runtime!")
        }
        num_cpus => {
            info!(%cores, "Using multi-threaded runtime");
            Builder::new_multi_thread()
                .enable_all()
                .thread_name("proxy")
                .worker_threads(num_cpus)
                .max_blocking_threads(num_cpus)
                .build()
                .expect("failed to build threaded runtime!")
        }
    }
}
