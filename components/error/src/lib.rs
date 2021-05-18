mod never;
mod recover;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
