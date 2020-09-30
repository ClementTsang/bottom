#![allow(unused_variables)]
#![allow(unused_imports)]
use crate::{app::App, canvas::Painter, constants};
use tui::{
    backend::Backend,
    layout::Constraint,
    layout::Direction,
    layout::Layout,
    layout::{Alignment, Rect},
    terminal::Frame,
    text::Span,
    text::Spans,
    widgets::{Block, Borders, Paragraph, Tabs},
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

        let titles: Vec<Spans<'_>> = app_state
            .config_page_settings
            .iter()
            .map(|category| Spans::from(category.category_name))
            .collect();

        f.render_widget(
            Tabs::new(titles)
                .block(config_block)
                .divider(tui::symbols::line::VERTICAL)
                .style(self.colours.text_style)
                .highlight_style(self.colours.currently_selected_text_style),
            draw_loc,
        )
    }
}
