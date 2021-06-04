use crate::Alarm;

pub struct WebHookHandle {

}

impl WebHookHandle {
    pub fn new() -> Self {
        Self {}
    }
}
impl Alarm for WebHookHandle {
    fn on_failed(&self) {
        todo!()
    }
}