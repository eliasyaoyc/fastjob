use rbatis::crud::{CRUDTable, CRUD};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u32)]
pub enum InstanceStatus {
    /// Waiting for dispatch.
    WaitingDispatch = 1,
    /// Waiting work receive.
    WaitingWorkerReceive = 2,
    Running = 3,
    Failed = 4,
    Success = 5,
    Canceled = 9,
    Stopped = 10,
}

#[derive(TryFromPrimitive)]
#[repr(u32)]
pub enum InstanceType {
    Normal = 1,
    WorkFlow = 2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstanceInfo {
    pub id: Option<u64>,
    pub job_id: Option<u64>,
    pub app_id: Option<u64>,
    pub instance_id: Option<u64>,
    pub job_params: Option<String>,
    pub instance_params: Option<String>,
    pub instance_type: Option<u32>,
    pub wf_instance_id: Option<u64>,
    pub status: Option<u32>,
    pub result: Option<String>,
    pub expected_trigger_time: Option<u64>,
    pub actual_trigger_time: Option<u64>,
    pub finished_time: Option<u64>,
    pub last_report_time: Option<u64>,
    pub task_tracker_address: Option<u64>,
    pub running_times: Option<u64>,
    pub gmt_create: Option<u64>,
    pub gmt_modified: Option<u64>,
}

impl CRUDTable for InstanceInfo {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "instance_info".to_string()
    }
}

