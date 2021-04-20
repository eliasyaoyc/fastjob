use crate::Scheduler;

#[derive(Debug, Clone)]
pub struct PrimaryScheduler {}

impl PrimaryScheduler {
    pub fn new() -> Self {
        PrimaryScheduler {}
    }

    fn fit_predicate() {
        todo!()
    }
}

impl Scheduler for PrimaryScheduler {
    fn get_algorithm(&self, name: &str) {
        todo!()
    }

    fn register_algorithm(&mut self, name: &str) {
        todo!()
    }

    fn unregister_algorithm(&mut self, name: &str) {
        todo!()
    }
}