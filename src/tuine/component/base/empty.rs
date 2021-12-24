use tui::{backend::Backend, Frame};

use crate::tuine::{DrawContext, StateContext, TmpComponent};

#[derive(Default)]
pub struct Empty {}

impl<Message> TmpComponent<Message> for Empty {
    fn draw<B>(
        &mut self, _state_ctx: &mut StateContext<'_>, _draw_ctx: &DrawContext<'_>,
        _frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
    }
}
