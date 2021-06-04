use snafu::{ResultExt, Snafu};

pub type Result<T, E = SchedError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum SchedError {
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

    #[snafu(display("Scheduler {} is too busy.", sched_id))]
    SchedTooBusy { sched_id: u64 },

    #[snafu(display("Constructor task id: {} encounter error: {}", task_id, source))]
    ConstructorTaskFailed {
        source: delay_timer::error::TaskError,
        task_id: u64,
    },
}
