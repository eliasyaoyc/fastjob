pub mod worker_manager;

#[derive(Copy, Clone)]
pub struct Worker {
    id: usize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
