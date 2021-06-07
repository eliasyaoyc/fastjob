use rbatis::crud::CRUDTable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkflowInfo {
}


impl CRUDTable for WorkflowInfo {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "workflow_info".to_string()
    }
}

