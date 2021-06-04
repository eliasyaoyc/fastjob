use snafu::{ResultExt, Snafu};

pub type Result<T, E = AlarmError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum AlarmError {
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

    #[snafu(display("Not match scheduler."))]
    NotMatch {},

    #[snafu(display("scheduler {} is too busy.", sched_id))]
    SchedTooBusy { sched_id: u64 },
}