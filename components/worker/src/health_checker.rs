pub(crate) struct HealthChecker {
    fail_over: FailOver,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self { fail_over: FailOver {} }
    }

    pub fn run(&self) {}

    pub fn shutdown(&self) {}
}

struct FailOver {}