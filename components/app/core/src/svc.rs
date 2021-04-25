pub use fastjob_components_stack::{self as stack, new_service::NewService};
use std::future::Future;
use std::task::{Context, Poll};
use tower::{
    buffer::{Buffer as TowerBuffer, BufferLayer},
    layer::util::{Identity, Stack as Pair},
    make::MakeService,
};
pub use tower::{
    layer::Layer,
    service_fn as mk,
    spawn_ready::SpawnReady,
    util::{Either, MapErrLayer},
    Service, ServiceExt,
};

#[derive(Clone, Debug)]
pub struct Layers<L>(L);

#[derive(Clone, Debug)]
pub struct Stack<S>(S);

pub fn layers() -> Layers<Identity> {
    Layers(Identity::new())
}

pub fn stack<S>(inner: S) -> Stack<S> {
    Stack(inner)
}

impl<L> Layers<L> {
    pub fn push<O>(self, outer: O) -> Layers<Pair<L, O>> {
        Layers(Pair::new(self.0, outer))
    }

    /// Wraps an inner `MakeService` to be a `NewService`.
    pub fn push_into_new_service(
        self,
    ) -> Layers<Pair<L, stack::new_service::FromMakeServiceLayer>> {
        self.push(stack::new_service::FromMakeServiceLayer::default())
    }

    // pub fn push_spawn_buffer<Req>(
    //     self,
    //     capacity: usize,
    // ) -> Layers<Pair<Pair<L,BoxServiceLayer>>>
    // {
    //
    // }
}

impl<S> Stack<S> {
    pub fn push<L: Layer<S>>(self, layer: L) -> Stack<L::Service> {
        Stack(layer.layer(self.0))
    }

    pub fn into_inner(self) -> S {
        self.0
    }
}

impl<T, N> NewService<T> for Stack<N>
where
    N: NewService<T>,
{
    type Service = N::Service;

    fn new_service(&mut self, target: T) -> Self::Service {
        self.0.new_service(t)
    }
}

impl<T, S> tower::Service<T> for Stack<S>
where
    S: tower::Service<T>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: T) -> Self::Future {
        self.0.call(T)
    }
}
