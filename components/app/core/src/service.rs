use crate::gossip::GossipConfig;
use crate::{
    meta::Meta,
    svc,
    svc::{stack::Param, Service},
    ListenAddr,
};
use fastjob_components_error::Error;
use fastjob_components_utils::drain;
use fastjob_components_worker::worker_manager::WorkerManager;
use futures::{future, FutureExt};
use hyper::{Body, Response};
use std::future::Future;
use std::net::TcpListener;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tower::make::Shared;
use tower::timeout::Timeout;
use tower::ServiceBuilder;
use warp::{path, Filter, Rejection, Reply};

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub addr: ListenAddr,
    /// Consensus algorithm related config.
    pub gossip: GossipConfig,
    pub log_level: String,
}

impl Param<ListenAddr> for ServiceConfig {
    fn param(&self) -> ListenAddr {
        self.addr.clone()
    }
}

#[derive(Clone, Debug)]
pub struct FastJobServe {
    id: usize,
    config: ServiceConfig,
    meta: Meta,
    work_managers: Vec<WorkerManager>,
}

impl FastJobServe {
    pub fn build(id: usize, config: &ServiceConfig) -> Self {
        let serve = Self {
            id,
            config: config.clone(),
            meta: Meta::new(),
            work_managers: vec![],
        };

        match serve.init() {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("init config error {}", e);
                std::process::exit(1);
            }
        };

        serve
    }

    /// Init server, do as follows:
    /// 1. load metadata from disk if exists.
    /// 2. broadcast information about the current node to all `WorkerManger.`
    /// 3. try to stealing task from another node in cluster.
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Construct the warp filter that provide the ability of route.
    pub fn construct_filter(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(path!(String))
            .map(|path: String| Response::new(Body::empty()))
        // warp::put()
        //     .and(path!(String))
        //     .map(|path: String| {
        //         if path == "shutdown" {
        //             tracing::info!("FastJob Service recv shutdown command.");
        //             match self.backend_shutdown() {
        //                 Ok(_) => std::process::exit(0),
        //                 Err(_) => std::process::exit(1),
        //             }
        //         }
        //         Response::new(Body::empty())
        //     }).or(
        //     warp::post()
        //         .and(path!(String))
        //         .map(|path: String| {
        //             if path == "registerTask" {} else if path == "unregisterTask" {}
        //             path
        //         })
        //         .map(|path: String| {
        //             if path == "registerWorkerManager" {} else if path == "unregisterWorkerManager" {}
        //             Response::new(Body::empty())
        //         })
        //         .or(
        //             warp::get()
        //                 .and(path!(String))
        //                 .map(|path: String| {
        //                     Response::new(Body::empty())
        //                 })
        //         )
        // )
    }

    // pub fn construct_filter(&self) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    //     warp::get()
    //         .and(path!(String))
    //         .map(|path: String| {
    //             Response::new(Body::empty())
    //         })
    // }

    pub async fn serve<F>(&self, listener: TcpListener) -> Result<(), hyper::Error> {
        let warp_service = warp::service(self.construct_filter());
        let service = ServiceBuilder::new()
            .timeout(Duration::from_secs(10))
            .service(warp_service);

        let addr = listener.local_addr().unwrap();

        tracing::info!("FastJob Service Listening on {}", addr);

        hyper::Server::from_tcp(listener)
            .unwrap()
            .serve(Shared::new(service))
            .await?;
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
