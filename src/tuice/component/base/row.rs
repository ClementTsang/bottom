use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Bounds, Component, DrawContext, Event, Size, Status};

#[derive(Default)]
pub struct Row<'a, Message, B>
where
    B: Backend,
{
    children: Vec<Box<dyn Component<Message, B> + 'a>>, // FIXME: For performance purposes, let's cheat and use enum-dispatch
}

impl<'a, Message, B> Row<'a, Message, B>
where
    B: Backend,
{
    /// Creates a new [`Row`] with the given children.
    pub fn with_children<C>(children: Vec<C>) -> Self
    where
        C: Into<Box<dyn Component<Message, B> + 'a>>,
    {
        Self {
            children: children.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a, Message, B> Component<Message, B> for Row<'a, Message, B>
where
    B: Backend,
{
    fn draw(&mut self, area: Rect, context: &DrawContext, frame: &mut Frame<'_, B>) {
        self.children.iter_mut().for_each(|child| {
            // TODO: This is just temp! We need layout!
            child.draw(area, context, frame);
        })
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
