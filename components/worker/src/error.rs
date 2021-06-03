use snafu::{ResultExt, Snafu};

pub type Result<T, E = WorkerError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum WorkerError {
    #[sanfu(display("WorkerManager storage error."))]
    WorkerStorageError,

    #[sanfu(display("App name or id {} is not registered, please register the app first.", app_name))]
    WorkerNotRegistered {
        app_name_or_id: String,
    },

    #[sanfu(display("server {} lookup failed", server_ip))]
    LookupFail {
        server_ip: String,
    },
}