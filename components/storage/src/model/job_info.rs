use rbatis::crud::{CRUDTable, CRUD};
use rbatis::utils::string_util::to_snake_name;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use crate::model::job_info::JobType::{Java, Shell};

#[derive(TryFromPrimitive)]
#[repr(usize)]
pub enum JobType {
    Java = 1,
    Shell = 2,
}

#[derive(TryFromPrimitive)]
#[repr(usize)]
pub enum JobTimeExpressionType {
    CRON = 1,
    API = 2,
    FixRate = 3,
    FixDelay = 4,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JobInfo {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub concurrency: Option<u32>,
    /// Specifies the machine to run, empty represents unlimited,
    /// non-empty will only use one of the machines to run (multi-value comma split)
    pub designated_workers: Option<String>,
    pub dispatch_strategy: Option<u32>,
    /// The execute type Standalone/Broadcast/MapReduce
    pub execute_type: Option<u32>,
    /// Extension parameters. This data is not used by FastJob itself and is reserved for developers to use when extending
    ///  For example, the custom Worker filtering logic of WorkerFilter can be passed in the filter metric GPUUsage < 10
    pub extra: Option<String>,
    pub gmt_create: Option<u64>,
    pub gmt_modified: Option<u64>,
    pub instance_retry_num: Option<usize>,
    /// The overall timeout for the task
    pub instance_time_limit: Option<u64>,
    pub job_description: Option<String>,
    pub job_name: Option<String>,
    pub job_params: Option<String>,
    pub lifecycle: Option<String>,
    /// deprecated
    pub max_instance_num: Option<usize>,
    /// The maximum worker numbers, just scope for the execution of mapreduce.
    pub max_worker_count: Option<usize>,
    /// The minimum cpu numbers, 0 represent unlimited.
    pub min_cpu_cores: Option<f64>,
    /// Minimum disk space, unit GB, 0 represents unlimited
    pub min_disk_space: Option<f64>,
    /// Minimum memory space, unit GB, 0 represents unlimited
    pub min_memory_space: Option<f64>,
    pub next_trigger_time: Option<u64>,
    /// Alarm list of user ids, multi-valued comma-separated.
    pub notify_user_ids: Option<String>,
    pub processor_info: Option<String>,
    /// The process type, Java/Shell.
    pub processor_type: Option<usize>,
    /// 1 normal running，2 stop
    pub status: Option<usize>,
    pub task_retry_num: Option<usize>,
    /// Time expression CRON/NULL/LONG/LONG
    pub time_expression: Option<String>,
    /// Time expression type（CRON/API/FIX_RATE/FIX_DELAY）
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

