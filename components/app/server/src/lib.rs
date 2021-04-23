use std::future::Future;
use fastjob_components_core::svc;
use fastjob_components_error::Error;

#[derive(Clone, Debug)]
pub struct Config {}

#[derive(Clone, Debug)]
pub struct FastJobServe<S> {
    config: Config,
    stack: svc::Stack<S>,
}

pub struct Accept<P> {}

impl FastJobServe<()> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            stack: svc::stack(()),
        }
    }

    pub fn with_stack<S>(self, stack: S) -> FastJobServe<S> {
        FastJobServe {
            config: self.config,
            stack: svc::stack(stack),
        }
    }

    pub fn serve() -> impl Future<Output=()> {}
}

impl<S> FastJobServe<S> {
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn into_stack(self) -> svc::Stack<S> {
        self.stack
    }

    pub fn into_inner(self) -> S {
        self.stack.into_inner()
    }
    pub fn push<L: svc::Layer<S>>(self, layer: L) -> FastJobServe<L::Service> {
        FastJobServe {
            config: self.config,
            stack: self.stack.push(layer),
        }
    }

    pub fn into_server<T, I>(self) -> impl svc::NewService<
        T,
        Service=impl svc::Service<I, Response=(), Error=Error, Future=impl Send>,
    >
        where
            Self: Clone + 'static,
    {}
}