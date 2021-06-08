use fastjob_components_scheduler::error::SchedError;
use crate::event::error::EventHandlerError;
use fastjob_components_storage::error::StorageError;
use snafu::{ResultExt, Snafu};

pub type Result<T, E = WorkerManagerError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum WorkerManagerError {
    #[snafu(display("WorkerManager storage encounter error: {}.", source))]
    WorkerStorageError { source: StorageError },

    #[snafu(display(
    "App name or id {} is not registered, please register the app first.",
    app_name_or_id
    ))]
    WorkerNotRegistered { app_name_or_id: &'static str },

    #[snafu(display("server {} lookup failed", server_ip))]
    LookupFail { server_ip: &'static str },

    #[snafu(display("WorkerManager scheduler encounter error: {}.", source))]
    SchedulerFailed { source: SchedError },

    #[snafu(display("WorkerManager event handle encounter error: {}.", source))]
    EventHandlerFailed { source: EventHandlerError },

    #[snafu(display("Permission Denied"))]
    PermissionDenied,
}
