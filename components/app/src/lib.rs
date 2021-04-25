pub mod option;

use fastjob_components_error::Error;
use fastjob_components_utils::drain;
use tokio::{
    sync::mpsc,
    time::{self, Duration},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub worker: u32,
    pub gossip: goss,

}

pub struct App {}

impl Config {
    pub async fn build(
        self,
        shutdown_tx: mpsc::UnboundedSender<()>,
    ) -> Result<App, Error> {
        Ok(App {})
    }
}

impl App {
    pub fn spawn(self) -> drain::Signal {
        let App {} = self;
    }
}