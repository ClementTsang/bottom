use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Component, Event, Status};

pub struct Column {}

impl<Message, B> Component<Message, B> for Column
where
    B: Backend,
{
    fn draw(&mut self, _bounds: Rect, _frame: &mut Frame<'_, B>) {
        todo!()
    }

    fn on_event(&mut self, _bounds: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
