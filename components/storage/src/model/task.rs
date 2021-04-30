use rbatis::crud::{CRUDTable, CRUD};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use rbatis::utils::string_util::to_snake_name;

enum TimeExpressionType {
    CRON,
}

enum ExecuteType {
    JAR,
    URL,
}

enum TaskStatus {
    INIT,
    READY,
    RUNNING,
    COMPLETED,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub task_id: Option<u64>,
    pub task_name: Option<String>,
    pub task_desc: Option<String>,
    pub app_id: Option<u64>,
    pub task_manager_id: Option<u64>,
    pub task_param: Option<String>,
    pub time_expression_type: Option<TimeExpressionType>,
    pub time_expression: Option<String>,
    pub max_instance_num: Option<u32>,
    pub concurrency: Option<u32>,
    pub instance_time_limit: Option<u64>,
    pub instance_retry_time: Option<u64>,
    pub task_retry_num: Option<u32>,
    pub task_status: Option<TaskStatus>,
    pub next_trigger_time: Option<u64>,
    pub max_cpu_core: Option<u32>,
    pub max_memory_space: Option<f64>,
    pub max_disk_space: Option<f64>,
    pub max_worker_count: Option<u32>,
    pub create_time: Option<u64>,
}

unsafe impl Send for Task {}

unsafe impl Sync for Task {}

impl CRUDTable for Task {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.task_id.as_ref()
    }

    fn table_name() -> String {
        "task".to_string()
    }
}