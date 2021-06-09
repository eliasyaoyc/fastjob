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
use fastjob_components_storage::model::instance_info::{InstanceInfo, InstanceStatus, InstanceType};
use fastjob_components_storage::model::{app_info::AppInfo, job_info::JobInfo, lock::Lock};
use fastjob_components_storage::{BatisError, Storage};
use fastjob_components_utils::grpc_returns::GrpcReturn;
use fastjob_components_utils::sched_pool::{JobHandle, SchedPool};
use fastjob_proto::fastjob::*;
use snafu::ResultExt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Sub, Add};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use std::convert::TryFrom;
use fastjob_components_storage::model::job_info::JobTimeExpressionType;
use fastjob_components_utils::event::{Event, CompletedInstance};
use std::cell::RefCell;
use crate::dispatch::Dispatch;

const WORKER_MANAGER_SCHED_POOL_NUM_SIZE: usize = 2;
const WORKER_MANAGER_SCHED_POOL_NAME: &str = "worker-manager";
const WORKER_MANAGER_INIT_TIME: Duration = Duration::from_secs(2);
const INSTANCE_STATUS_INTERVAL: Duration = Duration::from_millis(10000);
const CLEAN_INTERVAL: Duration = Duration::from_millis(10000);
const RETRY_TIMES: u32 = 3;

pub struct WorkerManager<S: Storage> {
    id: i64,
    address: &'static str,
    sched_pool: SchedPool,
    storage: Arc<S>,
    workers: RefCell<DashMap<u64, WorkerClusterHolder>>,
    scheduler: Scheduler<S>,
    event_handler: EventHandler,
    sender: Sender<Event>,
    dispatch: Dispatch<S>,
}

impl<S: Storage> Debug for WorkerManager<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("worker-manager")
            .field("id", &self.id)
            .field("address", &self.address)
            .field("workers", &self.workers)
            .field("scheduler", &self.scheduler)
            .field("dispatch", &self.dispatch)
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
        let (sched_tx, sched_rx) = channel(1024);
        let workers = RefCell::new(DashMap::default());
        WorkerManager {
            id: self.id,
            address: "",
            sched_pool: SchedPool::new(
                WORKER_MANAGER_SCHED_POOL_NUM_SIZE,
                WORKER_MANAGER_SCHED_POOL_NAME,
            ),
            storage: self.storage,
            workers,
            scheduler: Scheduler::new(self.storage.clone(),
                                      sched_tx.clone()),
            event_handler: EventHandler::new(rx),
            sender: tx,
            dispatch: Dispatch::new(
                sched_rx,
                self.storage.clone(),
                workers.clone(),
            ),
        }
    }
}

/// used for grpc service.
impl<S: Storage> WorkerManager<S> {
    async fn start(&mut self) {
        // First Start dispatch.
        self.dispatch.event_loop().await;

        // Start scheduler thread.
        self.sched_pool.schedule_at_fixed_rate(
            self.scheduler(),
            WORKER_MANAGER_INIT_TIME,
            SCHEDULE_INTERVAL,
        );

        // Start instance status check thread.
        self.sched_pool.schedule_at_fixed_rate(
            self.check_instance_status(),
            WORKER_MANAGER_INIT_TIME,
            INSTANCE_STATUS_INTERVAL,
        );

        // Start clean thread.
        self.sched_pool.schedule_at_fixed_rate(
            self.release(),
            WORKER_MANAGER_INIT_TIME,
            CLEAN_INTERVAL,
        );
    }

    fn stop(&mut self) {
        self.scheduler.shutdown();
    }

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
            app_name_or_id: app_name,
        })
    }

    /// Select the appropriate server according to the appName sent by the worker
    /// And check it whether alive,if dead the current service tries to usurp the throne.
    ///
    /// Thread Safety: Distributed-Lock.
    pub fn lookup(&self, current_server: &str, app_id: &str) -> Result<&str> {
        let cache = &vec![];
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
                    app_name_or_id: app_id,
                });
            }
            let name = rs.as_ref().unwrap().app_name.unwrap();
            let origin_server = rs.as_ref().unwrap().current_server.unwrap().as_str();
            if self.is_active(origin_server, cache) {
                return Ok(origin_server);
            }

            // Server is not available, try server election again, need to lock.
            let lock = Lock::new(app_id, 30000, current_server);
            if !self.lock(lock).is_err() {
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
            rs.take().unwrap().current_server = Some(current_server);
            rs.take().unwrap().gmt_modified = Some(Local::now().timestamp_millis());
            self.storage.save(rs.unwrap());
            info!(
                "[Election] server {} become the new server fo appId {}",
                current_server,
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
            .workers.borrow()
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

        self.update_status(&req).await?;

        if InstanceInfo::finish_status().contains(req.get_instanceStatus()) {
            return Ok(GrpcReturn::success());
        }

        Ok(GrpcReturn::empty())
    }

    /// Handle the deploy contain request.
    pub async fn handle_deploy_container(
        &self,
        req: &DeployContainerRequest,
    ) -> Result<Option<GrpcReturn>> {
        unreachable!()
    }

    /// Handle worker requests to get all processor nodes for the current task.
    pub async fn handle_query_executor_cluster(
        &self,
        req: &QueryExecutorClusterRequest,
    ) -> Result<Option<GrpcReturn>> {
        let job_id = req.get_jobId();
        let app_id = req.get_appId();

        if let Some(job_info) = self.storage.find_job_info_by_id().context(error::WorkerStorageError)? {
            if job_info.app_id.unwrap() != app_id {
                return Err(error::PermissionDenied);
            }
            if let Some(holder) = self.workers.take().get(&app_id) {}
        }
        warn!("[WorkerManager] don't find job (job id = {}) or available workers (app id = {}).", job_id, app_id);
        Ok(GrpcReturn::empty())
    }
}

impl<S: Storage> WorkerManager<S> {
    /// Update instance status (i.e. instance execution num.)
    async fn update_status(&self, req: &ReportInstanceStatusRequest) -> Result<()> {
        let instance_id = req.get_instanceId();
        if let Some(job_info) = self.storage.find_job_info_by_instance_id(instance_id).context(error::WorkerStorageError)? {
            if let Some(mut instance_info) = self.storage.find_instance_by_id(instance_id).context(error::WorkerStorageError)? {

                // drop th expired request.
                if req.get_reportTime() <= instance_info.instance_id.unwrap() {
                    warn!("[WorkerManager instance status] receive the expired status report request: {}, this report will be dropped", req);
                    return Ok(());
                }

                // drop the reported data of non-target worker (split brain issues).
                if req.get_sourceAdrdress != instance_info.task_tracker_address.unwrap() {
                    warn!("[WorkerManager instance status] receive the other Worker report: {}, but current Worker is {}, this report will be dropped",
                          req.get_sourceAddress,
                          instance_info.task_tracker_address.unwrap());
                    return Ok(());
                }

                let status = InstanceStatus::try_from(req.get_instanceStatus())?;
                instance_info.last_report_time = Some(req.get_reportTime());

                // Frequent task don't have failure to retry, so keep running and sync the survival msg to db.
                // Frequent task only has two cases:
                // 1. Running
                // 2. Failure that represent the machine on which the work works overload, so need re-choose available one.
                if JobTimeExpressionType::try_from(job_info.time_expression_type.unwrap())?.is_frequent() {
                    instance_info.status = Some(req.get_instanceStatus());
                    instance_info.result = Some(req.get_result());
                    instance_info.running_times = Some(req.get_result());
                    self.storage.update(&mut instance_info);
                    return Ok(());
                }

                // update running time.
                if instance_info.status.unwrap() == InstanceStatus::WaitingWorkerReceive.into() {
                    instance_info.running_times.unwrap() += 1;
                }

                instance_info.status = Some(req.get_instanceStatus());

                let finished = match status {
                    InstanceStatus::Success => {
                        instance_info.result = Some(req.get_result());
                        instance_info.finished_time = Some(chrono::Local::now().timestamp_millis());
                        true
                    }
                    InstanceStatus::Failed => {
                        // satisfied retry.
                        if instance_info.running_times.unwrap() <= job_info.instance_retry_num.unwrap() as u64 {
                            info!("[WorkerManager instance status] execute instance id: {} failed then will retry, retry num: {}",
                                  req.get_instanceId(),
                                  job_info.instance_retry_num.unwrap(),
                            );
                            instance_info.expected_trigger_time = Some(chrono::Local::now().timestamp_millis().add(Duration::from_secs(10)));
                            instance_info.status = Some(InstanceStatus::WaitingDispatch.into());
                            false
                        } else {
                            // exceed instance max retry num.
                            instance_info.result = Some(req.get_result());
                            instance_info.finished_time = Some(chrono::Local::now().timestamp_millis());
                            warn!("[WorkerManager instance status] instance id: {} exceeded the maximum retry num that can't retry", req.get_instanceId());
                            true
                        }
                    }
                    _ => { false }
                };

                self.storage.update(&mut instance_info);

                if finished {
                    self.sender.send(Event::InstanceCompletedEvent(CompletedInstance {
                        instance_id: req.get_instanceId(),
                        wf_instance_id: req.get_wfInstanceId(),
                        status: req.get_instanceStatus(),
                        result: req.get_result(),
                    }));
                }
            } else {
                warn!(
                    "[WorkerManager instance status] can't find instance {}.",
                    instance_id
                );
                return Ok(());
            }
        }
        Ok(())
    }

    fn is_active(&self, target_server: &str, cache: &[&str]) -> bool {
        if cache.contains(&target_server) {
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

    fn lock(&self, lock: Lock) -> Result<()> {
        self.storage.save(lock).context(error::WorkerStorageError)?;
        Ok(())
    }

    /// Release related resource.
    fn release_resource(&self) {
        // 1. release the local cache.
        // 2. release disk space.
        // 3. delete history records.
    }

    fn check_instance_status(&self) {
        let app_ids = self.storage.find_all_app_id_by_current_server().context(error::WorkerStorageError)?;
    }

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

        let app_ids = self.storage.find_all_app_id_by_current_server().context(error::WorkerStorageError)?;

        match app_ids {
            None => {
                info!("[JobScheduler] current server has no app's job to schedule.");
            }
            Some(ids) => {
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
            }
        };

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
