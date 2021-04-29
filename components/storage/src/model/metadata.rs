use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use rbatis::crud::CRUDTable;
use std::fmt::Display;
use rbatis::utils::string_util::to_snake_name;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub id: Option<String>,
}

impl CRUDTable for Metadata {
    type IdType = String;

    fn get_id(&self) -> Option<&Self::IdType> {
        self.id.as_ref()
    }

    fn table_name() -> String {
        "metadata".to_string()
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            id: None
        }
    }
}

