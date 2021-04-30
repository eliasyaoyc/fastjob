use rbatis::crud::CRUDTable;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub id: Option<u64>,
}

impl CRUDTable for Metadata {
    type IdType = u64;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "metadata".to_string()
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self { id: None }
    }
}
