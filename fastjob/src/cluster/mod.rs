use fastjob_components_utils::component::Component;

#[derive(Clone, Debug)]
pub struct Cluster {}

impl Cluster {
    pub fn new() -> Cluster {
        Self {}
    }
}

impl Component for Cluster {
    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}
