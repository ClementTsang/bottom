use tui::{text::Text, widgets::Paragraph, Frame};

use crate::tuine::{DrawContext, StateContext, TmpComponent};

/// A [`BatteryTable`] is a widget displaying battery stats.
pub struct BatteryTable {}

impl super::AppWidget for BatteryTable {
    fn build_widget(
        ctx: &mut crate::tuine::BuildContext<'_>, painter: &crate::canvas::Painter,
        config: &crate::app::AppConfig, data: &mut crate::data_conversion::ConvertedData<'_>,
    ) -> Self {
        Self {}
    }
}

impl<Message> TmpComponent<Message> for BatteryTable {
    fn draw<Backend>(
        &mut self, _state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        let rect = draw_ctx.global_rect();
        frame.render_widget(
            Paragraph::new(Text::raw("Battery Table")).block(tui::widgets::Block::default()),
            rect,
        );
    }
}
