use crate::svc::stack::Param;
use crate::{ListenAddr, ListenAddrs};

#[derive(Clone, Debug)]
pub struct Meta {}

impl Meta {
    pub fn new() -> Meta {
        Self {}
    }
}
