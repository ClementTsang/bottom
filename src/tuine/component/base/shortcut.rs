use tui::{backend::Backend, Frame};

use crate::tuine::{DrawContext, Event, StateContext, Status, TmpComponent};

/// A [`Component`] to handle keyboard shortcuts and assign actions to them.
///
/// Inspired by [Flutter's approach](https://docs.flutter.dev/development/ui/advanced/actions_and_shortcuts).
pub struct Shortcut {}

impl<Message> TmpComponent<Message> for Shortcut {
    fn draw<B>(
        &mut self, _state_ctx: &mut StateContext<'_>, _draw_ctx: DrawContext<'_>,
        _frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        todo!()
    }

    fn on_event(
        &mut self, _state_ctx: &mut StateContext<'_>, _draw_ctx: DrawContext<'_>, _event: Event,
        _messages: &mut Vec<Message>,
    ) -> Status {
        Status::Ignored
    }
}
