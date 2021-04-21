use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A middleware that boxed *just* the response future from an inner service,
/// without erasing the service's type (and its trait impls, such as `Clone`).
///
/// This is primarily useful when a service's `Future` type is not `Unpin` and
/// must be boxed.
#[derive(Clone, Debug)]
pub struct BoxFuture<T>(T);

impl<T> BoxFuture<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn layer() -> impl tower::Layer<T, Service=Self> {
        crate::layer::mk(Self::new)
    }
}

impl<T, R> tower::Service<R> for BoxFuture<T>
    where
        T: tower::Service<R>,
        T::Future: Send + 'static
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        Box::pin(self.0.call(req))
    }
}