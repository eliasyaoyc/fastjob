use std::fmt::Formatter;
use std::net::{SocketAddr, ToSocketAddrs};

pub use app::Config;
pub use error::Result;
use ipnet::IpAdd;

mod app;
mod rt;

mod config;
mod error;
mod services;

mod log;
pub mod server;

#[macro_use]
extern crate fastjob_components_log;

#[derive(Clone)]
pub struct ListenAddrs(pub Vec<ListenAddr>);

#[derive(Clone, Debug)]
pub struct ListenAddr(pub SocketAddr);

impl AsRef<SocketAddr> for ListenAddr {
    fn as_ref(&self) -> &SocketAddr {
        &self.0
    }
}

impl Into<SocketAddr> for ListenAddr {
    fn into(self) -> SocketAddr {
        self.0
    }
}

impl ToSocketAddrs for ListenAddr {
    type Iter = std::option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(Some(self.0).into_iter())
    }
}

impl std::fmt::Display for ListenAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
