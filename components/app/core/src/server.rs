use crate::log::initial_logger;
use crate::services::FastJobService;
use crate::{gossip::GossipConfig, meta::MetaManager, ListenAddr};
use fastjob_components_error::Error;
use fastjob_components_log::LogFormat;
use fastjob_components_scheduler::Scheduler;
use fastjob_components_storage::{StorageBuilder, StorageConfig};
use fastjob_components_utils::component::Component;
use fastjob_components_utils::Either;
use fastjob_components_worker::worker_manager::WorkerManager;
use fastjob_proto::fastjob_grpc::create_fast_job;
use futures::prelude::*;
use futures::{future, FutureExt};
use grpcio::{
    ChannelBuilder, EnvBuilder, RpcContext, Server as GrpcServer, ServerBuilder, UnarySink,
};
use grpcio_health::{create_health, HealthService, ServingStatus};
use std::future::Future;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

const GRPC_SERVER: &str = "GRPC-SERVER";

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub addr: String,
    /// Consensus algorithm related config.
    pub gossip: GossipConfig,
    pub storage_config: StorageConfig,
    pub log_level: slog::Level,
    pub log_file: String,
    pub log_format: LogFormat,
    pub slow_log_file: String,
    pub slow_log_threshold: Duration,
    pub log_rotation_timespan: Duration,
    pub log_rotation_size: u64,
}

pub struct Server {
    id: u64,
    addr: SocketAddr,
    pub config: ServiceConfig,
    /// A GrpcServer build or a GrpcServer.
    ///
    /// If the listening port is configured, the server will be started lazily.
    builder_or_server: Option<Either<ServerBuilder, GrpcServer>>,
    health_service: HealthService,
    components: Vec<Box<dyn Component>>,
}

impl Server {
    pub fn build(id: u64, config: &ServiceConfig) -> Self {
        let addr = SocketAddr::from_str(&config.addr).unwrap();

        let health_service = HealthService::default();

        let env = Arc::new(EnvBuilder::new().name_prefix(GRPC_SERVER).build());
        let channel_args = ChannelBuilder::new(Arc::clone(&env)).build_args();

        // Constructor FastJob service.
        let fastjob_service = FastJobService::new();
        fastjob_service.prepare();

        let builder = {
            let mut sb = ServerBuilder::new(Arc::clone(&env))
                .channel_args(channel_args)
                .register_service(create_fast_job(fastjob_service))
                .register_service(create_health(health_service.clone()));
            sb = sb.bind(format!("{}", &addr.ip()), addr.port());
            Either::Left(sb)
        };

        // Constructor Storage.
        let storage = StorageBuilder::builder()
            .config(config.storage_config.clone())
            .build();

        // Constructor MetaManager.
        let meta_mgr = MetaManager::new();

        // Constructor Scheduler.
        let scheduler = Scheduler::new();

        let components: Vec<Box<dyn Component>> =
            vec![Box::new(storage), Box::new(meta_mgr), Box::new(scheduler)];

        let mut serve = Self {
            id,
            addr,
            config: config.clone(),
            builder_or_server: Some(builder),
            health_service,
            components,
        };

        match serve.pre_start() {
            Ok(_) => {}
            Err(e) => {
                error!("init config error {}.", e);
                std::process::exit(1);
            }
        };

        serve
    }

    /// pre_start server, do as follows:
    /// 1. load metadata from disk if exists.
    /// 2. broadcast information about the current node to all `WorkerManger.`
    /// 3. try to stealing task from another node in cluster.
    fn pre_start(&mut self) -> Result<(), Error> {
        // Start all inner components.
        if !self.components.is_empty() {
            for elem in self.components.iter_mut() {
                elem.prepare();
            }
        }
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
        // 1. setup log component.
        initial_logger(&self.config);

        log_info();

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
        info!("listening on addr {}.", self.addr);
        grpc_server.start();
        self.builder_or_server = Some(Either::Right(grpc_server));

        self.health_service
            .set_serving_status("", ServingStatus::Serving);
        info!("FastJob Server is ready to serve.");
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

fn log_info() {
    info!("Welcome to FastJob.");
}
