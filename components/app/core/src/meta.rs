use crate::svc::stack::Param;
use crate::{ListenAddr, ListenAddrs};

#[derive(Clone, Copy)]
pub struct Meta {}

impl Meta {
    pub fn new() -> Meta {
        Self {}
    }
}
