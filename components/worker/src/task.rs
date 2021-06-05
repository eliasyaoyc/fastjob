use std::fmt::{Debug, Formatter};

pub struct Task {
    instance_id: u64,
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("task")
            .field("instance_id", &self.instance_id)
            .finish()
    }
}

impl Task {
    pub fn create_task(instance_id: u64) -> Self {
        Self { instance_id }
    }
}
