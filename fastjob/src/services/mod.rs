pub use self::job_service::Service as FastJobService;

mod admin_service;
mod job_service;
mod worker_status_service;
mod ha_service;
mod instance_service;
mod ping_pong_service;

pub const GRPC_RESPONSE_CODE: u64 = 200;

