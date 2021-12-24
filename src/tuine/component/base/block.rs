use tui::{backend::Backend, Frame};

use crate::tuine::{DrawContext, Event, StateContext, Status, TmpComponent};

pub struct Block {}

impl<Message> TmpComponent<Message> for Block {
    fn draw<B>(
        &mut self, _state_ctx: &mut StateContext<'_>, _draw_ctx: DrawContext<'_>,
        _frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        todo!()
    }

    fn on_event(
        &mut self, _state_ctx: &mut StateContext<'_>, _draw_ctx: DrawContext<'_>, _event: Event,
        _messages: &mut Vec<Message>,
    ) -> Status {
        Status::Ignored
    }
}
