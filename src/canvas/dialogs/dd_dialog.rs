use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
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
                let dd_text = Text::from(vec![
                    Spans::from(vec![]),
                    Spans::from(vec![
                        if app_state.is_grouped(app_state.current_widget.widget_id) {
                            if to_kill_processes.1.len() != 1 {
                                Span::from(format!(
                                    "\nKill {} processes with the name \"{}\"?",
                                    to_kill_processes.1.len(),
                                    to_kill_processes.0
                                ))
                            } else {
                                Span::from(format!(
                                    "\nKill 1 process with the name \"{}\"?",
                                    to_kill_processes.0
                                ))
                            }
                        } else {
                            Span::from(format!(
                                "\nKill process \"{}\" with PID {}?",
                                to_kill_processes.0, first_pid
                            ))
                        },
                    ]),
                    Spans::from(vec![]),
                    Spans::from(vec![
                        if app_state.delete_dialog_state.is_on_yes {
                            Span::styled("Yes", self.colours.currently_selected_text_style)
                        } else {
                            Span::from("Yes")
                        },
                        Span::from("                 "),
                        if app_state.delete_dialog_state.is_on_yes {
                            Span::from("No")
                        } else {
                            Span::styled("No", self.colours.currently_selected_text_style)
                        },
                    ]),
                ]);

                let dd_title = Span::styled(
                    format!(
                        " Confirm Kill Process ─{}─ Esc to close ",
                        "─".repeat(
                            usize::from(draw_loc.width).saturating_sub(DD_BASE.chars().count() + 2)
                        )
                    ),
                    self.colours.border_style,
                );

                f.render_widget(
                    Paragraph::new(dd_text)
                        .block(
                            Block::default()
                                .title(dd_title)
                                .style(self.colours.border_style)
                                .borders(Borders::ALL)
                                .border_style(self.colours.border_style),
                        )
                        .style(self.colours.text_style)
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: true }),
                    draw_loc,
                );

                return true;
            }
        }

        // Currently we just return "false" if things go wrong finding
        // the process or a first PID (if an error arises it should be caught).
        // I don't really like this, and I find it ugly, but it works for now.
        false
    }

    fn draw_dd_error_dialog<B: Backend>(&self, f: &mut Frame<'_, B>, dd_err: &str, draw_loc: Rect) {
        let dd_text = Span::from(format!(
            "\nFailure to properly kill the process - {}",
            dd_err
        ));

        let error_title = Span::styled(
            format!(
                " Error ─{}─ Esc to close ",
                "─".repeat(
                    usize::from(draw_loc.width).saturating_sub(DD_ERROR_BASE.chars().count() + 2)
                )
            ),
            self.colours.border_style,
        );

        f.render_widget(
            Paragraph::new(dd_text)
                .block(
                    Block::default()
                        .title(error_title)
                        .style(self.colours.border_style)
                        .borders(Borders::ALL)
                        .border_style(self.colours.border_style),
                )
                .style(self.colours.text_style)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            draw_loc,
        );
    }
}
