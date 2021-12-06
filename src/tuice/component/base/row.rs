use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Component, Event, Status};

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
    fn draw(&mut self, bounds: Rect, frame: &mut Frame<'_, B>) {
        self.children.iter_mut().for_each(|child| {
            // TODO: This is just temp! We need layout!
            child.draw(bounds, frame);
        })
    }

    fn on_event(&mut self, _bounds: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
