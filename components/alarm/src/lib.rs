mod alarm_config;
mod error;
mod qiye_wechat_handle;
mod web_hook_handle;

pub trait Alarm {
    async fn on_failed(&self);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
