use fastjob_components_utils::component::{Component, ComponentStatus};
use fastjob_components_worker::worker_manager::{WorkerManager, WorkerManagerBuilder};
use fastjob_proto::fastjob::*;
use fastjob_proto::fastjob_grpc::FastJob;
use futures::prelude::*;
use grpcio::{RpcContext, UnarySink};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use fastjob_components_storage::model::task::Task;

const GRPC_RESPONSE_CODE: u64 = 200;

/// Service handles the RPC messages for the `FastJob` service.
#[derive(Clone)]
pub struct Service {
    // Manager all tasks that belongs itself. Note that this manager collection includes all the
    // workload  and server itself that are registered with the server,so the collection's key is server id
    work_mgrs: HashMap<u64, WorkerManager>,
}

impl Service {
    pub fn new() -> Self {
        Self {
            work_mgrs: HashMap::new(),
        }
    }

    /// Prepare inner components.
    pub fn prepare(&self) {}
}

impl FastJob for Service {
    fn register_worker_manager(
        &mut self,
        ctx: RpcContext,
        req: RegisterWorkerManagerRequest,
        sink: UnarySink<RegisterWorkerManagerResponse>,
    ) {
        let msg = format!(
            "Hello register_worker_manager {}",
            req.get_workerManagerId()0-
        );

        debug!(
            "recv register worker manager request, id: {}, addr: {}",
            req.get_workerManagerId(),
            req.get_localAddr()
        );

        let mut resp = RegisterWorkerManagerResponse::default();

        let key = req.get_workerManagerId();

        if !self.work_mgrs.contains_key(&key) {
            let mut worker_mgr =
                WorkerManagerBuilder::builder(req.get_workerManagerConfig().clone())
                    .id(req.get_workerManagerId())
                    .scope(req.get_workerManagerScope())
                    .build();

            // Start worker manager.
            // todo. `Result` needs to be added to determine whether the execution was successful.
            worker_mgr.prepare();
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

    // fn register_task(
    //     &mut self,
    //     ctx: RpcContext,
    //     req: RegisterTaskRequest,
    //     sink: UnarySink<RegisterTaskResponse>,
    // ) {
    //     let msg = format!("Hello register_task {}", req.get_taskId());
    //     debug!("recv register task request");
    //     let mut resp = RegisterTaskResponse::default();
    //
    //     MaybeUninit::<>::uninit();
    //     let task_manager_id = req.get_taskManagerId();
    //     if let Some(mgr) = self.work_mgrs.get_mut(&task_manager_id) {
    //         match mgr.get_status() {
    //             ComponentStatus::Ready => {
    //                 mgr.start();
    //             }
    //             ComponentStatus::Running => {
    //             }
    //             ComponentStatus::Starting => {
    //                 // need to wait.
    //
    //             }
    //             ComponentStatus::Initialized => {
    //                 mgr.prepare();
    //                 mgr.start()();
    //             }
    //             _ => {
    //                 // return failure response.
    //
    //             }
    //         }
    //     }
    //     resp.set_message(msg);
    //     resp.set_code(GRPC_RESPONSE_CODE);
    //     let f = sink
    //         .success(resp)
    //         .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
    //         .map(|_| ());
    //     ctx.spawn(f)
    // }
    //
    // fn un_register_task(
    //     &mut self,
    //     ctx: RpcContext,
    //     req: UnRegisterTaskRequest,
    //     sink: UnarySink<UnRegisterTaskResponse>,
    // ) {
    //     let msg = format!("Hello un_register_task {}", req.get_taskId());
    //     debug!("recv unregister task request");
    //     let mut resp = UnRegisterTaskResponse::default();
    //     resp.set_message(msg);
    //     let f = sink
    //         .success(resp)
    //         .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
    //         .map(|_| ());
    //     ctx.spawn(f)
    // }

    fn entropy_metadata(
        &mut self,
        ctx: RpcContext,
        req: EntropyMetadataRequest,
        sink: UnarySink<EntropyMetadataResponse>,
    ) {
        let msg = format!("Hello entropy_metadata {}", req.get_nodeId());
        let mut resp = EntropyMetadataResponse::default();
        resp.set_message(msg);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    fn direct_mail_metadata(
        &mut self,
        ctx: RpcContext,
        req: DirectMailMetadataRequest,
        sink: UnarySink<DirectMailMetadataResponse>,
    ) {
        let msg = format!("Hello direct_mail_metadata {}", req.get_nodeId());
        debug!("recv direct mail metadata request");
        let mut resp = DirectMailMetadataResponse::default();
        resp.set_message(msg);
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
