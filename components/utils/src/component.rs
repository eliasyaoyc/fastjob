pub trait Component: Send + Sync + 'static {
    fn start(&mut self);
    fn stop(&mut self);
}
