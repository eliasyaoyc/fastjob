use crate::Scheduler;
use crate::algo::Algorithm;
use std::collections::hash_map::HashMap;
use std::fmt::{Debug, Formatter};

enum PriorityStrategy {
    LeastTaskPriority,
    ResourceSurplusPriority,
}

impl PriorityStrategy {
    pub fn getStrategy(name: &str) -> Box<dyn PriorityScheduler> {
        return match name {
            "leastTask" => Box::new(LeastTaskPriority::default()),
            "resourceSurplus" => Box::new(ResourceSurplusPriority::default()),
            _ => panic!(""),
        };
    }
}

impl Debug for PriorityStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PriorityStrategy::LeastTaskPriority => {
                writeln!(f, "leastTask")
            }
            PriorityStrategy::ResourceSurplusPriority => {
                writeln!(f, "resourceSurplus")
            }
        }
    }
}

pub trait PriorityScheduler {
    fn priority_map(&self);
    fn priority_reduce(&self);
}

pub struct PrioritySchedulerProvider {
    pub strategies: HashMap<String, Box<dyn PriorityScheduler>>,
}

impl Default for PrioritySchedulerProvider {
    fn default() -> Self {
        PrioritySchedulerProvider {
            strategies: Default::default(),
        }
    }
}

impl PrioritySchedulerProvider {
    pub fn new() -> Self {
        PrioritySchedulerProvider::default()
    }
    pub fn init(&mut self) {
        if self.strategies.is_empty() {
            self.strategies = HashMap::<String, Box<dyn PriorityScheduler>>::new();
        }
    }
}

impl Algorithm for PrioritySchedulerProvider {
    fn get_algorithm(&self, name: &str) {
        self.strategies.get(name);
    }

    fn register_algorithm(&mut self, name: &str) {
        if !self.strategies.contains_key(name) {
            let strategy = PriorityStrategy::getStrategy(name);
            self.strategies.insert(name.into(), strategy);
        }
    }

    fn unregister_algorithm(&mut self, name: &str) {
        if self.strategies.contains_key(name) {
            self.strategies.remove(name);
        }
    }
}

#[derive(Default)]
pub struct LeastTaskPriority {}

#[derive(Default)]
pub struct ResourceSurplusPriority {}

impl PriorityScheduler for LeastTaskPriority {
    fn priority_map(&self) {
        todo!()
    }

    fn priority_reduce(&self) {
        todo!()
    }
}

impl PriorityScheduler for ResourceSurplusPriority {
    fn priority_map(&self) {
        todo!()
    }

    fn priority_reduce(&self) {
        todo!()
    }
}
