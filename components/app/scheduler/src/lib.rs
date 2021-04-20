mod priority_scheduler;
mod primary_scheduler;

// pub enum Scheduler {
//     PrimaryScheduler,
//     PriorityScheduler,
// }

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
