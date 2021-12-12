use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Event, Status, TmpComponent};

pub struct Carousel {}

impl<Message> TmpComponent<Message> for Carousel {
    fn draw<B>(&mut self, _area: Rect, _frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }
}
