#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ComponentStatus {
    Initialized,
    Ready,
    Starting,
    Running,
    Terminating,
    Shutdown,
}

pub trait Component: Send + Sync + 'static {
    fn start(&mut self);
    fn stop(&mut self);
}
