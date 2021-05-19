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
    fn prepare(&self) {
        todo!()
    }

    fn start(&self) {
        todo!()
    }

    fn stop(&self) {
        todo!()
    }
}
