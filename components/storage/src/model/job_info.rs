use rbatis::crud::{CRUDTable, CRUD};
use rbatis::utils::string_util::to_snake_name;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JobInfo {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub concurrency: Option<u32>,
    pub designated_workers: Option<String>,
    pub dispatch_strategy: Option<u32>,
    pub execute_type: Option<u32>,
    pub extra: Option<String>,
    pub gmt_create: Option<u64>,
    pub gmt_modified: Option<u64>,
    pub instance_retry_num: Option<usize>,
    pub instance_time_limit: Option<u64>,
    pub job_description: Option<String>,
    pub job_name: Option<String>,
    pub job_params: Option<String>,
    pub lifecycle: Option<String>,
    pub max_instance_num: Option<usize>,
    pub max_worker_count: Option<usize>,
    pub min_cpu_cores: Option<f64>,
    pub min_disk_space: Option<f64>,
    pub min_memory_space: Option<f64>,
    pub next_trigger_time: Option<u64>,
    pub notify_user_ids: Option<String>,
    pub processor_info: Option<String>,
    pub processor_type: Option<usize>,
    pub status: Option<usize>,
    pub task_retry_num: Option<usize>,
    pub time_expression: Option<String>,
    pub time_expression_type: Option<usize>,
}


impl CRUDTable for JobInfo {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "job_info".to_string()
    }
}

