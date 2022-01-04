use tui::{text::Text, widgets::Paragraph, Frame};

use crate::tuine::{DrawContext, StateContext, TmpComponent};

/// A [`ProcessTable`] is a widget displaying process data, and with controls for searching/filtering entries.
pub struct ProcessTable {}

impl super::AppWidget for ProcessTable {
    fn build_widget(
        ctx: &mut crate::tuine::BuildContext<'_>, painter: &crate::canvas::Painter,
        config: &crate::app::AppConfig, data: &mut crate::data_conversion::ConvertedData<'_>,
    ) -> Self {
        Self {}
    }
}

impl<Message> TmpComponent<Message> for ProcessTable {
    fn draw<Backend>(
        &mut self, _state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        let rect = draw_ctx.global_rect();
        frame.render_widget(
            Paragraph::new(Text::raw("Process Table")).block(tui::widgets::Block::default()),
            rect,
        );
    }
}
