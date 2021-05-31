use crate::{ListenAddr, ListenAddrs};
use fastjob_components_utils::component::Component;

#[derive(Clone, Debug)]
pub struct MetaManager {}

impl MetaManager {
    pub fn new() -> MetaManager {
        Self {}
    }
}

impl Component for MetaManager {
    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}
