use snafu::{ResultExt, Snafu};

pub type Result<T, E = WorkerError> = std::result::Result<T, E>;

#[derive(Snafu)]
enum WorkerError {}
