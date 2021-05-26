use snafu::{ResultExt, Snafu};
use std::fmt::{Display, Formatter};

pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[derive(Snafu)]
enum AppError {
    #[snafu(display("Unable to read configuration from {}: {}", path.display(), source))]
    ReadConfiguration {
        source: std::io::Error,
        path: std::path::PathBuf,
    },
    #[snafu(display("Unable to write result to {}: {}", path.display(), source))]
    WriteResult {
        source: std::io::Error,
        path: std::path::PathBuf,
    },
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
