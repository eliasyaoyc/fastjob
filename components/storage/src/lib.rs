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

pub mod error;
mod mysql_storage;

use crate::model::instance_info::InstanceInfo;
use crate::model::job_info::JobInfo;
use crate::mysql_storage::MysqlStorage;
use error::{Result, StorageError};
use snafu::ResultExt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub struct StorageConfig {
    pub address: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

impl Debug for StorageConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("storageConfig")
            .field("address", &self.address)
            .field("username", &self.username)
            .field("password", &self.password)
            .field("database", &self.database)
            .field("maxConnections", &self.max_connections)
            .field("minConnections", &self.min_connections)
            .field("connectTimeout", &self.connect_timeout)
            .field("idleTimeout", &self.idle_timeout)
            .finish()
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            address: "".to_string(),
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

    fn find_job_info_by_instance_id(&self, instance_id: u64) -> Result<Option<JobInfo>>;

    fn find_job_info_by_id(&self, instance_id: u64) -> Result<Option<JobInfo>>;

    fn find_instance_by_id(&self, instance_id: u64) -> Result<Option<InstanceInfo>>;

    fn find_all_by_current_server<T>(&self) -> Result<Option<Vec<T>>>
        where
            T: CRUDTable;

    fn find_cron_jobs(&self, ids: &[u64], time_threshold: i64) -> Result<Vec<JobInfo>>;

    fn find_frequent_jobs(&self, ids: &[u64]) -> Result<Vec<JobInfo>>;

    fn find_frequent_instance_by_job_id(&self, ids: &[u64]) -> Result<Vec<u64>>;

    fn count_instance_by_status(&self, id: u64, status: Vec<u32>) -> Result<u64>;
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
