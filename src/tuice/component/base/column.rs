use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Component, DrawContext, Event, Status};

pub struct Column {}

impl<Message, B> Component<Message, B> for Column
where
    B: Backend,
{
    fn draw(&mut self, _area: Rect, _context: &DrawContext, _frame: &mut Frame<'_, B>) {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
