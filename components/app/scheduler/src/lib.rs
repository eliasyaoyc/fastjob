use fastjob_components_utils::component::Component;

mod primary_scheduler;
mod priority_scheduler;

// pub enum Scheduler {
//     PrimaryScheduler,
//     PriorityScheduler,
// }

#[derive(Clone)]
pub struct SchedulerManger {}

impl SchedulerManger {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for SchedulerManger {
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

pub trait Scheduler {
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
