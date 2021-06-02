use serde::Deserialize;
use serde::Serialize;
use rbatis::crud::CRUDTable;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppInfo {
    pub id: Option<u64>,
    pub app_name: Option<String>,
    pub password: Option<String>,
    pub current_server: Option<String>,
    pub gmt_create: Option<i64>,
    pub gmt_modified: Option<i64>,
}


impl CRUDTable for AppInfo {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "app_info".to_string()
    }
}
