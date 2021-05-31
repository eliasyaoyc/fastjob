mod admin_service;
mod job_service;
mod job_service_client;

pub use self::job_service::Service as FastJobService;
pub use self::job_service_client::Client as FastJobClient;
