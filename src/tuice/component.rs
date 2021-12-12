pub mod base;
pub use base::*;

pub mod widget;
pub use widget::*;

use enum_dispatch::enum_dispatch;
use tui::{layout::Rect, Frame};

use super::{Bounds, Event, LayoutNode, Size, Status};

/// A component displays information and can be interacted with.
#[allow(unused_variables)]
#[enum_dispatch]
pub trait TmpComponent<Message> {
    /// Draws the component.
    fn draw<Backend>(&mut self, area: Rect, frame: &mut Frame<'_, Backend>)
    where
        Backend: tui::backend::Backend;

    /// How a component should react to an [`Event`].
    ///
    /// Defaults to just ignoring the event.
    fn on_event(&mut self, area: Rect, event: Event, messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }

    /// How a component should size itself and its children, given some [`Bounds`].
    ///
    /// Defaults to returning a [`Size`] that fills up the bounds given.
    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        Size {
            width: bounds.max_width,
            height: bounds.max_height,
        }
    }
}
