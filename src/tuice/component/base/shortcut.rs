use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Component, DrawContext, Event, Status};

/// A [`Component`] to handle keyboard shortcuts and assign actions to them.
///
/// Inspired by [Flutter's approach](https://docs.flutter.dev/development/ui/advanced/actions_and_shortcuts).
pub struct Shortcut {}

impl<Message, B: Backend> Component<Message, B> for Shortcut {
    fn draw(&mut self, _context: DrawContext<'_>, _frame: &mut Frame<'_, B>) {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
