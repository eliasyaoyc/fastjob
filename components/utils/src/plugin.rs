use crate::component::Component;

/// FastJob plugin system that flexible opening and closing.
pub trait Plugin: Component {
    fn name(&self) -> String;
    fn is_enabled(&self) -> bool;
}