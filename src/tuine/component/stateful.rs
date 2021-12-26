use crate::tuine::{State, ViewContext};

use super::TmpComponent;

/// A [`StatefulComponent`] is a builder-style pattern for building a stateful
/// [`Component`].
///
/// Inspired by Flutter's StatefulWidget interface.
pub trait StatefulComponent<Message>: TmpComponent<Message> {
    type Properties;
    type ComponentState: State;

    fn build(ctx: &mut ViewContext<'_>, props: Self::Properties) -> Self;
}
