use crate::drain::Watch;
use fastjob_components_stack::{layer, Service};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;

/// Holds a drain::Watch for as long as a request is pending.
#[derive(Clone, Debug)]
pub struct Retain<S> {
    inner: S,
    drain: Watch,
}

impl<S> Retain<S> {
    pub fn new(drain: Watch, inner: S) -> Self {
        Self { inner, drain }
    }

    pub fn layer(drain: Watch) -> impl layer::Layer<S, Service=Self> + Clone {
        layer::mk(move |inner| Self::new(drain.clone(), inner))
    }
}

impl<Req, S> tower::Service<Req> for Retain<S>
    where
        S: tower::Service<Req>,
        S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output=Result<S::Response, S::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        Box::pin(
            self.drain
                .clone()
                .ignore_signaled()
                .release_after(self.inner.call(req)),
        )
    }
}