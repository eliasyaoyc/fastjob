use crate::{gossip::GossipConfig, meta::MetaManager, ListenAddr};
use crate::services::FastJobService;
use fastjob_components_error::Error;
use fastjob_components_scheduler::{Scheduler, SchedulerManger};
use fastjob_components_utils::{drain, Either};
use fastjob_components_worker::worker_manager::WorkerManager;
use futures::{future, FutureExt};
use grpcio::{ChannelBuilder, EnvBuilder, Server as GrpcServer, ServerBuilder, RpcContext, UnarySink};
use grpcio_health::{create_health, HealthService, ServingStatus};
use std::future::Future;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use futures::prelude::*;
use fastjob_proto::fastjob_grpc::create_fast_job;
use fastjob_components_storage::StorageConfig;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub addr: String,
    /// Consensus algorithm related config.
    pub gossip: GossipConfig,
    pub log_level: String,
    pub storage_config: StorageConfig,
}

pub struct Server {
    id: usize,
    addr: SocketAddr,
    pub config: ServiceConfig,
    /// A GrpcServer build or a GrpcServer.
    ///
    /// If the listening port is configured, the server will be started lazily.
    builder_or_server: Option<Either<ServerBuilder, GrpcServer>>,
    health_service: HealthService,
}

impl Server {
    pub fn build(id: usize, config: &ServiceConfig) -> Self {
        let addr = SocketAddr::from_str(&config.addr).unwrap();

        let health_service = HealthService::default();

        let env = Arc::new(EnvBuilder::new().name_prefix("GRPC-SERVER").build());
        let channel_args = ChannelBuilder::new(Arc::clone(&env)).build_args();

        let fastjob_service = FastJobService::new(config.storage_config.clone());
        fastjob_service.prepare();

        let builder = {
            let mut sb = ServerBuilder::new(Arc::clone(&env))
                .channel_args(channel_args)
                .register_service(create_fast_job(fastjob_service))
                .register_service(create_health(health_service.clone()));
            sb = sb.bind(format!("{}", &addr.ip()), addr.port());
            Either::Left(sb)
        };

        let serve = Self {
            id,
            addr,
            config: config.clone(),
            builder_or_server: Some(builder),
            // meta_mgr: MetaManager::new(),
            // sched_mgr: SchedulerManger::new(),
            // work_mgrs: vec![],
            health_service,
        };

        match serve.pre_start() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("init config error {}", e);
                std::process::exit(1);
            }
        };
        serve
    }

    /// pre_start server, do as follows:
    /// 1. load metadata from disk if exists.
    /// 2. broadcast information about the current node to all `WorkerManger.`
    /// 3. try to stealing task from another node in cluster.
    fn pre_start(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Register a gRPC service.
    /// Register after starting, it failed and returns the service.
    pub fn register_service(&mut self, svc: grpcio::Service) -> Option<grpcio::Service> {
        match self.builder_or_server.take() {
            Some(Either::Left(mut builder)) => {
                builder = builder.register_service(svc);
                self.builder_or_server = Some(Either::Left(builder));
                None
            }
            Some(server) => {
                self.builder_or_server = Some(server);
                Some(svc)
            }
            None => Some(svc),
        }
    }

    /// Run the fastJob server.
    pub fn run(&mut self) -> Result<(), Error> {
        // Build grpc server and bind to address.
        let sb = self.builder_or_server.take().unwrap().left().unwrap();
        let server = sb.build()?;
        let (host, port) = server.bind_addrs().next().unwrap();
        let addr = SocketAddr::new(IpAddr::from_str(host)?, port);
        self.addr = addr;
        self.builder_or_server = Some(Either::Right(server));

        // 1. start inner scheduler-manager.
        // self.sched_mgr.start();
        // 2. start inner meta-manger.
        // self.meta_mgr.start();

        // 3. start fastjob-server.
        let mut grpc_server = self.builder_or_server.take().unwrap().right().unwrap();
        println!("listening on addr {}", self.addr);
        grpc_server.start();
        self.builder_or_server = Some(Either::Right(grpc_server));

        self.health_service
            .set_serving_status("", ServingStatus::Serving);
        println!("FastJob Server is ready to serve.");
        Ok(())
    }

    /// Shutdown the serve when recv a shutdown api, it will do as follows:
    /// 1. update metadata that remove itself related information，pre-prevent client from registering task with this node again.
    /// 2. transfer task and related task metadata that belong itself, waiting for execute completed
    ///    if the task is not ready (in working), other state will directly transfer.
    /// 3. transfer and storage the current node metadata, if this node just restart simply so directly loading it from disk
    ///    and try to stealing task from other nodes
    fn backend_shutdown(&self) -> Result<(), Error> {
        Ok(())
    }
}
