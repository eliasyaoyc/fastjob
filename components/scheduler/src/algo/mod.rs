pub mod primary_scheduler;
pub mod priority_scheduler;

/// Algorithm defines the behaviour for a scheduler execute task.
pub trait Algorithm {
    fn get_algorithm(&self, name: &str);
    fn register_algorithm(&mut self, name: &str);
    fn unregister_algorithm(&mut self, name: &str);
}
