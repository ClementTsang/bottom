//! State code is heavily inspired by crochet's work - see
//! [here](https://github.com/raphlinus/crochet/blob/master/src/state.rs) for the original.

use std::any::Any;

/// A trait that any sort of [`Component`](crate::tuice::Component) state should implement.
pub trait State {
    fn as_any(&self) -> &dyn Any;

    fn are_equal(&self, other: &dyn State) -> bool;
}

impl<S: PartialEq + 'static> State for S {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn are_equal(&self, other: &dyn State) -> bool {
        other
            .as_any()
            .downcast_ref()
            .map(|other| self == other)
            .unwrap_or(false)
    }
}
