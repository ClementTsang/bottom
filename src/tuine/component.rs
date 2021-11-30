pub mod text_table;
pub use text_table::{TextColumn, TextColumnConstraint, TextTable};

pub mod shortcut;
pub use shortcut::Shortcut;

use tui::{backend::Backend, layout::Rect, Frame};

use super::{Event, Status};

pub type ShouldRender = bool;

/// A  is an element that displays information and can be interacted with.
#[allow(unused_variables)]
pub trait Component {
    /// How to inform a component after some event takes place. Typically some enum.
    type Message: 'static;

    /// Information passed to the component from its parent.
    type Properties;

    /// Handles an [`Event`]. Defaults to just ignoring the event.
    fn on_event(
        &mut self, bounds: Rect, event: Event, messages: &mut Vec<Self::Message>,
    ) -> Status {
        Status::Ignored
    }

    /// How the component should handle a [`Self::Message`]. Defaults to doing nothing.
    fn update(&mut self, message: Self::Message) -> ShouldRender {
        false
    }

    /// How the component should handle an update to its properties. Defaults to doing nothing.
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        false
    }

    /// Draws the component.
    fn draw<B: Backend>(&mut self, bounds: Rect, frame: &mut Frame<'_, B>);
}
