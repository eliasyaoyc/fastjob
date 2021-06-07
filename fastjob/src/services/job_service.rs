use crate::services::GRPC_RESPONSE_CODE;
use crossbeam::channel::Sender;
use fastjob_components_storage::model::job_info::JobInfo;
use fastjob_components_storage::model::task::Task;
use fastjob_components_storage::Storage;
use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_components_worker::worker_manager::{WorkerManager, WorkerManagerBuilder};
use fastjob_proto::fastjob::*;
use fastjob_proto::fastjob_grpc::FastJob;
use futures::prelude::*;
use grpcio::{RpcContext, UnarySink};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

/// Service handles the RPC messages for the `FastJob` service.
#[derive(Clone)]
pub struct Service<S: Storage> {
    // Manager all tasks that belongs itself. Note that this manager collection includes all the
    // workload  and server itself that are registered with the server,so the collection's key is server id
    work_mgr: WorkerManager<S>,
    storage: Arc<S>,
}

impl<S: Storage> Service<S> {
    pub fn new(sender: Sender<Vec<JobInfo>>) -> Self {
        Self {
            work_mgr: WorkerManagerBuilder::builder(
                req.get_workerManagerConfig().clone(),
                sender,
            )
                .id(req.get_workerManagerId())
                .scope(req.get_workerManagerScope())
                .build(),
            storage: Arc::new(S),
        }
    }

    /// Prepare inner components.
    pub fn prepare(&self) {}
}

impl<S: Storage> FastJob for Service<S> {
    fn register_worker_manager(
        &mut self,
        ctx: RpcContext,
        req: RegisterWorkerManagerRequest,
        sink: UnarySink<RegisterWorkerManagerResponse>,
    ) {
        let msg = format!(
            "Hello register_worker_manager {}",
            req.get_workerManagerId()
        );

        debug!(
            "recv register worker manager request, id: {}, addr: {}",
            req.get_workerManagerId(),
            req.get_localAddr()
        );

        let mut resp = RegisterWorkerManagerResponse::default();

        let key = req.get_workerManagerId();

        if !self.work_mgrs.contains_key(&key) {
            let mut worker_mgr = WorkerManagerBuilder::builder(
                req.get_workerManagerConfig().clone(),
                self.sender.clone(),
            )
                .id(req.get_workerManagerId())
                .scope(req.get_workerManagerScope())
                .build();

            // Start worker manager.
            // todo. `Result` needs to be added to determine whether the execution was successful.
            worker_mgr.start();
            self.work_mgrs.insert(key, worker_mgr);
        }

        resp.set_message(msg);
        resp.set_code(GRPC_RESPONSE_CODE);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    fn un_register_worker_manager(
        &mut self,
        ctx: RpcContext,
        req: UnRegisterWorkerManagerRequest,
        sink: UnarySink<UnRegisterWorkerManagerResponse>,
    ) {
        let msg = format!(
            "Hello un_register_worker_manager {}",
            req.get_workerManagerId()
        );
        debug!(
            "recv unregister worker manager request, id: {}, addr: {}",
            req.get_workerManagerId(),
            req.get_localAddr()
        );

        let mut resp = UnRegisterWorkerManagerResponse::default();

        let key = req.get_workerManagerId();

        if self.work_mgrs.contains_key(&key) {
            let res = match self.work_mgrs.remove(&key) {
                Some(mut m) => m.stop(),
                None => {}
            };
        }

        resp.set_message(msg);
        resp.set_code(GRPC_RESPONSE_CODE);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    fn fetch_worker_managers(
        &mut self,
        ctx: RpcContext,
        req: FetchWorkerManagersRequest,
        sink: UnarySink<FetchWorkerManagersResponse>,
    ) {
        debug!("recv fetch worker managers request");
        let mut resp = FetchWorkerManagersResponse::default();
        let msg = format!("{:#?}", self.work_mgrs);
        resp.set_message(msg);
        resp.set_code(GRPC_RESPONSE_CODE);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    /// Receive worker heartbeat request.
    fn heart_beat(
        &mut self,
        ctx: RpcContext,
        req: HeartBeatRequest,
        sink: UnarySink<HeartBeatResponse>,
    ) {
        let msg = format!("success.");
        debug!("receive worker {} heartbeat request.");

        self.work_mgr.handle_worker_heartbeat(&req).await;

        let mut resp = HeartBeatResponse::default();
        self.work_mgr.resp.set_message(msg);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    /// Receive status escalation request for a task instance.
    fn report_instance_status(
        &mut self,
        ctx: RpcContext,
        req: ReportInstanceStatusRequest,
        sink: UnarySink<ReportInstanceStatusResponse>,
    ) {
        let msg = format!("success.");
        debug!("receive worker {} report instance status request.");

        self.work_mgr.handle_report_instance_status(&req).await;

        let mut resp = ReportInstanceStatusResponse::default();
        self.work_mgr.resp.set_message(msg);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    /// Deploy contain request.
    fn deploy_container(
        &mut self,
        ctx: RpcContext,
        req: DeployContainerRequest,
        sink: UnarySink<DeployContainerResponse>,
    ) {
        let msg = format!("success.");
        debug!("receive worker {} report instance status request.");

        self.work_mgr.handle_deploy_container(&req).await;

        let mut resp = ReportInstanceStatusResponse::default();
        self.work_mgr.resp.set_message(msg);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    /// Processes worker requests to get all processor nodes for the current task.
    fn query_executor_cluster(
        &mut self,
        ctx: RpcContext,
        req: QueryExecutorClusterRequest,
        sink: UnarySink<QueryExecutorClusterResponse>,
    ) {
        let msg = format!("success.");
        debug!("receive worker {} report instance status request.");

        self.work_mgr.handle_query_executor_cluster(&req).await;

        let mut resp = ReportInstanceStatusResponse::default();
        self.work_mgr.resp.set_message(msg);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn t_print() {
        let mut map = HashMap::<String, String>::new();
        map.insert("1".into(), "1".into());
        let a = format!("{:#?}", map);
        println!("{}", a)
    }
}
