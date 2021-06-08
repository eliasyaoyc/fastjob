use crate::model::job_info::JobType::{Java, Shell};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rbatis::crud::{CRUDTable, CRUD};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use crate::model::task::TimeExpressionType;

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum JobType {
    Java = 1,
    Shell = 2,
}

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum JobTimeExpressionType {
    API = 1,
    CRON = 2,
    FixRate = 3,
    FixDelay = 4,
    WORKFLOW = 5,
}

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum JobStatus {
    Running = 1,
    Stop = 2,
    DISABLED = 3,
    DELETED = 10,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JobInfo {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub concurrency: Option<u32>,
    /// Specifies the machine to run, empty represents unlimited,
    /// non-empty will only use one of the machines to run (multi-value comma split)
    pub designated_workers: Option<&'static str>,
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
    pub job_description: Option<&'static str>,
    pub job_name: Option<&'static str>,
    pub job_params: Option<&'static str>,
    pub lifecycle: Option<&'static str>,
    pub max_instance_num: Option<usize>,
    /// The maximum worker numbers, just scope for the execution of mapreduce.
    pub max_worker_count: Option<usize>,
    /// The minimum cpu numbers, 0 represent unlimited.
    pub min_cpu_cores: Option<f64>,
    /// Minimum disk space, unit GB, 0 represents unlimited
    pub min_disk_space: Option<f64>,
    /// Minimum memory space, unit GB, 0 represents unlimited
    pub min_memory_space: Option<f64>,
    pub next_trigger_time: Option<i64>,
    /// Alarm list of user ids, multi-valued comma-separated.
    pub notify_user_ids: Option<&'static str>,
    pub processor_info: Option<&'static str>,
    /// The process type, Java/Shell.
    pub processor_type: Option<usize>,
    /// 1 normal running，2 stop
    pub status: Option<usize>,
    pub task_retry_num: Option<usize>,
    /// Time expression CRON/NULL/LONG/LONG
    pub time_expression: Option<&'static str>,
    /// Time expression type（CRON/API/FIX_RATE/FIX_DELAY）
    pub time_expression_type: Option<usize>,
}

impl JobInfo {
    #[inline]
    pub fn get_id(&self) -> Option<u64> {
        self.id.clone()
    }

    #[inline]
    pub fn get_app_id(&self) -> Option<u64> {
        self.app_id.clone()
    }

    #[inline]
    pub fn get_job_params(&self) -> Option<&str> {
        self.job_params.clone()
    }

    #[inline]
    pub fn get_next_trigger_time(&self) -> Option<i64> {
        self.next_trigger_time.clone()
    }
}

impl JobTimeExpressionType {
    #[inline]
    pub fn is_frequent(&self) -> bool {
        self == JobTimeExpressionType::FixRate || self == JobTimeExpressionType::FixDelay
    }
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
