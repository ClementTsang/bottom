use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuine::{DrawContext, Event, Status, TmpComponent};

pub struct Carousel {}

impl<Message> TmpComponent<Message> for Carousel {
    fn draw<B>(&mut self, _context: DrawContext<'_>, _frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
