use std::cmp::{max, min};

use tui::{text::Text, widgets::Paragraph, Frame};

use crate::tuine::{Bounds, DrawContext, LayoutNode, Size, StateContext, TmpComponent};

/// A [`CpuSimple`] is a widget displaying simple CPU stats.
pub struct CpuSimple {}

impl super::AppWidget for CpuSimple {
    fn build(
        ctx: &mut crate::tuine::ViewContext<'_>, painter: &crate::canvas::Painter,
        config: &crate::app::AppConfig, data: &mut crate::data_conversion::ConvertedData<'_>,
    ) -> Self {
        Self {}
    }
}

impl<Message> TmpComponent<Message> for CpuSimple {
    fn draw<Backend>(
        &mut self, _state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        let rect = draw_ctx.global_rect();
        frame.render_widget(
            Paragraph::new(Text::raw("CPU Simple")).block(tui::widgets::Block::default()),
            rect,
        );
    }

    fn layout(&self, bounds: Bounds, _node: &mut LayoutNode) -> Size {
        Size {
            width: bounds.max_width,
            height: max(bounds.min_height, min(4, bounds.max_height)), // FIXME: Temp value - this is not correct; should be based on data.
        }
    }
}
