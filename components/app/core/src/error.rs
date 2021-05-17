use fastjob_components_error::Error;
use snafu::{ResultExt, Snafu};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum AppError {
    #[snafu(display("Unable to read configuration from {}: {}", path.display(), source))]
    ReadConfiguration { source: std::io::Error, path: std::path::PathBuf },
    #[snafu(display("Unable to write result to {}: {}", path.display(), source))]
    WriteResult { source: std::io::Error, path: std::path::PathBuf },
}
