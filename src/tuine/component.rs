pub mod text_table;
pub use text_table::{TextColumn, TextColumnConstraint, TextTable};

pub mod shortcut;
pub use shortcut::Shortcut;

use tui::{backend::Backend, layout::Rect, Frame};

use super::{Event, Status};

/// A [`Component`] is an element that displays information and can be interacted with.
#[allow(unused_variables)]
pub trait Component {
    type Message: 'static;

    /// Handles an [`Event`]. Defaults to just ignoring the event.
    fn on_event(
        &mut self, bounds: Rect, event: Event, messages: &mut Vec<Self::Message>,
    ) -> Status {
        Status::Ignored
    }

    fn update(&mut self, message: Self::Message) {}

    /// Draws the [`Component`].
    fn draw<B: Backend>(&mut self, bounds: Rect, frame: &mut Frame<'_, B>);
}
