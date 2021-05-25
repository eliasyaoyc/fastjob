use crossbeam::channel::Receiver;

#[derive(Clone)]
pub struct TimingWheel {
    receiver: Receiver<()>,
}

impl TimingWheel {
    pub fn new(receiver: Receiver<()>) -> Self {
        Self { receiver }
    }

    pub fn push(&mut self) {}

    pub fn pop(&mut self) {}

    #[inline]
    pub fn is_empty(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn t_push() {}

    #[test]
    fn t_pop() {}
}