use fastjob_components_error::Error;
use rbatis::core::db::DBPoolOptions;
use rbatis::crud::{CRUDTable, CRUD};
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::{Rbatis, RbatisOption};
use rbatis::wrapper::Wrapper;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use fastjob_components_utils::component::Component;

pub mod model;
mod rbatis_test;

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
    fn save<T>(&self, t: &T) -> Result<(), Error>
        where
            T: CRUDTable;

    fn save_batch<T>(&self, t: &[T]) -> Result<(), Error>
        where
            T: CRUDTable;

    fn delete<T>(&self, id: &T::IdType) -> Result<u64, Error>
        where
            T: CRUDTable;

    fn delete_batch<T>(&self, ids: &[T::IdType]) -> Result<(), Error>
        where
            T: CRUDTable;

    fn fetch<T>(&self, w: &Wrapper) -> Result<T, Error>
        where
            T: CRUDTable;

    fn fetch_page<T>(&self, w: &Wrapper, page_no: u64, page_size: u64) -> Result<Page<T>, Error>
        where
            T: CRUDTable;

    fn update<T>(&self, models: &mut [T]) -> Result<(), Error>
        where
            T: CRUDTable;
}

pub struct MysqlStorage {
    config: StorageConfig,
    rb: Rbatis,
}

impl Clone for MysqlStorage {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rb: Rbatis::new(),
        }
    }
}

impl MysqlStorage {
    pub fn new(config: StorageConfig) -> Self {
        let opt = RbatisOption::default();
        let rb = Rbatis::new_with_opt(opt);
        Self { config, rb }
    }
}

impl Component for MysqlStorage {
    fn prepare(&self) {
        rbatis::core::runtime::task::block_on(async {
            // rb.link("mysql://root:yaoyichen52@localhost:3306/neptune")
            //     .await
            //     .unwrap();
            let mut link_opt = DBPoolOptions::new();
            link_opt.max_connections = self.config.max_connections;
            link_opt.connect_timeout = Duration::new(self.config.connect_timeout, 0);
            link_opt.idle_timeout = Some(Duration::new(self.config.idle_timeout, 0));
            link_opt.min_connections = self.config.min_connections;

            let derive_url = format!(
                "mysql://{}:{}@{}/{}",
                self.config.username, self.config.password, self.config.addr, self.config.database
            );
            self.rb.link_opt(&derive_url, &link_opt).await.unwrap();
        });
    }

    fn start(&self) {
        todo!()
    }

    fn stop(&self) {
        todo!()
    }
}

impl Storage for MysqlStorage {
    fn save<T>(&self, model: &T) -> Result<(), Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.save("", model).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn save_batch<T>(&self, model: &[T]) -> Result<(), Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.save_batch("", model).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn delete<T>(&self, id: &T::IdType) -> Result<u64, Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.remove_by_id::<T>("", id).await
        }) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn delete_batch<T>(&self, ids: &[<T as CRUDTable>::IdType]) -> Result<(), Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.remove_batch_by_id::<T>("", ids).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn fetch<T>(&self, w: &Wrapper) -> Result<T, Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.fetch_by_wrapper("", w).await
        }) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn fetch_page<T>(&self, w: &Wrapper, page_no: u64, page_size: u64) -> Result<Page<T>, Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            let page = &PageRequest::new(page_no, page_size);
            self.rb.fetch_page_by_wrapper("", w, page).await
        }) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn update<T>(&self, modes: &mut [T]) -> Result<(), Error>
        where
            T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.update_batch_by_id("", modes).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
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
    use rbatis::crud::CRUDTable;
    use serde::Deserialize;
    use serde::Serialize;
    use fastjob_components_utils::component::Component;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub pc_link: Option<String>,
        pub h5_link: Option<String>,
        pub pc_banner_img: Option<String>,
        pub h5_banner_img: Option<String>,
        pub sort: Option<String>,
        pub status: Option<i32>,
        pub remark: Option<String>,
        pub create_time: Option<String>,
        pub version: Option<i32>,
        pub delete_flag: Option<i32>,
    }

    impl CRUDTable for BizActivity {
        type IdType = String;

        fn get_id(&self) -> Option<&Self::IdType> {
            self.id.as_ref()
        }
    }

    #[test]
    fn test_save() {
        let config = StorageConfig {
            addr: "localhost:3306".to_string(),
            username: "root".to_string(),
            password: "yaoyichen52".to_string(),
            database: "neptune".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout: 5,
            idle_timeout: 5,
        };
        let storage = StorageBuilder::builder().config(config).build();

        let activity = BizActivity {
            id: Some("12312".to_string()),
            name: Some("111".to_string()),
            pc_link: None,
            h5_link: None,
            pc_banner_img: None,
            h5_banner_img: None,
            sort: Some("0".to_string()),
            status: Some(1),
            remark: None,
            create_time: Some("2020-02-09 00:00:00".to_string()),
            version: Some(1),
            delete_flag: Some(1),
        };
        storage.prepare();
        storage.save(&activity);
    }
}
