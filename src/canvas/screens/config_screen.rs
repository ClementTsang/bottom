use crate::{app::App, canvas::Painter, constants};
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    terminal::Frame,
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
            .title(&" Config ")
            .title_style(self.colours.border_style)
            .style(self.colours.border_style)
            .borders(Borders::ALL)
            .border_style(self.colours.border_style);

        f.render_widget(
            Paragraph::new(self.styled_help_text.iter())
                .block(config_block)
                .style(self.colours.text_style)
                .alignment(Alignment::Left)
                .wrap(true)
                .scroll(
                    app_state
                        .help_dialog_state
                        .scroll_state
                        .current_scroll_index,
                ),
            draw_loc,
        );
    }
}
