use num_enum::{IntoPrimitive, TryFromPrimitive};
use rbatis::crud::{CRUDTable, CRUD};
use serde::Deserialize;
use serde::Serialize;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Debug, Display, Formatter};

#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Hash)]
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

impl Debug for InstanceStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::WaitingDispatch => {
                write!(f, "等待派发")
            }
            InstanceStatus::WaitingWorkerReceive => {
                write!(f, "等待 Worker 接受")
            }
            InstanceStatus::Running => {
                write!(f, "运行中")
            }
            InstanceStatus::Failed => {
                write!(f, "失败")
            }
            InstanceStatus::Success => {
                write!(f, "成功")
            }
            InstanceStatus::Canceled => {
                write!(f, "取消")
            }
            InstanceStatus::Stopped => {
                write!(f, "手动停止")
            }
        }
    }
}

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum InstanceType {
    Normal = 1,
    WorkFlow = 2,
}

impl Debug for InstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceType::Normal => {
                write!(f, "normal instance.")
            }
            InstanceType::WorkFlow => {
                write!(f, "workflow instance.")
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstanceInfo {
    pub id: Option<u64>,
    pub job_id: Option<u64>,
    pub app_id: Option<u64>,
    pub instance_id: Option<u64>,
    pub job_params: Option<&'static str>,
    pub instance_params: Option<&'static str>,
    pub instance_type: Option<u32>,
    pub wf_instance_id: Option<u64>,
    pub status: Option<u32>,
    pub result: Option<&'static str>,
    pub expected_trigger_time: Option<i64>,
    pub actual_trigger_time: Option<i64>,
    pub finished_time: Option<i64>,
    pub last_report_time: Option<i64>,
    pub task_tracker_address: Option<&'static str>,
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
        job_params: Option<&'static str>,
        instance_params: Option<&'static str>,
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
            gmt_create: None,
            gmt_modified: None,
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
        ].to_vec()
    }

    #[inline]
    pub fn finish_status() -> Vec<u32> {
        vec![
            InstanceStatus::Failed.into(),
            InstanceStatus::Success.into(),
            InstanceStatus::Canceled.into(),
            InstanceStatus::Stopped.into(),
        ].to_vec()
    }

    #[inline]
    pub fn as_ref(&self) -> &InstanceInfo {
        self.borrow()
    }

    #[inline]
    pub fn as_mut_ref(&mut self) -> &mut InstanceInfo {
        self.borrow_mut()
    }
}
