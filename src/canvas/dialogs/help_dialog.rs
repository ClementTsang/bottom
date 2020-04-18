use std::cmp::max;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{App, AppHelpCategory},
    canvas::Painter,
};

const HELP_BASE: &str = " Help ── 1: General ─── 2: Processes ─── 3: Search ─── Esc to close ";

pub trait HelpDialog {
    fn draw_help_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    );
}

impl HelpDialog for Painter {
    fn draw_help_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        let repeat_num = max(
            0,
            draw_loc.width as i32 - HELP_BASE.chars().count() as i32 - 2,
        );
        let help_title = format!(
            " Help ─{}─ 1: General ─── 2: Processes ─── 3: Search ─── Esc to close ",
            "─".repeat(repeat_num as usize)
        );

        f.render_widget(
            Paragraph::new(
                match app_state.help_dialog_state.current_category {
                    AppHelpCategory::General => &self.styled_general_help_text,
                    AppHelpCategory::Process => &self.styled_process_help_text,
                    AppHelpCategory::Search => &self.styled_search_help_text,
                }
                .iter(),
            )
            .block(
                Block::default()
                    .title(&help_title)
                    .title_style(self.colours.border_style)
                    .style(self.colours.border_style)
                    .borders(Borders::ALL)
                    .border_style(self.colours.border_style),
            )
            .style(self.colours.text_style)
            .alignment(Alignment::Left)
            .wrap(true),
            draw_loc,
        );
    }
}
