use std::fmt::{Debug, Formatter};
use fastjob_components_storage::model::workflow_info::WorkflowInfo;

pub struct WorkerFlowManager {}

impl Debug for WorkerFlowManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}


impl WorkerFlowManager {
    pub fn new(workflow_info: WorkflowInfo, int_params: &str, expect_trigger_time: u64) -> Self {
        Self {}
    }

    pub fn init_node(&self) {}
}