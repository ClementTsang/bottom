use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    app::{App, KillSignal},
    canvas::Painter,
};

const DD_BASE: &str = " Confirm Kill Process ── Esc to close ";
const DD_ERROR_BASE: &str = " Error ── Esc to close ";

pub trait KillDialog {
    fn get_dd_spans(&self, app_state: &App) -> Option<Text<'_>>;

    fn draw_dd_confirm_buttons<B: Backend>(
        &self, f: &mut Frame<'_, B>, button_draw_loc: &Rect, app_state: &mut App,
    );

    fn draw_dd_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, dd_text: Option<Text<'_>>, app_state: &mut App, draw_loc: Rect,
    ) -> bool;
}

impl KillDialog for Painter {
    fn get_dd_spans(&self, app_state: &App) -> Option<Text<'_>> {
        if let Some(dd_err) = &app_state.dd_err {
            return Some(Text::from(vec![
                Spans::default(),
                Spans::from("Failed to kill process."),
                Spans::from(dd_err.clone()),
                Spans::from("Please press ENTER or ESC to close this dialog."),
            ]));
        } else if let Some(to_kill_processes) = app_state.get_to_delete_processes() {
            if let Some(first_pid) = to_kill_processes.1.first() {
                return Some(Text::from(vec![
                    Spans::from(""),
                    if app_state.is_grouped(app_state.current_widget.widget_id) {
                        if to_kill_processes.1.len() != 1 {
                            Spans::from(format!(
                                "Kill {} processes with the name \"{}\"?  Press ENTER to confirm.",
                                to_kill_processes.1.len(),
                                to_kill_processes.0
                            ))
                        } else {
                            Spans::from(format!(
                                "Kill 1 process with the name \"{}\"?  Press ENTER to confirm.",
                                to_kill_processes.0
                            ))
                        }
                    } else {
                        Spans::from(format!(
                            "Kill process \"{}\" with PID {}?  Press ENTER to confirm.",
                            to_kill_processes.0, first_pid
                        ))
                    },
                ]));
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn draw_dd_confirm_buttons<B: Backend>(
        &self, f: &mut Frame<'_, B>, button_draw_loc: &Rect, app_state: &mut App,
    ) {
        let (yes_button, no_button) = match app_state.delete_dialog_state.selected_signal {
            KillSignal::KILL(_) => (
                Span::styled("Yes", self.colours.currently_selected_text_style),
                Span::raw("No"),
            ),
            KillSignal::CANCEL => (
                Span::raw("Yes"),
                Span::styled("No", self.colours.currently_selected_text_style),
            ),
        };

        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(35),
                    Constraint::Percentage(30),
                    Constraint::Percentage(35),
                ]
                .as_ref(),
            )
            .split(*button_draw_loc);

        f.render_widget(
            Paragraph::new(yes_button)
                .block(Block::default())
                .alignment(Alignment::Right),
            button_layout[0],
        );
        f.render_widget(
            Paragraph::new(no_button)
                .block(Block::default())
                .alignment(Alignment::Left),
            button_layout[2],
        );

        if app_state.should_get_widget_bounds() {
            app_state.delete_dialog_state.button_positions = vec![
                (
                    button_layout[2].x,
                    button_layout[2].y,
                    button_layout[2].x + button_layout[2].width,
                    button_layout[2].y + button_layout[2].height,
                    0,
                ),
                (
                    button_layout[0].x,
                    button_layout[0].y,
                    button_layout[0].x + button_layout[0].width,
                    button_layout[0].y + button_layout[0].height,
                    1,
                ),
            ];
        }
    }

    #[cfg(target_family = "unix")]
    fn draw_dd_confirm_buttons<B: Backend>(
        &self, f: &mut Frame<'_, B>, button_draw_loc: &Rect, app_state: &mut App,
    ) {
        let signal_text = vec![
            "0: Cancel",
            "1: HUP",
            "2: INT",
            "3: QUIT",
            "4: ILL",
            "5: TRAP",
            "6: ABRT",
            "7: BUS",
            "8: FPE",
            "9: KILL",
            "10: USR1",
            "11: SEGV",
            "12: USR2",
            "13: PIPE",
            "14: ALRM",
            "15: TERM",
            "16: STKFLT",
            "17: CHLD",
            "18: CONT",
            "19: STOP",
            "20: TSTP",
            "21: TTIN",
            "22: TTOU",
            "23: URG",
            "24: XCPU",
            "25: XFSZ",
            "26: VTALRM",
            "27: PROF",
            "28: WINCH",
            "29: IO",
            "30: PWR",
            "31: SYS",
            "34: RTMIN",
            "35: RTMIN+1",
            "36: RTMIN+2",
            "37: RTMIN+3",
            "38: RTMIN+4",
            "39: RTMIN+5",
            "40: RTMIN+6",
            "41: RTMIN+7",
            "42: RTMIN+8",
            "43: RTMIN+9",
            "44: RTMIN+10",
            "45: RTMIN+11",
            "46: RTMIN+12",
            "47: RTMIN+13",
            "48: RTMIN+14",
            "49: RTMIN+15",
            "50: RTMAX-14",
            "51: RTMAX-13",
            "52: RTMAX-12",
            "53: RTMAX-11",
            "54: RTMAX-10",
            "55: RTMAX-9",
            "56: RTMAX-8",
            "57: RTMAX-7",
            "58: RTMAX-6",
            "59: RTMAX-5",
            "60: RTMAX-4",
            "61: RTMAX-3",
            "62: RTMAX-2",
            "63: RTMAX-1",
            "64: RTMAX",
        ];
        let buttons = signal_text
            .iter()
            .map(|text| ListItem::new(*text))
            .collect::<Vec<ListItem<'_>>>();
        let mut selected = match app_state.delete_dialog_state.selected_signal {
            KillSignal::CANCEL => 0,
            KillSignal::KILL(signal) => signal,
        };
        // 32+33 are skipped
        if selected > 31 {
            selected -= 2;
        }

        let mut state = ListState::default();
        state.select(Some(selected));

        let button_rect = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Length((button_draw_loc.width - 14) / 2),
                    Constraint::Min(0),
                    Constraint::Length((button_draw_loc.width - 14) / 2),
                ]
                .as_ref(),
            )
            .split(*button_draw_loc)[1];

        let layout = List::new(buttons)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(self.colours.currently_selected_text_style);

        f.render_stateful_widget(layout, button_rect, &mut state);

        // if app_state.should_get_widget_bounds() {
        let layout_height: usize = button_rect.height.into();
        app_state.delete_dialog_state.button_positions = (button_rect.y
            ..button_rect.y + button_rect.height)
            .map(|pos| {
                (
                    button_rect.x,
                    pos,
                    button_rect.x + button_rect.width - 1,
                    pos,
                    (if selected >= layout_height {
                        selected - layout_height + 1
                    } else {
                        0
                    }) + ((pos - button_rect.y) as usize),
                )
            })
            .collect::<Vec<(u16, u16, u16, u16, usize)>>();
        // }
    }

    fn draw_dd_dialog<B: Backend>(
        &self, f: &mut Frame<'_, B>, dd_text: Option<Text<'_>>, app_state: &mut App, draw_loc: Rect,
    ) -> bool {
        if let Some(dd_text) = dd_text {
            let dd_title = if app_state.dd_err.is_some() {
                Spans::from(vec![
                    Span::styled(" Error ", self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to close ",
                            "─".repeat(
                                usize::from(draw_loc.width)
                                    .saturating_sub(DD_ERROR_BASE.chars().count() + 2)
                            )
                        ),
                        self.colours.border_style,
                    ),
                ])
            } else {
                Spans::from(vec![
                    Span::styled(" Confirm Kill Process ", self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to close ",
                            "─".repeat(
                                usize::from(draw_loc.width)
                                    .saturating_sub(DD_BASE.chars().count() + 2)
                            )
                        ),
                        self.colours.border_style,
                    ),
                ])
            };

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

            let btn_height;
            #[cfg(target_family = "unix")]
            {
                btn_height = 20;
            }
            #[cfg(target_os = "windows")]
            {
                btn_height = 3;
            }
            // Now draw buttons if needed...
            let split_draw_loc = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    if app_state.dd_err.is_some() {
                        vec![Constraint::Percentage(100)]
                    } else {
                        vec![Constraint::Min(3), Constraint::Length(btn_height)]
                    }
                    .as_ref(),
                )
                .split(draw_loc);

            // This being true implies that dd_err is none.
            if let Some(button_draw_loc) = split_draw_loc.get(1) {
                self.draw_dd_confirm_buttons(f, button_draw_loc, app_state);
            }

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
