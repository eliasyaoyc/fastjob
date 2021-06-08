use rbatis::crud::CRUDTable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppInfo {
    pub id: Option<u64>,
    pub app_name: Option<&'static str>,
    pub password: Option<&'static str>,
    pub current_server: Option<&'static str>,
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
