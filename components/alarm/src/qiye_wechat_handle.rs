use crate::Alarm;

pub struct QyWechat {}

impl QyWechat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Alarm for QyWechat {
    async fn on_failed(&self) {
        todo!()
    }
}
