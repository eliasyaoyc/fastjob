use crate::{ListenAddr, ListenAddrs};

#[derive(Clone, Debug)]
pub struct MetaManager {}

impl MetaManager {
    pub fn new() -> MetaManager {
        Self {}
    }

    pub fn prepare(&self) {}
    pub fn start(&self) {}
}
