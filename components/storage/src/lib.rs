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

use crate::model::instance_info::InstanceInfo;
use crate::model::job_info::JobInfo;
use crate::mysql_storage::MysqlStorage;
use error::{Result, StorageError};
use snafu::ResultExt;
use std::fmt::{Debug, Display};

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

    fn update<T>(&self, models: &mut [T]) -> Result<()>
    where
        T: CRUDTable;

    fn find_instance_by_id(&self, instance_id: u64) -> Result<Option<InstanceInfo>>;

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
