pub mod base;
pub use base::*;

use tui::{layout::Rect, Frame};

use super::{Event, Status};

/// A component displays information and can be interacted with.
#[allow(unused_variables)]
pub trait Component<Message, Backend>
where
    Backend: tui::backend::Backend,
{
    /// Handles an [`Event`]. Defaults to just ignoring the event.
    fn on_event(&mut self, bounds: Rect, event: Event, messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }

    /// Returns the desired layout of the component. Defaults to returning
    fn layout(&self) {}

    /// Draws the component.
    fn draw(&mut self, bounds: Rect, frame: &mut Frame<'_, Backend>);
}
