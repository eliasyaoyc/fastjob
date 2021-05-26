use std::collections::HashMap;
use fastjob_components_utils::plugin::Plugin;

#[derive(Debug)]
pub struct PluginHandler {
    // A collection of plugins opened.
    enabled_plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginHandler {
    pub fn new() -> Self {
        Self { enabled_plugins: Default::default() }
    }
}