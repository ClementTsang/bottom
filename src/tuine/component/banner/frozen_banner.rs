use tui::{
    layout::{Constraint, Layout},
    style::Style,
    text::Span,
    widgets::Paragraph,
};

use crate::tuine::TmpComponent;

/// A small banner to display that the app is "frozen" from data updates.
pub struct FrozenBanner {
    style: Style,
}

impl<Message> TmpComponent<Message> for FrozenBanner {
    fn draw<Backend>(
        &mut self, _state_ctx: &mut crate::tuine::StateContext<'_>,
        draw_ctx: &crate::tuine::DrawContext<'_>, frame: &mut tui::Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        let rect = draw_ctx.global_rect();

        frame.render_widget(
            Paragraph::new(Span::styled("Frozen, press 'f' to unfreeze", self.style)),
            Layout::default()
                .horizontal_margin(1)
                .constraints([Constraint::Length(1)])
                .split(rect)[0],
        )
    }
}
