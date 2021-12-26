use crate::tuine::{State, ViewContext};

use super::TmpComponent;

/// A [`StatefulTemplate`] is a builder-style pattern for building a stateful
/// [`Component`].
///
/// Inspired by Flutter's StatefulWidget interface.
pub trait StatefulTemplate<Message> {
    type Component: TmpComponent<Message>;
    type ComponentState: State;

    fn build(self, ctx: &mut ViewContext<'_>) -> Self::Component;
}
