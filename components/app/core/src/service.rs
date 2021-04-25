use crate::{
    meta::Meta,
    svc,
    svc::{Service, stack::Param},
    ListenAddr,
};
use fastjob_components_error::Error;
use fastjob_components_utils::drain;
use futures::future;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::net::TcpListener;
use tower::make::Shared;
use tower::timeout::Timeout;
use tower::ServiceBuilder;
use warp::{Filter, Rejection, Reply};

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    addr: ListenAddr,
}

impl Param<ListenAddr> for ServiceConfig {
    fn param(&self) -> ListenAddr {
        self.addr
    }
}

#[derive(Clone, Debug)]
pub struct FastJobServe<S> {
    config: ServiceConfig,
    meta: Meta,
    stack: svc::Stack<S>,
    work_managers: Vec<>,
}

impl FastJobServe<()> {
    pub fn build<S>(config: Config, meta: Meta, stack: S) -> Self <S> {
        match Self::init() {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("init config error {}", e);
                std::process::exit(1);
            }
        }
        Self { config, meta, stack }
    }

    /// Init server, do as follows:
    /// 1. load metadata from disk if exists.
    /// 2. broadcast information about the current node to all `WorkerManger.`
    /// 3. try to stealing task from another node in cluster.
    fn init() -> Result<(), Error> {
        Ok(())
    }

    /// Construct the warp filter that provide the ability of route.
    pub fn construct_filter<F>(&self) -> F
        where
            F: Filter<Extract=impl Reply, Error=Rejection> + Clone,
    {
        warp::put()
            .and(path!(String))
            .map(|path: String| {
                if path == "shutdown" {
                    tracing::info!("FastJob Service recv shutdown command.");
                    match self.backend_shutdown() {
                        Ok(_) => std::process::exit(0),
                        Err(_) => std::process::exit(1),
                    }
                }
            }).or(
            warp::post()
                .and(path!(String))
                .map(|path: String| {
                    if path == "register" {} else if path == "unregister" {}
                }).or(
                warp::get()
                    .and(path!(String))
                    .map(|path: String| {})
            )
        )
    }

    pub fn serve<F>(filter: F, listener: TcpListener)
        where
            F: Filter<Extract=impl Reply, Error=Rejection> + Clone,
    {
        let warp_service = warp::service(filter);
        let service = ServiceBuilder::new()
            .timeout(Duration::from_secs(10))
            .service(warp_service);

        let addr = listener.local_addr().unwrap();

        tracing::info!("FastJob Service Listening on {}", addr);

        hyper::Server::from_tcp(listener)
            .unwrap()
            .serve(Shared::new(service))
            .await?;
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
