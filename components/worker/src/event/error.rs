use snafu::{ResultExt, Snafu};

pub type Result<T, E = EventHandlerError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum EventHandlerError {
    #[snafu(display("WorkerManager storage error."))]
    WorkerStorageError,

    #[snafu(display(
    "App name or id {} is not registered, please register the app first.",
    app_name_or_id
    ))]
    WorkerNotRegistered { app_name_or_id: String },

    #[snafu(display("server {} lookup failed", server_ip))]
    LookupFail { server_ip: String },
}
