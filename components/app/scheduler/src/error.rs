use snafu::{ResultExt, Snafu};

pub type Result<T, E = SchedError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum SchedError {
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