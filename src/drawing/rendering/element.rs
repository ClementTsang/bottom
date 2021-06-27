use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    Frame,
};

use crate::drawing::{Event, EventStatus, Node, Widget};

/// The instantiated and boxed representation of a [`Widget`].
pub struct Element<'a, B: Backend> {
    pub(crate) widget: Box<dyn Widget<B> + 'a>,
}

impl<'a, B: Backend> Element<'a, B> {
    /// Creates a new [`Element`] given a boxed [`Widget`].
    pub fn new(widget: Box<dyn Widget<B> + 'a>) -> Self {
        Self { widget }
    }

    /// How the [`Element`] should handle an event.
    pub fn on_event(&mut self, event: Event) -> EventStatus {
        self.widget.on_event(event)
    }

    /// How the [`Element`] should be drawn given a [`Node`].
    pub fn draw(&mut self, ctx: &mut Frame<'_, B>, node: &'_ Node) {
        self.widget.draw(ctx, node)
    }

    /// How the [`Element`] should be laid out given boundaries.
    pub fn layout(&self, bounds: Rect) -> Node {
        self.widget.layout(bounds)
    }

    /// Returns the width of the [`Element`]
    pub fn width(&self) -> Constraint {
        self.widget.width()
    }

    /// Returns the height of the [`Element`]
    pub fn height(&self) -> Constraint {
        self.widget.height()
    }
}
