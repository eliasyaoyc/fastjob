use fastjob_components_utils::component::Component;

mod primary_scheduler;
mod priority_scheduler;

// pub enum Scheduler {
//     PrimaryScheduler,
//     PriorityScheduler,
// }

#[derive(Clone)]
pub struct Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Scheduler {
    fn prepare(&mut self) {
        todo!()
    }

    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

pub trait Executor {
    fn get_algorithm(&self, name: &str);
    fn register_algorithm(&mut self, name: &str);
    fn unregister_algorithm(&mut self, name: &str);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
