use std::fmt::Formatter;
use std::net::{SocketAddr, ToSocketAddrs};
use ipnet::IpAdd;

mod config;
mod errors;
pub mod gossip;
mod meta;
pub mod server;

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