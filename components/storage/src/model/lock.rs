use serde::Deserialize;
use serde::Serialize;
use rbatis::crud::CRUDTable;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lock {
    pub id: Option<u64>,
    pub lock_name: Option<String>,
    pub max_lock_time: Option<u64>,
    pub owner_ip: Option<String>,
    pub gmt_create: Option<u64>,
    pub gmt_modified: Option<u64>,
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
    pub fn new(
        lock_name: String,
        max_lock_time: u64,
        owner_ip: String,
    ) -> Self {
        Self {
            id: None,
            lock_name: Some(lock_name),
            max_lock_time: Some(max_lock_time),
            owner_ip: Some(owner_ip),
            gmt_create: None,
            gmt_modified: None,
        }
    }

    pub fn lock(&self) -> bool {
        true
    }

    pub fn release(&self) {}
}