use std::io::Stdout;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
    Frame,
};

use super::*;

/// An [`Element`] is an instantiated [`Component`].
pub struct Element<'a, Message, B = CrosstermBackend<Stdout>>
where
    B: Backend,
{
    component: Box<dyn Component<Message, B> + 'a>,
}

impl<'a, Message, B> Element<'a, Message, B>
where
    B: Backend,
{
    pub fn new<C: Component<Message, B> + 'a>(component: C) -> Self {
        Self {
            component: Box::new(component),
        }
    }

    /// Draws the element.
    pub fn draw(&mut self, context: DrawContext<'_>, frame: &mut Frame<'_, B>) {
        self.component.draw(context, frame)
    }

    /// How an element should react to an [`Event`].
    pub fn on_event(&mut self, area: Rect, event: Event, messages: &mut Vec<Message>) -> Status {
        self.component.on_event(area, event, messages)
    }

    /// How an element should size itself and its children, given some [`Bounds`].
    pub fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        self.component.layout(bounds, node)
    }
}
