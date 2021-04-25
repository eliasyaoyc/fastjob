pub struct GossipConfig {

}
#[derive(Clone)]
pub struct MetaConfig {
    pub peers: ListenAddrs,
}

impl MetaConfig {
    pub fn with_peers(peers: ListenAddrs) -> MetaConfig {
        Self { peers }
    }
}
impl Param<Vec<ListenAddr>> for MetaConfig {
    fn param(&self) -> Vec<ListenAddr> {
        self.ListenAddrs
    }
}
