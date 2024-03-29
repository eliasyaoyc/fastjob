use rbatis::crud::CRUDTable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerInfo {
    pub id: Option<u64>,
    pub ip: Option<&'static str>,
    pub gmt_create: Option<u64>,
    pub gmt_modified: Option<u64>,
}

impl CRUDTable for ServerInfo {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "server_info".to_string()
    }
}
