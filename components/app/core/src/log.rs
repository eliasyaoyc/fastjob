use crate::server::ServiceConfig;

pub fn initial_logger(config: &ServiceConfig) {}

fn build_logger<D>(drainer: D, config: &ServiceConfig)
    where
        D: slog::Drain + Send + 'static,
        <D as slog::Drain>::Err: std::fmt::Display,
{
}
