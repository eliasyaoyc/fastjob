use rbatis::crud::CRUDTable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lock {
    pub id: Option<u64>,
    pub lock_name: Option<&'static str>,
    pub max_lock_time: Option<u64>,
    pub owner_ip: Option<&'static str>,
    pub gmt_create: Option<i64>,
    pub gmt_modified: Option<i64>,
}

impl CRUDTable for Lock {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "lock".to_string()
    }
}

impl Lock {
    pub fn new(lock_name: &str, max_lock_time: u64, owner_ip: &str) -> Self {
        Self {
            id: None,
            lock_name: Some(lock_name),
            max_lock_time: Some(max_lock_time),
            owner_ip: Some(owner_ip),
            gmt_create: None,
            gmt_modified: None,
        }
    }
}
