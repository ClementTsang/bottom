use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    Frame,
};

use crate::drawing::{Event, EventStatus};

use super::Node;

pub trait Widget<B: Backend> {
    // /// Returns a hash of the [`Widget`].  Useful for determining whether recalculations are needed due to
    // /// new state.
    // fn hash(&self, state: &mut Hasher);

    /// How the [`Widget`] should handle an event.  Defaults to ignoring the event.
    fn on_event(&mut self, _event: Event) -> EventStatus {
        EventStatus::Ignored
    }

    /// How the [`Widget`] should be drawn, given a [`Node`] for its layout..
    fn draw(&mut self, ctx: &mut Frame<'_, B>, node: &'_ Node);

    /// How the [`Widget`] should be laid out given boundaries.
    fn layout(&self, bounds: Rect) -> Node;

    /// Returns the width of the [`Widget`]
    fn width(&self) -> Constraint;

    /// Returns the height of the [`Widget`]
    fn height(&self) -> Constraint;
}
