mod web_hook_handle;
mod qiye_wechat_handle;
mod error;

pub trait Alarm {
    fn on_failed(&self);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
