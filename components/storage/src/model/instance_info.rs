use num_enum::{IntoPrimitive, TryFromPrimitive};
use rbatis::crud::{CRUDTable, CRUD};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(TryFromPrimitive, IntoPrimitive)]
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

#[derive(TryFromPrimitive, IntoPrimitive)]
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
    pub expected_trigger_time: Option<i64>,
    pub actual_trigger_time: Option<u64>,
    pub finished_time: Option<u64>,
    pub last_report_time: Option<u64>,
    pub task_tracker_address: Option<u64>,
    pub running_times: Option<u64>,
    pub gmt_create: Option<i64>,
    pub gmt_modified: Option<i64>,
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

impl InstanceInfo {
    pub fn create(
        job_id: Option<u64>,
        app_id: Option<u64>,
        job_params: Option<String>,
        instance_params: Option<String>,
        wf_instance_id: Option<u64>,
        expected_trigger_time: Option<i64>,
    ) -> InstanceInfo {
        let instance_id = 1;
        let instance_type = if wf_instance_id.is_none() {
            Some(InstanceType::Normal.into())
        } else {
            Some(InstanceType::WorkFlow.into())
        };
        InstanceInfo {
            id: None,
            job_id,
            app_id,
            instance_id: Some(instance_id),
            job_params,
            instance_params,
            instance_type,
            wf_instance_id,
            status: Some(InstanceStatus::WaitingDispatch.into()),
            result: None,
            expected_trigger_time,
            actual_trigger_time: None,
            finished_time: None,
            last_report_time: Some(-1),
            task_tracker_address: None,
            running_times: Some(0),
            gmt_create: Some(chrono::Local::now().timestamp()),
            gmt_modified: Some(chrono::Local::now().timestamp()),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id.unwrap_or(0)
    }

    #[inline]
    pub fn generalized_running_status() -> Vec<u32> {
        vec![
            InstanceStatus::WaitingDispatch.into(),
            InstanceStatus::WaitingWorkerReceive.into(),
            InstanceStatus::Running.into(),
        ]
        .to_vec()
    }
}
