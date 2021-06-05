pub struct AlarmController {
    alarm_receiver: async_channel::Receiver<()>,
    alarm_chain: Vec<Box<dyn Alarm>>,
}

impl AlarmController {}
