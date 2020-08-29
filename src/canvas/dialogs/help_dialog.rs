use unicode_width::UnicodeWidthStr;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph},
};

use crate::{app::App, canvas::Painter, constants};

const HELP_BASE: &str = " Help ── Esc to close ";

pub trait HelpDialog {
    fn draw_help_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    );
}

impl HelpDialog for Painter {
    fn draw_help_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        // let help_title = Text::styled(
        //     format!(
        //         " Help ─{}─ Esc to close ",
        //         "─".repeat(
        //             usize::from(draw_loc.width).saturating_sub(HELP_BASE.chars().count() + 2)
        //         )
        //     ),
        //     self.colours.border_style,
        // );

        let help_title = format!(
            " Help ─{}─ Esc to close ",
            "─".repeat(usize::from(draw_loc.width).saturating_sub(HELP_BASE.chars().count() + 2))
        );

        if app_state.should_get_widget_bounds() {
            // We must also recalculate how many lines are wrapping to properly get scrolling to work on
            // small terminal sizes... oh joy.

            let mut overflow_buffer = 0;
            let paragraph_width = std::cmp::max(draw_loc.width.saturating_sub(2), 1);
            let mut prev_section_len = 0;

            constants::HELP_TEXT
                .iter()
                .enumerate()
                .for_each(|(itx, section)| {
                    let mut buffer = 0;

                    if itx == 0 {
                        section.iter().for_each(|text_line| {
                            buffer += UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16
                                / paragraph_width;
                        });

                        app_state.help_dialog_state.index_shortcuts[itx] = 0;
                        prev_section_len = section.len() as u16 + buffer;
                        overflow_buffer += buffer;
                    } else {
                        section.iter().for_each(|text_line| {
                            buffer += UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16
                                / paragraph_width;
                        });

                        app_state.help_dialog_state.index_shortcuts[itx] =
                            app_state.help_dialog_state.index_shortcuts[itx - 1]
                                + 1
                                + prev_section_len;
                        prev_section_len = section.len() as u16 + buffer;
                        overflow_buffer += buffer;
                    }
                });

            app_state.help_dialog_state.scroll_state.max_scroll_index =
                (self.styled_help_text.len() as u16
                    + (constants::HELP_TEXT.len() as u16 - 5)
                    + overflow_buffer)
                    .saturating_sub(draw_loc.height);

            // Fix if over-scrolled
            if app_state
                .help_dialog_state
                .scroll_state
                .current_scroll_index
                >= app_state.help_dialog_state.scroll_state.max_scroll_index
            {
                app_state
                    .help_dialog_state
                    .scroll_state
                    .current_scroll_index = app_state
                    .help_dialog_state
                    .scroll_state
                    .max_scroll_index
                    .saturating_sub(1);
            }
        }

        f.render_widget(
            Paragraph::new(self.styled_help_text.iter())
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
