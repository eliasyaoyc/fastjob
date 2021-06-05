use crate::error::{Result, StorageError};
use crate::model::app_info::AppInfo;
use crate::model::instance_info::InstanceInfo;
use crate::model::job_info::{JobInfo, JobStatus, JobTimeExpressionType, JobType};
use crate::{Storage, StorageConfig};
use fastjob_components_utils::component::Component;
use rbatis::core::db::DBPoolOptions;
use rbatis::crud::{CRUDTable, CRUD};
use rbatis::plugin::page::{Page, PageRequest};
use rbatis::rbatis::{Rbatis, RbatisOption};
use rbatis::wrapper::Wrapper;
use rbatis::Error;
use std::time::Duration;

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
    fn start(&mut self) {
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

    fn stop(&mut self) {
        unreachable!()
    }
}

impl Storage for MysqlStorage {
    fn save<'a, T>(&self, model: T) -> Result<()>
    where
        T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.save("", &model).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn save_batch<T>(&self, model: &[T]) -> Result<()>
    where
        T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.save_batch("", model).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn delete<T>(&self, id: &T::IdType) -> Result<u64>
    where
        T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.remove_by_id::<T>("", id).await
        }) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    fn delete_batch<T>(&self, ids: &[<T as CRUDTable>::IdType]) -> Result<()>
    where
        T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.remove_batch_by_id::<T>("", ids).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn update<T>(&self, modes: &mut [T]) -> Result<()>
    where
        T: CRUDTable,
    {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            self.rb.update_batch_by_id("", modes).await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn find_instance_by_id(&self, instance_id: u64) -> Result<Option<InstanceInfo>> {
        match rbatis::core::runtime::task::block_on(async {
            // fast_log::init_log("requests.log", 1000, log::Level::Info, None, true);
            let wrapper = self.get_wrapper().eq("id", instance_id);
            let r: Result<Option<InstanceInfo>> = self.rb.fetch_by_wrapper("", &wrapper).await;
            r
        }) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    fn find_all_by_current_server<T>(&self) -> Result<Option<Vec<T>>>
    where
        T: CRUDTable,
    {
        todo!()
    }

    fn find_cron_jobs(&self, ids: &[u64], time_threshold: i64) -> Result<Vec<JobInfo>> {
        let wrapper = Wrapper::new(&rb.driver_type().unwrap())
            .r#in("appId", ids)
            .and()
            .eq("status", JobStatus::Running.into())
            .and()
            .eq("time_expression_type", JobTimeExpressionType::CRON.into())
            .and()
            .le("next_trigger_time", time_threshold);

        rbatis::core::runtime::task::block_on(async {
            let r: Result<Vec<AppInfo>> = self.rb.fetch_list_by_wrapper("", &wrapper).await;
            r
        })
    }

    fn find_frequent_jobs(&self, ids: &[u64]) -> Result<Vec<JobInfo>> {
        let wrapper = Wrapper::new(&rb.driver_type().unwrap())
            .r#in("appId", ids)
            .and()
            .eq("status", JobStatus::Running.into())
            .and()
            .r#in(
                "time_expression_type",
                &[
                    JobTimeExpressionType::FixRate.into(),
                    JobTimeExpressionType::FixDelay.into(),
                ],
            );

        rbatis::core::runtime::task::block_on(async {
            let r: Result<Vec<AppInfo>> = self.rb.fetch_list_by_wrapper("", &wrapper).await;
            r
        })
    }

    fn find_frequent_instance_by_job_id(&self, ids: &[u64]) -> Result<Vec<u64>> {
        rbatis::core::runtime::task::block_on(async {
            let py = r#"
                   select distinct job_id from instance_info
                   where job_id in #{job_id} and status in #{status}"#;
            let r: Resul<Vec<u64>> = self
                .rb
                .py_fetch(
                    "",
                    py,
                    &serde_json::json!({
                        "job_id": ids,
                        "status": InstanceInfo::generalized_running_status()
                    }),
                )
                .await;
            r
        })
    }
}

impl MysqlStorage {
    fn get_wrapper(&self) -> Wrapper {
        Wrapper::new(&self.rb.driver_type().unwrap())
    }
}
