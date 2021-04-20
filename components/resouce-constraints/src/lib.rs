//! This Cargo achieved resource isolation of the runtime environment is realized through `namespace` and `cgroup`.
mod cgroup;
mod filesystem;
mod mount;
mod namespace;
mod runtime;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
