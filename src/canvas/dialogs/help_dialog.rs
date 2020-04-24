use std::cmp::max;
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
        let repeat_num = max(
            0,
            draw_loc.width as i32 - HELP_BASE.chars().count() as i32 - 2,
        );
        let help_title = format!(" Help ─{}─ Esc to close ", "─".repeat(repeat_num as usize));

        if app_state.is_force_redraw {
            // We must also recalculate how many lines are wrapping to properly get scrolling to work on
            // small terminal sizes... oh joy.

            // TODO: Make this more automated and easier to add.

            let mut overflow_buffer = 0;
            let paragraph_width = draw_loc.width - 2;
            constants::HELP_CONTENTS_TEXT.iter().for_each(|text_line| {
                overflow_buffer +=
                    UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16 / paragraph_width;
            });

            // General
            app_state.help_dialog_state.general_index =
                constants::HELP_CONTENTS_TEXT.len() as u16 + 1 + overflow_buffer;
            constants::GENERAL_HELP_TEXT.iter().for_each(|text_line| {
                overflow_buffer +=
                    UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16 / paragraph_width;
            });

            // CPU
            app_state.help_dialog_state.cpu_index =
                (constants::HELP_CONTENTS_TEXT.len() + constants::GENERAL_HELP_TEXT.len()) as u16
                    + 2
                    + overflow_buffer;
            constants::CPU_HELP_TEXT.iter().for_each(|text_line| {
                overflow_buffer +=
                    UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16 / paragraph_width;
            });

            // Processes
            app_state.help_dialog_state.process_index = (constants::HELP_CONTENTS_TEXT.len()
                + constants::GENERAL_HELP_TEXT.len()
                + constants::CPU_HELP_TEXT.len())
                as u16
                + 3
                + overflow_buffer;
            constants::PROCESS_HELP_TEXT.iter().for_each(|text_line| {
                overflow_buffer +=
                    UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16 / paragraph_width;
            });

            // Search
            app_state.help_dialog_state.search_index = (constants::HELP_CONTENTS_TEXT.len()
                + constants::GENERAL_HELP_TEXT.len()
                + constants::CPU_HELP_TEXT.len()
                + constants::PROCESS_HELP_TEXT.len())
                as u16
                + 4
                + overflow_buffer;
            constants::SEARCH_HELP_TEXT.iter().for_each(|text_line| {
                overflow_buffer +=
                    UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16 / paragraph_width;
            });

            // Battery
            app_state.help_dialog_state.battery_index = (constants::HELP_CONTENTS_TEXT.len()
                + constants::GENERAL_HELP_TEXT.len()
                + constants::CPU_HELP_TEXT.len()
                + constants::PROCESS_HELP_TEXT.len()
                + constants::SEARCH_HELP_TEXT.len())
                as u16
                + 5
                + overflow_buffer;
            constants::BATTERY_HELP_TEXT.iter().for_each(|text_line| {
                overflow_buffer +=
                    UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16 / paragraph_width;
            });

            app_state.help_dialog_state.scroll_state.max_scroll_index =
                (self.styled_help_text.len() as u16
                    + (constants::NUM_CATEGORIES - 3)
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
