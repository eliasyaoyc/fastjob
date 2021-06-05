use rbatis::crud::CRUDTable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: Option<u64>,
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub web_hook: Option<String>,
    pub extra: Option<String>,
    pub gmt_create: Option<i64>,
    pub gmt_modified: Option<i64>,
}

impl CRUDTable for UserInfo {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "user_info".to_string()
    }
}

impl UserInfo {}
