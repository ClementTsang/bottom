use std::cmp::max;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, Text, Widget},
};

use crate::{app::App, canvas::Painter};

const DD_BASE: &str = " Confirm Kill Process ── Esc to close ";
const DD_ERROR_BASE: &str = " Error ── Esc to close ";

pub trait KillDialog {
    fn draw_dd_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) -> bool;

    fn draw_dd_error_dialog<B: Backend>(&self, f: &mut Frame<'_, B>, dd_err: &str, draw_loc: Rect);
}

impl KillDialog for Painter {
    fn draw_dd_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) -> bool {
        if let Some(to_kill_processes) = app_state.get_to_delete_processes() {
            if let Some(first_pid) = to_kill_processes.1.first() {
                let dd_text = vec![
                    if app_state.is_grouped(app_state.current_widget.widget_id) {
                        if to_kill_processes.1.len() != 1 {
                            Text::raw(format!(
                                "\nKill {} processes with the name {}?",
                                to_kill_processes.1.len(),
                                to_kill_processes.0
                            ))
                        } else {
                            Text::raw(format!(
                                "\nKill {} process with the name {}?",
                                to_kill_processes.1.len(),
                                to_kill_processes.0
                            ))
                        }
                    } else {
                        Text::raw(format!(
                            "\nKill process {} with PID {}?",
                            to_kill_processes.0, first_pid
                        ))
                    },
                    Text::raw("\n\n"),
                    if app_state.delete_dialog_state.is_on_yes {
                        Text::styled("Yes", self.colours.currently_selected_text_style)
                    } else {
                        Text::raw("Yes")
                    },
                    Text::raw("                 "),
                    if app_state.delete_dialog_state.is_on_yes {
                        Text::raw("No")
                    } else {
                        Text::styled("No", self.colours.currently_selected_text_style)
                    },
                ];

                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - DD_BASE.chars().count() as i32 - 2,
                );
                let dd_title = format!(
                    " Confirm Kill Process ─{}─ Esc to close ",
                    "─".repeat(repeat_num as usize)
                );

                Paragraph::new(dd_text.iter())
                    .block(
                        Block::default()
                            .title(&dd_title)
                            .title_style(self.colours.border_style)
                            .style(self.colours.border_style)
                            .borders(Borders::ALL)
                            .border_style(self.colours.border_style),
                    )
                    .style(self.colours.text_style)
                    .alignment(Alignment::Center)
                    .wrap(true)
                    .render(f, draw_loc);

                return true;
            }
        }

        // Currently we just return "false" if things go wrong finding
        // the process or a first PID (if an error arises it should be caught).
        // I don't really like this, and I find it ugly, but it works for now.
        false
    }

    fn draw_dd_error_dialog<B: Backend>(&self, f: &mut Frame<'_, B>, dd_err: &str, draw_loc: Rect) {
        let dd_text = [Text::raw(format!(
            "\nFailure to properly kill the process - {}",
            dd_err
        ))];

        let repeat_num = max(
            0,
            draw_loc.width as i32 - DD_ERROR_BASE.chars().count() as i32 - 2,
        );
        let error_title = format!(" Error ─{}─ Esc to close ", "─".repeat(repeat_num as usize));

        Paragraph::new(dd_text.iter())
            .block(
                Block::default()
                    .title(&error_title)
                    .title_style(self.colours.border_style)
                    .style(self.colours.border_style)
                    .borders(Borders::ALL)
                    .border_style(self.colours.border_style),
            )
            .style(self.colours.text_style)
            .alignment(Alignment::Center)
            .wrap(true)
            .render(f, draw_loc);
    }
}
