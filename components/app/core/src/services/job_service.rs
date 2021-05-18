use crate::meta::MetaManager;
use fastjob_components_scheduler::SchedulerManger;
use fastjob_components_storage::{MysqlStorage, Storage, StorageBuilder, StorageConfig};
use fastjob_components_worker::worker_manager::WorkerManager;
use fastjob_proto::fastjob::*;
use fastjob_proto::fastjob_grpc::FastJob;
use futures::prelude::*;
use grpcio::{RpcContext, UnarySink};
use std::collections::HashMap;

const GRPC_RESPONSE_CODE: u64 = 200;

/// Service handles the RPC messages for the `FastJob` service.
#[derive(Clone)]
pub struct Service {
    storage: MysqlStorage,
    meta_mgr: MetaManager,
    sched_mgr: SchedulerManger,
    work_mgrs: HashMap<u64, WorkerManager>,
}

impl Service {
    pub fn new(config: StorageConfig) -> Self {
        let storage = StorageBuilder::builder().config(config).build();

        let meta_mgr = MetaManager::new();
        Self {
            storage,
            meta_mgr,
            sched_mgr: SchedulerManger::new(),
            work_mgrs: HashMap::new(),
        }
    }

    /// Prepare inner components.
    pub fn prepare(&self) {
        // prepare mysqlStorage.
        self.storage.prepare();
        self.meta_mgr.prepare();
        self.sched_mgr.prepare();
    }
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
            req.get_workerManagerId()
        );

        dbg!(
            "FastJob recv register worker manager request, id: {}, addr: {}",
            req.get_workerManagerId(),
            req.get_localAddr()
        );

        let mut resp = RegisterWorkerManagerResponse::default();

        let key = req.get_workerManagerId();

        if !self.work_mgrs.contains_key(&key) {
            let mut worker_mgr = WorkerManager::builder(req.get_workerManagerConfig())
                .id(req.get_workerManagerId())
                .scope(req.get_workerManagerScope())
                .build();

            // start worker manager.
            worker_mgr.run();
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
        dbg!(
            "FastJob recv unregister worker manager request, id: {}, addr: {}",
            req.get_workerManagerId(),
            req.get_localAddr()
        );

        let mut resp = UnRegisterWorkerManagerResponse::default();

        let key = req.get_workerManagerId();

        if self.work_mgrs.contains_key(&key) {
            let res = match self.work_mgrs.remove(&key) {
                Some(mut m) => m.shutdown(),
                None => Ok(()),
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
        dbg!("FastJob recv fetch worker managers request");
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

    fn register_task(
        &mut self,
        ctx: RpcContext,
        req: RegisterTaskRequest,
        sink: UnarySink<RegisterTaskResponse>,
    ) {
        let msg = format!("Hello register_task {}", req.get_taskId());
        dbg!("FastJob recv register task request");
        let mut resp = RegisterTaskResponse::default();

        let task_manager_id = req.get_taskManagerId();
        if self.work_mgrs.contains_key(&task_manager_id) {
            let mgr = self.work_mgrs.get_mut(&task_manager_id).unwrap();
            // if mgr
            // mgr.register_task();
        }

        resp.set_message(msg);
        resp.set_code(GRPC_RESPONSE_CODE);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

    fn un_register_task(
        &mut self,
        ctx: RpcContext,
        req: UnRegisterTaskRequest,
        sink: UnarySink<UnRegisterTaskResponse>,
    ) {
        let msg = format!("Hello un_register_task {}", req.get_taskId());
        let mut resp = UnRegisterTaskResponse::default();
        resp.set_message(msg);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }

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
