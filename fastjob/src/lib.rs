mod rt;
mod app;
mod services;
mod config;
mod error;

pub use error::Result;

mod log;
mod meta;
pub mod server;
mod plugin_handler;

use ipnet::IpAdd;
use std::fmt::Formatter;
use std::net::{SocketAddr, ToSocketAddrs};

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
