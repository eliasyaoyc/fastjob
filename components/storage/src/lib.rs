use fastjob_components_utils::component::Component;
use rbatis::core::db::{DBExecResult, DBPoolOptions};
use rbatis::crud::{CRUDTable, CRUD};
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::{Rbatis, RbatisOption};
use rbatis::wrapper::Wrapper;
pub use rbatis::Error as BatisError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

pub mod model;
mod rbatis_test;

mod error;
mod mysql_storage;

use crate::mysql_storage::MysqlStorage;
use error::{Result, StorageError};
use snafu::ResultExt;
use std::fmt::{Debug, Display};
use crate::model::job_info::JobInfo;

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub addr: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            addr: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            database: "".to_string(),
            max_connections: 0,
            min_connections: 0,
            connect_timeout: 0,
            idle_timeout: 0,
        }
    }
}

pub trait Storage {
    fn save<T>(&self, t: T) -> Result<()>
        where
            T: CRUDTable;

    fn save_batch<T>(&self, t: &[T]) -> Result<()>
        where
            T: CRUDTable;

    fn delete<T>(&self, id: &T::IdType) -> Result<u64>
        where
            T: CRUDTable;

    fn delete_batch<T>(&self, ids: &[T::IdType]) -> Result<()>
        where
            T: CRUDTable;

    fn fetch<T>(&self, w: &Wrapper) -> Result<T>
        where
            T: CRUDTable;

    fn fetch_page<T>(&self, w: &Wrapper, page_no: u64, page_size: u64) -> Result<Page<T>>
        where
            T: CRUDTable;

    fn update<T>(&self, models: &mut [T]) -> Result<()>
        where
            T: CRUDTable;

    fn find_all_by_current_server<T>(&self) -> Result<Option<Vec<T>>>
        where
            T: CRUDTable;

    fn find_cron_jobs(&self, ids: &[u64], time_threshold: i64) -> Result<Vec<JobInfo>>;

    fn find_frequent_jobs(&self, ids: &[u64]) -> Result<Vec<JobInfo>>;

    fn find_frequent_instance_by_job_id(&self, ids: &[u64]) -> Result<Vec<u64>>;
}

/// Storage Builder.
pub struct StorageBuilder {
    config: StorageConfig,
}

impl StorageBuilder {
    pub fn builder() -> Self {
        Self {
            config: StorageConfig::default(),
        }
    }

    pub fn config(self, config: StorageConfig) -> Self {
        Self { config }
    }

    pub fn build(self) -> MysqlStorage {
        MysqlStorage::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Storage, StorageBuilder, StorageConfig};
    use fastjob_components_utils::component::Component;
    use rbatis::crud::CRUDTable;
    use serde::Deserialize;
    use serde::Serialize;

    // #[derive(Serialize, Deserialize, Clone, Debug)]
    // struct BizActivity {
    //     pub id: Option<String>,
    //     pub name: Option<String>,
    //     pub pc_link: Option<String>,
    //     pub h5_link: Option<String>,
    //     pub pc_banner_img: Option<String>,
    //     pub h5_banner_img: Option<String>,
    //     pub sort: Option<String>,
    //     pub status: Option<i32>,
    //     pub remark: Option<String>,
    //     pub create_time: Option<String>,
    //     pub version: Option<i32>,
    //     pub delete_flag: Option<i32>,
    // }
    //
    // impl CRUDTable for BizActivity {
    //     type IdType = String;
    //
    //     fn get_id(&self) -> Option<&Self::IdType> {
    //         self.id.as_ref()
    //     }
    // }
    //
    // #[test]
    // fn test_save() {
    //     let config = StorageConfig {
    //         addr: "localhost:3306".to_string(),
    //         username: "root".to_string(),
    //         password: "yaoyichen52".to_string(),
    //         database: "neptune".to_string(),
    //         max_connections: 20,
    //         min_connections: 5,
    //         connect_timeout: 5,
    //         idle_timeout: 5,
    //     };
    //     let mut storage = StorageBuilder::builder().config(config).build();
    //
    //     let activity = BizActivity {
    //         id: Some("12312".to_string()),
    //         name: Some("111".to_string()),
    //         pc_link: None,
    //         h5_link: None,
    //         pc_banner_img: None,
    //         h5_banner_img: None,
    //         sort: Some("0".to_string()),
    //         status: Some(1),
    //         remark: None,
    //         create_time: Some("2020-02-09 00:00:00".to_string()),
    //         version: Some(1),
    //         delete_flag: Some(1),
    //     };
    //     storage.start();
    //     storage.save(&activity);
    // }
}
