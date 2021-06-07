//! Worker Manager only stores tasks and is not going to scheduling.
//! If worker manager checks itself is free, it will start the steal thread
//! that steals the task from another server in FastJob Cluster. However,
//! if don't have enough space that will reject task request and respond a full error message.
//! so client will retry this request that send to another server util success unless achieved
//! the maximum retry numbers and send has failed.
use super::{error, Result};
use crate::event::event_handler::EventHandler;
use crate::{init_grpc_client, Worker, WorkerClusterHolder};
use chrono::Local;
use dashmap::DashMap;
use fastjob_components_scheduler::{Scheduler, SCHEDULE_INTERVAL};
use fastjob_components_storage::model::instance_info::{InstanceInfo, InstanceStatus};
use fastjob_components_storage::model::{app_info::AppInfo, job_info::JobInfo, lock::Lock};
use fastjob_components_storage::{BatisError, Storage};
use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_components_utils::grpc_returns::GrpcReturn;
use fastjob_components_utils::pair::PairCond;
use fastjob_components_utils::sched_pool::{JobHandle, SchedPool};
use fastjob_proto::fastjob::*;
use snafu::ResultExt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Sub;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Receiver};

const WORKER_MANAGER_SCHED_POOL_NUM_SIZE: usize = 2;
const WORKER_MANAGER_SCHED_POOL_NAME: &str = "worker-manager";
const WORKER_MANAGER_FETCH_INIT_TIME: Duration = Duration::from_secs(2);
const RETRY_TIMES: u32 = 3;

pub struct WorkerManager<S: Storage> {
    id: i64,
    address: String,
    sched_pool: SchedPool,
    storage: Arc<S>,
    workers: DashMap<u64, WorkerClusterHolder>,
    scheduler: Scheduler<S>,
    event_handler: EventHandler,
}

impl<S: Storage> Debug for WorkerManager<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("worker-manager")
            .field("id", &self.id)
            .field("address", &self.address)
            .field("workers", &self.workers)
            .field("scheduler", &self.scheduler)
            .finish()
    }
}

pub struct WorkerManagerBuilder<S: Storage> {
    id: i64,
    config: WorkerManagerConfig,
    storage: Arc<S>,
}

impl<S: Storage> WorkerManagerBuilder<S> {
    pub fn builder(config: WorkerManagerConfig, storage: S) -> Self {
        Self {
            id: 0,
            config,
            storage: Arc::new(storage),
        }
    }

    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn build(self) -> WorkerManager<S> {
        let (tx, rx) = channel(1024);
        WorkerManager {
            id: self.id,
            address: "".to_string(),
            sched_pool: SchedPool::new(
                WORKER_MANAGER_SCHED_POOL_NUM_SIZE,
                WORKER_MANAGER_SCHED_POOL_NAME,
            ),
            storage: self.storage,
            workers: DashMap::default(),
            scheduler: Scheduler::new(self.storage.clone(), tx.clone()),
            event_handler: EventHandler::new(rx),
        }
    }
}

impl<S: Storage> Component for WorkerManager<S> {
    fn start(&mut self) {
        // Start scheduler thread.
        self.sched_pool.schedule_at_fixed_rate(
            self.scheduler(),
            WORKER_MANAGER_FETCH_INIT_TIME,
            SCHEDULE_INTERVAL,
        );
    }

    fn stop(&mut self) {
        self.scheduler.shutdown();
    }
}

/// used for grpc service.
impl<S: Storage> WorkerManager<S> {
    /// Connect to worker grpc client.
    pub fn connect(&self, addr: u64) -> Result<()> {
        self.workers.entry(addr).or_insert_with(Worker::new())?;
        Ok(())
    }

    /// Validate worker is effective when worker init.
    pub fn validate_worker(&self, app_name: &str) -> Result<()> {
        let wrapper = self.storage.get_wrapper().eq("app_name", app_name);
        let rs: std::result::Result<AppInfo, BatisError> = self.storage.fetch(&wrapper);

        if rs.is_ok() {
            return Ok(());
        }

        Err(error::WorkerNotRegistered {
            app_name_or_id: String::from(app_name),
        })
    }

    /// Select the appropriate server according to the appName sent by the worker
    /// And check it whether alive,if dead the current service tries to usurp the throne.
    ///
    /// Thread Safety: Distributed-Lock.
    pub fn lookup(&self, current_server: &str, app_id: u64) -> Result<&str> {
        let cache = &Vec::<String>::new();
        if self.address.eq(current_server) {
            return Ok(current_server);
        }
        let wrapper = &self.storage.get_wrapper().eq("id", app_id);
        for _ in 0..RETRY_TIMES {
            let rs: Option<AppInfo> = self
                .storage
                .fetch(wrapper)
                .context(error::WorkerStorageError)?;

            if rs.is_none() {
                return Err(error::WorkerNotRegistered {
                    app_name_or_id: app_id.to_string(),
                });
            }
            let name = rs.as_ref().unwrap().app_name.unwrap();
            let origin_server = rs.as_ref().unwrap().current_server.unwrap().as_str();
            if self.is_active(origin_server, cache) {
                return Ok(origin_server);
            }

            // Server is not available, try server election again, need to lock.
            let lock = Lock::new(String::from(app_id), 30000, String::from(current_server));
            if !self.lock(lock) {
                std::thread::sleep(Duration::from_millis(500));
            }

            // It is possible that a machine has already completed the Server election and needs to be judged again.
            let mut rs: Option<AppInfo> = self
                .storage
                .fetch(wrapper)
                .context(error::WorkerStorageError)?;
            let cur = rs.as_ref().unwrap().current_server.unwrap().as_str();
            if self.is_active(cur, cache) {
                return Ok(cur);
            }
            // Usurpation, native as current server.
            rs.take().unwrap().current_server = Some(current_server.to_string());
            rs.take().unwrap().gmt_modified = Some(Local::now().timestamp_millis());
            self.storage.save(rs.unwrap());
            info!(
                "[Election] server {} become the new server fo appId {}",
                current_server.to_string(),
                app_id
            )
        }
        Err(error::LookupFail {
            server_ip: self.address.clone(),
        })
    }

    /// Immediately execute a schedule.
    pub fn immediate_sched(&mut self) -> Result<()> {
        self.sched()?;
        Ok(())
    }

    /// Handle the worker heartbeat request, then update the correspond worker.
    pub async fn handle_worker_heartbeat(
        &mut self,
        req: &HeartBeatRequest,
    ) -> Result<Option<GrpcReturn>> {
        let app_id = req.get_appId();
        let app_name = req.get_appName();
        let mut holder = self
            .workers
            .entry(app_id)
            .or_insert(WorkerClusterHolder::new(app_name));
        holder.update_worker_status(req);
        Ok(GrpcReturn::success())
    }

    /// Handle the worker report instance status request.
    pub async fn handle_report_instance_status(
        &self,
        req: &ReportInstanceStatusRequest,
    ) -> Result<Option<GrpcReturn>> {
        // handle related workflow.
        if req.get_wfInstanceId() > 0 && req.get_workflowContext() {
            // workerManager.updateWorkflowContext(&req);
        }

        self.update_status(&req).await;

        if InstanceInfo::finish_status().contains(req.get_instanceStatus()) {
            return Ok(GrpcReturn::success());
        }

        Ok(GrpcReturn::empty())
    }

    /// Handle the deploy container request.
    pub async fn handle_deploy_container(
        &self,
        req: &DeployContainerRequest,
    ) -> Result<Option<GrpcReturn>> {
        Ok(GrpcReturn::success())
    }

    /// Handle worker requests to get all processor nodes for the current task.
    pub async fn handle_query_executor_cluster(
        &self,
        req: &QueryExecutorClusterRequest,
    ) -> Result<Option<GrpcReturn>> {
        Ok(GrpcReturn::success())
    }
}

impl<S: Storage> WorkerManager<S> {
    async fn update_status(&self, req: &ReportInstanceStatusRequest) -> Result<()> {

        Ok(())
    }

    fn is_active(&self, target_server: &str, cache: &Vec<String>) -> bool {
        if cache.contains(&target_server.to_string()) {
            return false;
        }
        // send hello request to target server.
        let client = init_grpc_client(target_server);
        let mut req = Ping::default();
        let reply = client.ping(&req).expect("Ping failed");
        if reply.get_code() == 200 {
            return true;
        }
        false
    }

    fn lock(&self, lock: Lock) -> bool {
        let r = self.storage.save(lock);
        if r.is_ok() {
            return true;
        }
        false
    }

    fn release(&self) {}

    fn scheduler(&mut self) {
        match self.sched() {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e)
            }
        }
    }

    fn sched(&mut self) -> Result<()> {
        info!("Schedule task start.");
        let instant = Instant::now();
        let app_ids: Option<Vec<AppInfo>> =
            self.storage.find_all_by_current_server().context("")?;
        if app_ids.is_none() {
            info!("[JobScheduler] current server has no app's job to schedule.");
            return Ok(());
        }
        let ids: Vec<u64> = app_ids.unwrap().iter().map(|app| app.id).collect();

        let ids = ids.as_slice();

        self.clean_useless_worker(ids);

        self.scheduler
            .schedule_cron_job(ids)
            .context(error::SchedulerFailed)?;
        let cron_cost = instant.elapsed();

        self.scheduler
            .schedule_worker_flow(ids.clone())
            .context(error::SchedulerFailed)?;
        let worker_flow_cost = instant.elapsed().sub(cron_cost);

        self.scheduler
            .schedule_frequent_job(ids)
            .context(error::SchedulerFailed)?;
        let frequent_cost = instant.elapsed().sub(worker_flow_cost + cron_cost);

        info!("[JobScheduler] cron schedule cost: {}, workflow schedule cost: {}, frequent schedule: {}", cron_cost, worker_flow_cost, frequent_cost);

        let total_cost = instant.elapsed().as_millis();
        if total_cost > SCHEDULE_INTERVAL.as_millis() {
            warn!(
                "[JobScheduler] The database query is using too much time {} ms",
                total_cost
            );
        }

        Ok(())
    }

    /// Clean the useless workers.
    fn clean_useless_worker(&mut self, app_ids: &[u64]) {
        self.workers.retain(|k, _| app_ids.contains(&k));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn t_sched() {}
}
