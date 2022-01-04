use tui::{text::Text, widgets::Paragraph, Frame};

use crate::tuine::{DrawContext, StateContext, TmpComponent};

/// A [`CpuGraph`] is a widget displaying CPU data in a graph-like form, and with controls for showing only
/// specific plots.
pub struct CpuGraph {}

impl super::AppWidget for CpuGraph {
    fn build_widget(
        ctx: &mut crate::tuine::BuildContext<'_>, painter: &crate::canvas::Painter,
        config: &crate::app::AppConfig, data: &mut crate::data_conversion::ConvertedData<'_>,
    ) -> Self {
        Self {}
    }
}

impl<Message> TmpComponent<Message> for CpuGraph {
    fn draw<Backend>(
        &mut self, _state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        let rect = draw_ctx.global_rect();
        frame.render_widget(
            Paragraph::new(Text::raw("CPU Graph")).block(tui::widgets::Block::default()),
            rect,
        );
    }
}
