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
    fn prepare(&mut self);
    fn start(&mut self);
    fn stop(&mut self);
}
