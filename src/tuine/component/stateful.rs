use crate::tuine::{State, StateContext, ViewContext};

use super::TmpComponent;

/// A [`StatefulComponent`] is a builder-style pattern for building a stateful
/// [`Component`].
///
/// Inspired by Flutter's [StatefulWidget class](https://api.flutter.dev/flutter/widgets/StatefulWidget-class.html).
pub trait StatefulComponent<Message>: TmpComponent<Message> {
    type Properties;
    type ComponentState: State;

    #[track_caller]
    fn build(ctx: &mut ViewContext<'_>, props: Self::Properties) -> Self;
}
