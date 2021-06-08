pub enum Event {
    AlarmEvent,
    InstanceCompletedEvent(CompletedInstance),
}

pub struct CompletedInstance {
    pub instance_id: u64,
    pub wf_instance_id: u64,
    pub status: usize,
    pub result: &'static str,
}