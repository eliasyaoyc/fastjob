use crate::Alarm;

pub struct QyWechat {}

impl QyWechat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Alarm for QyWechat {
    fn on_failed(&self) {
        todo!()
    }
}
