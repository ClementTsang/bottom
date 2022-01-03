use tui::{text::Text, widgets::Paragraph, Frame};

use crate::tuine::{DrawContext, StateContext, TmpComponent};

/// A [`MemGraph`] is a widget displaying RAM/SWAP data in a graph-like form.
pub struct MemGraph {}

impl super::AppWidget for MemGraph {
    fn build(
        ctx: &mut crate::tuine::BuildContext<'_>, painter: &crate::canvas::Painter,
        config: &crate::app::AppConfig, data: &mut crate::data_conversion::ConvertedData<'_>,
    ) -> Self {
        Self {}
    }
}

impl<Message> TmpComponent<Message> for MemGraph {
    fn draw<Backend>(
        &mut self, _state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        let rect = draw_ctx.global_rect();
        frame.render_widget(
            Paragraph::new(Text::raw("Mem Graph")).block(tui::widgets::Block::default()),
            rect,
        );
    }
}
