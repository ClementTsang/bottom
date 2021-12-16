use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Component, DrawContext, Event, Status};

pub struct Block {}

impl<Message, B: Backend> Component<Message, B> for Block {
    fn draw(&mut self, _context: DrawContext<'_>, _frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
