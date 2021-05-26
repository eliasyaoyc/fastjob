use crate::server::ServiceConfig;
use chrono::Local;
use fastjob_components_log::{
    crit, file_writer, init_log, json_format, term_writer, text_format, LogDispatcher, LogFormat,
    DATETIME_ROTATE_SUFFIX,
};
use fastjob_components_utils::time::duration_to_ms;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

// A workaround for checking if log is initialized.
pub static LOG_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[macro_export]
macro_rules! fatal {
    ($lvl:expr $(, $arg:expr)*) => ({
        if $crate::log::LOG_INITIALIZED.load(::std::sync::atomic::Ordering::SeqCst) {
            crit!($lvl $(, $arg)*);
        } else {
            eprintln!($lvl $(, $arg)*);
        }
        slog_global::clear_global();
        ::std::process::exit(1)
    })
}

pub fn initial_logger(config: &ServiceConfig) {
    if config.log_file.is_empty() {
        let writer = term_writer();
        match config.log_format {
            LogFormat::Text => build_logger(text_format(writer), config),
            LogFormat::Json => build_logger(json_format(writer), config),
        };
    } else {
        let writer = file_writer(
            &config.log_file,
            config.log_rotation_timespan,
            config.log_rotation_size,
            rename_by_timestamp,
        )
        .unwrap_or_else(|e| {
            fatal!(
                "failed to initialize log with file {}: {}",
                config.log_file,
                e
            );
        });

        let slow_log_writer = if config.slow_log_file.is_empty() {
            None
        } else {
            let slow_log_writer = file_writer(
                &config.slow_log_file,
                config.log_rotation_timespan,
                config.log_rotation_size,
                rename_by_timestamp,
            )
            .unwrap_or_else(|e| {
                fatal!(
                    "failed to initialize slow-log with file {}: {}",
                    config.slow_log_file,
                    e
                );
            });
            Some(slow_log_writer)
        };

        match config.log_format {
            LogFormat::Text => build_logger_with_slow_log(
                text_format(writer),
                slow_log_writer.map(text_format),
                config,
            ),
            LogFormat::Json => build_logger_with_slow_log(
                json_format(writer),
                slow_log_writer.map(json_format),
                config,
            ),
        };
    }

    LOG_INITIALIZED.store(true, Ordering::SeqCst);
}

fn build_logger<D>(drainer: D, config: &ServiceConfig)
where
    D: slog::Drain + Send + 'static,
    <D as slog::Drain>::Err: std::fmt::Display,
{
    // use async drainer and init std log.
    init_log(
        drainer,
        config.log_level,
        true,
        true,
        vec![],
        duration_to_ms(config.slow_log_threshold),
    )
    .unwrap_or_else(|e| {
        fatal!("failed to initialize log: {}", e);
    });
}

fn build_logger_with_slow_log<N, S>(normal: N, slow: Option<S>, config: &ServiceConfig)
where
    N: slog::Drain<Ok = (), Err = std::io::Error> + Send + 'static,
    S: slog::Drain<Ok = (), Err = std::io::Error> + Send + 'static,
{
    let drainer = LogDispatcher::new(normal, slow);

    init_log(
        drainer,
        config.log_level,
        true,
        true,
        vec![],
        duration_to_ms(config.slow_log_threshold),
    )
    .unwrap_or_else(|e| {
        fatal!("failed to initialize log: {}", e);
    });
}

/// a lot of logs written in a very short time. Consider rename the rotated file with a version
/// number while rotate by size.
fn rename_by_timestamp(path: &Path) -> std::io::Result<PathBuf> {
    let mut new_path = path.to_path_buf().into_os_string();
    new_path.push(format!(".{}", Local::now().format(DATETIME_ROTATE_SUFFIX)));
    Ok(PathBuf::from(new_path))
}
