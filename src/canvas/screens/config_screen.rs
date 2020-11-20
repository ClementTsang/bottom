#![allow(unused_variables)] //FIXME: Remove this
#![allow(unused_imports)] //FIXME: Remove this
use crate::{app::App, canvas::Painter, constants};
use tui::{
    backend::Backend,
    layout::Constraint,
    layout::Direction,
    layout::Layout,
    layout::{Alignment, Rect},
    terminal::Frame,
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

pub trait ConfigScreen {
    fn draw_config_screen<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    );
}

impl ConfigScreen for Painter {
    fn draw_config_screen<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        let config_block = Block::default()
            .title(Span::styled(" Config ", self.colours.widget_title_style))
            .style(self.colours.border_style)
            .borders(Borders::ALL)
            .border_style(self.colours.border_style);

        f.render_widget(config_block, draw_loc);
    }
}
