use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, Text},
};

use crate::{app::App, canvas::Painter};

const DD_BASE: &str = " Confirm Kill Process ── Esc to close ";
const DD_ERROR_BASE: &str = " Error ── Esc to close ";

pub trait KillDialog {
    fn get_dd_spans(&self, app_state: &App) -> Option<Vec<Text<'_>>>;

    fn draw_dd_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, dd_text: Option<Vec<Text<'_>>>, app_state: &App,
        draw_loc: Rect,
    ) -> bool;
}

impl KillDialog for Painter {
    fn get_dd_spans(&self, app_state: &App) -> Option<Vec<Text<'_>>> {
        if let Some(dd_err) = &app_state.dd_err {
            return Some(vec![
                Text::raw("\n"),
                Text::raw(format!("Failed to kill process.\n{}\n", dd_err)),
                Text::raw("Please press ENTER or ESC to close this dialog."),
            ]);
        } else if let Some(to_kill_processes) = app_state.get_to_delete_processes() {
            if let Some(first_pid) = to_kill_processes.1.first() {
                return Some(vec![
                    Text::raw("\n"),
                    if app_state.is_grouped(app_state.current_widget.widget_id) {
                        if to_kill_processes.1.len() != 1 {
                            Text::raw(format!(
                                "Kill {} processes with the name \"{}\"?",
                                to_kill_processes.1.len(),
                                to_kill_processes.0
                            ))
                        } else {
                            Text::raw(format!(
                                "Kill 1 process with the name \"{}\"?",
                                to_kill_processes.0
                            ))
                        }
                    } else {
                        Text::raw(format!(
                            "Kill process \"{}\" with PID {}?",
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
                    Text::raw("\n"),
                ]);
            }
        }

        None
    }

    fn draw_dd_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, dd_text: Option<Vec<Text<'_>>>, app_state: &App,
        draw_loc: Rect,
    ) -> bool {
        if let Some(dd_text) = dd_text {
            // let dd_title = if app_state.dd_err.is_some() {
            //     Text::styled(
            //         format!(
            //             " Error ─{}─ Esc to close ",
            //             "─".repeat(
            //                 usize::from(draw_loc.width)
            //                     .saturating_sub(DD_ERROR_BASE.chars().count() + 2)
            //             )
            //         ),
            //         self.colours.border_style,
            //     )
            // } else {
            //     Text::styled(
            //         format!(
            //             " Confirm Kill Process ─{}─ Esc to close ",
            //             "─".repeat(
            //                 usize::from(draw_loc.width).saturating_sub(DD_BASE.chars().count() + 2)
            //             )
            //         ),
            //         self.colours.border_style,
            //     )
            // };

            let dd_title = if app_state.dd_err.is_some() {
                format!(
                    " Error ─{}─ Esc to close ",
                    "─".repeat(
                        usize::from(draw_loc.width)
                            .saturating_sub(DD_ERROR_BASE.chars().count() + 2)
                    )
                )
            } else {
                format!(
                    " Confirm Kill Process ─{}─ Esc to close ",
                    "─".repeat(
                        usize::from(draw_loc.width).saturating_sub(DD_BASE.chars().count() + 2)
                    )
                )
            };

            f.render_widget(
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
                    .wrap(true),
                draw_loc,
            );

            if app_state.dd_err.is_some() {
                return app_state.delete_dialog_state.is_showing_dd;
            } else {
                return true;
            }
        }

        // Currently we just return "false" if things go wrong finding
        // the process or a first PID (if an error arises it should be caught).
        // I don't really like this, and I find it ugly, but it works for now.
        false
    }
}
