pub trait Component: Send + Sync + 'static {
    fn prepare(&self);
    fn start(&self);
    fn stop(&self);
}