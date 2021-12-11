use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Component, Context, Event, Status};

pub struct Block {}

impl<Message, B> Component<Message, B> for Block
where
    B: Backend,
{
    fn draw(&mut self, _area: Rect, _context: &Context, _frame: &mut Frame<'_, B>) {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
