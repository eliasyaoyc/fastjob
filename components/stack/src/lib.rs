//! Utilities for composing Tower Services.
pub mod new_service;

pub use new_service::{NewService};

mod box_future;
mod layer;
mod box_new_service;
mod map_target;

pub use tower::{
    util::{future_service, FutureService, ServiceExt},
    Service,
};

/// Describes a stack target that can produce `T` typed parameters.
///
/// Stacks (usually layered `NewService` implementations) frequently need to be
/// able to obtain configuration from the stack target, but stack modules are
/// decoupled from any concrete target types. The `Param` trait provides a way to
/// statically guarantee that a given target can provide a configuration parameter.
pub trait Param<T> {
    /// Produces `T`-typed stack parameter.
    fn param(&self) -> T;
}

impl<T: ToOwned> Param<T::Owned> for T {
    fn param(&self) -> <T as ToOwned>::Owned {
        self.to_owned()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
