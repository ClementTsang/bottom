//! A dialog box to handle killing processes.

use cfg_if::cfg_if;
use tui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{List, ListState, Padding, Paragraph, Wrap},
};

use crate::{
    canvas::drawing_utils::dialog_block, collection::processes::Pid, options::config::style::Styles,
};

cfg_if! {
    if #[cfg(target_os = "linux")] {
        const SIGNAL_TEXT: [&str; 63] = [
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
    } else if #[cfg(target_os = "macos")] {
        const SIGNAL_TEXT: [&str; 32] = [
            "0: Cancel",
            "1: HUP",
            "2: INT",
            "3: QUIT",
            "4: ILL",
            "5: TRAP",
            "6: ABRT",
            "7: EMT",
            "8: FPE",
            "9: KILL",
            "10: BUS",
            "11: SEGV",
            "12: SYS",
            "13: PIPE",
            "14: ALRM",
            "15: TERM",
            "16: URG",
            "17: STOP",
            "18: TSTP",
            "19: CONT",
            "20: CHLD",
            "21: TTIN",
            "22: TTOU",
            "23: IO",
            "24: XCPU",
            "25: XFSZ",
            "26: VTALRM",
            "27: PROF",
            "28: WINCH",
            "29: INFO",
            "30: USR1",
            "31: USR2",
        ];
    } else if #[cfg(target_os = "freebsd")] {
        const SIGNAL_TEXT: [&str; 34] = [
            "0: Cancel",
            "1: HUP",
            "2: INT",
            "3: QUIT",
            "4: ILL",
            "5: TRAP",
            "6: ABRT",
            "7: EMT",
            "8: FPE",
            "9: KILL",
            "10: BUS",
            "11: SEGV",
            "12: SYS",
            "13: PIPE",
            "14: ALRM",
            "15: TERM",
            "16: URG",
            "17: STOP",
            "18: TSTP",
            "19: CONT",
            "20: CHLD",
            "21: TTIN",
            "22: TTOU",
            "23: IO",
            "24: XCPU",
            "25: XFSZ",
            "26: VTALRM",
            "27: PROF",
            "28: WINCH",
            "29: INFO",
            "30: USR1",
            "31: USR2",
            "32: THR",
            "33: LIBRT",
        ];
    }
}

/// Button state type for a [`ProcessKillDialog`].
///
/// Simple only has two buttons (yes/no), while signals (AKA advanced) are
/// a list of signals to send.
///
/// Note that signals are not available for Windows.
pub(crate) enum ButtonState {
    #[cfg(not(target_os = "windows"))]
    Signals(ListState),
    Simple {
        yes: bool,
    },
}

/// The current state of the process kill dialog.
#[derive(Default)]
pub(crate) enum ProcessKillDialogState {
    #[default]
    NotEnabled,
    Selecting(String, Vec<Pid>, ButtonState),
    Killing(Vec<Pid>),
    Error(String),
}

/// Process kill dialog.
#[derive(Default)]
pub(crate) struct ProcessKillDialog {
    state: ProcessKillDialogState,
}

impl ProcessKillDialog {
    pub fn reset(&mut self) {
        self.state = ProcessKillDialogState::default();
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        !(matches!(self.state, ProcessKillDialogState::NotEnabled))
    }

    pub fn on_esc(&mut self) {}

    pub fn on_delete(&mut self) {}

    pub fn on_enter(&mut self) {
        let mut current = ProcessKillDialogState::NotEnabled;
        std::mem::swap(&mut self.state, &mut current);

        match &self.state {
            ProcessKillDialogState::NotEnabled => {} // Do nothing
            ProcessKillDialogState::Selecting(name, pids, button_state) => match button_state {
                ButtonState::Signals(list_state) => {}
                ButtonState::Simple { yes } => {
                    if *yes {
                    } else {
                    }
                }
            },
            ProcessKillDialogState::Killing(items) => {}
            ProcessKillDialogState::Error(_) => {}
        }
    }

    pub fn on_char(&mut self, c: char) {
        match c {
            // 'h' => self.on_left_key(),
            // 'j' => self.on_down_key(),
            // 'k' => self.on_up_key(),
            // 'l' => self.on_right_key(),
            // #[cfg(target_family = "unix")]
            // '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            //     self.on_number(caught_char)
            // }
            // 'g' => {
            //     let mut is_first_g = true;
            //     if let Some(second_char) = self.second_char {
            //         if self.awaiting_second_char && second_char == 'g' {
            //             is_first_g = false;
            //             self.awaiting_second_char = false;
            //             self.second_char = None;
            //             self.skip_to_first();
            //         }
            //     }
            //
            //     if is_first_g {
            //         self.awaiting_second_char = true;
            //         self.second_char = Some('g');
            //     }
            // }
            // 'G' => self.skip_to_last(),
            _ => {}
        }
    }

    pub fn on_number(&mut self, value: u32) {
        // if self.delete_dialog_state.is_showing_dd {
        //     if self
        //         .delete_dialog_state
        //         .last_number_press
        //         .map_or(100, |ins| ins.elapsed().as_millis())
        //         >= 400
        //     {
        //         self.delete_dialog_state.keyboard_signal_select = 0;
        //     }
        //     let mut kbd_signal = self.delete_dialog_state.keyboard_signal_select * 10;
        //     kbd_signal += number_char.to_digit(10).unwrap() as usize;
        //     if kbd_signal > 64 {
        //         kbd_signal %= 100;
        //     }
        //     #[cfg(target_os = "linux")]
        //     if kbd_signal > 64 || kbd_signal == 32 || kbd_signal == 33 {
        //         kbd_signal %= 10;
        //     }
        //     #[cfg(target_os = "macos")]
        //     if kbd_signal > 31 {
        //         kbd_signal %= 10;
        //     }
        //     self.delete_dialog_state.selected_signal = KillSignal::Kill(kbd_signal);
        //     if kbd_signal < 10 {
        //         self.delete_dialog_state.keyboard_signal_select = kbd_signal;
        //     } else {
        //         self.delete_dialog_state.keyboard_signal_select = 0;
        //     }
        //     self.delete_dialog_state.last_number_press = Some(Instant::now());
        // }
    }

    pub fn on_click(&mut self) -> bool {
        // if self.is_in_dialog() {
        //     match self.delete_dialog_state.button_positions.iter().find(
        //         |(tl_x, tl_y, br_x, br_y, _idx)| {
        //             (x >= *tl_x && y >= *tl_y) && (x <= *br_x && y <= *br_y)
        //         },
        //     ) {
        //         Some((_, _, _, _, 0)) => {
        //             self.delete_dialog_state.selected_signal = KillSignal::Cancel
        //         }
        //         Some((_, _, _, _, idx)) => {
        //             if *idx > 31 {
        //                 self.delete_dialog_state.selected_signal = KillSignal::Kill(*idx + 2)
        //             } else {
        //                 self.delete_dialog_state.selected_signal = KillSignal::Kill(*idx)
        //             }
        //         }
        //         _ => {}
        //     }
        //     return;
        // }
        false
    }

    /// Scroll up in the signal list.
    pub fn on_scroll_up(&mut self) {
        if let ProcessKillDialogState::Selecting(_, _, button_state) = &mut self.state {
            if let ButtonState::Signals(list_state) = button_state {
                if let Some(selected) = list_state.selected() {
                    if selected > 0 {
                        list_state.select(Some(selected - 1));
                    }
                }
            }
        }
    }

    /// Scroll down in the signal list.
    pub fn on_scroll_down(&mut self) {
        if let ProcessKillDialogState::Selecting(_, _, button_state) = &mut self.state {
            if let ButtonState::Signals(list_state) = button_state {
                if let Some(selected) = list_state.selected() {
                    if selected < SIGNAL_TEXT.len() - 1 {
                        list_state.select(Some(selected + 1));
                    }
                }
            }
        }
    }

    /// Handle a left key press.
    pub fn on_left_key(&mut self) {}

    /// Handle a right key press.
    pub fn on_right_key(&mut self) {}

    /// Handle an up key press.
    pub fn on_up_key(&mut self) {}

    /// Handle a down key press.
    pub fn on_down_key(&mut self) {}

    // Handle page up.
    pub fn on_page_up(&mut self) {
        // let mut new_signal = match self.delete_dialog_state.selected_signal {
        //     KillSignal::Cancel => 0,
        //     KillSignal::Kill(signal) => max(signal, 8) - 8,
        // };
        // if new_signal > 23 && new_signal < 33 {
        //     new_signal -= 2;
        // }
        // self.delete_dialog_state.selected_signal = match new_signal {
        //     0 => KillSignal::Cancel,
        //     sig => KillSignal::Kill(sig),
        // };
    }

    /// Handle page down.
    pub fn on_page_down(&mut self) {
        // let mut new_signal = match self.delete_dialog_state.selected_signal {
        //     KillSignal::Cancel => 8,
        //     KillSignal::Kill(signal) => min(signal + 8, MAX_PROCESS_SIGNAL),
        // };
        // if new_signal > 31 && new_signal < 42 {
        //     new_signal += 2;
        // }
        // self.delete_dialog_state.selected_signal = KillSignal::Kill(new_signal);
    }

    pub fn scroll_to_first(&mut self) {}

    pub fn scroll_to_last(&mut self) {}

    /// Enable the process kill process.
    pub fn start_process_kill(
        &mut self, process_name: String, pids: Vec<Pid>, simple_selection: bool,
    ) {
        self.state = ProcessKillDialogState::Selecting(
            process_name,
            pids,
            if simple_selection {
                ButtonState::Simple { yes: false }
            } else {
                ButtonState::Signals(ListState::default().with_selected(Some(0)))
            },
        )
    }

    #[inline]
    fn draw_selecting(
        f: &mut Frame<'_>, draw_loc: Rect, styles: &Styles, name: &str, pids: &[Pid],
        button_state: &mut ButtonState,
    ) {
        let text = {
            const MAX_PROCESS_NAME_WIDTH: usize = 20;

            if let Some(first_pid) = pids.first() {
                let truncated_process_name =
                    unicode_ellipsis::truncate_str(name, MAX_PROCESS_NAME_WIDTH);

                let text = if pids.len() > 1 {
                    Line::from(format!(
                        "Kill {} processes with the name '{}'? Press ENTER to confirm.",
                        pids.len(),
                        truncated_process_name
                    ))
                } else {
                    Line::from(format!(
                        "Kill process '{truncated_process_name}' with PID {first_pid}? Press ENTER to confirm."
                    ))
                };

                Text::from(vec![text])
            } else {
                Text::from(vec![
                    "Could not find process to kill.".into(),
                    "Please press ENTER or ESC to close this dialog.".into(),
                ])
            }
        };

        let text = Paragraph::new(text)
            .style(styles.text_style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let title = match button_state {
            #[cfg(not(target_os = "windows"))]
            ButtonState::Signals(_) => Line::styled(" Select Signal ", styles.widget_title_style),
            ButtonState::Simple { .. } => {
                Line::styled(" Confirm Kill Process ", styles.widget_title_style)
            }
        };

        let block = dialog_block(styles.border_type)
            .title_top(title)
            .title_top(Line::styled(" Esc to close ", styles.widget_title_style).right_aligned())
            .style(styles.border_style)
            .border_style(styles.border_style);

        let num_lines = text.line_count(block.inner(draw_loc).width) as u16;

        match button_state {
            #[cfg(not(target_os = "windows"))]
            ButtonState::Signals(list_state) => {
                // A list of options, displayed vertically.
                const SIGNAL_TEXT_LEN: u16 = SIGNAL_TEXT.len() as u16;

                // Make the rect only as big as it needs to be, which is the height of the text,
                // the buttons, and up to 2 spaces (margin and space between).
                let draw_loc = Layout::vertical([Constraint::Max(num_lines + SIGNAL_TEXT_LEN + 2)])
                    .flex(Flex::Center)
                    .areas::<1>(draw_loc)[0];

                // // If there's enough room, add padding to the top.
                // if draw_loc.height > num_lines + 2 + 2 {
                //     block = block.padding(Padding::top(1));
                // }

                // Now we need to divide the block into one area for the paragraph,
                // and one for the buttons.
                let draw_locs = Layout::vertical([
                    Constraint::Max(num_lines),
                    Constraint::Max(SIGNAL_TEXT_LEN),
                ])
                .flex(Flex::SpaceAround)
                .areas::<2>(draw_loc);

                // Now render the text + block...
                f.render_widget(text.block(block), draw_locs[0]);

                // And the tricky part, rendering the buttons.
                let selected = list_state.selected().unwrap_or(0);

                let buttons = List::new(SIGNAL_TEXT.iter().enumerate().map(|(index, &signal)| {
                    let style = if index == selected {
                        styles.selected_text_style
                    } else {
                        styles.text_style
                    };

                    Span::styled(signal, style)
                }));

                // FIXME: I have no idea what will happen here...
                f.render_stateful_widget(buttons, draw_locs[0], list_state);
            }
            ButtonState::Simple { yes } => {
                // Just a yes/no, horizontally.

                // Make the rect only as big as it needs to be, which is the height of the text,
                // the buttons, and up to 3 spaces (margin and space between).
                let draw_loc = Layout::vertical([Constraint::Max(num_lines + 1 + 3)])
                    .flex(Flex::Center)
                    .areas::<1>(draw_loc)[0];

                // // If there's enough room, add padding.
                // if draw_loc.height > num_lines + 2 + 2 {
                //     block = block.padding(Padding::vertical(1));
                // }

                // Now we need to divide the block into one area for the paragraph,
                // and one for the buttons.
                let draw_locs =
                    Layout::vertical([Constraint::Max(num_lines), Constraint::Length(1)])
                        .flex(Flex::SpaceAround)
                        .areas::<2>(draw_loc);

                f.render_widget(text.block(block), draw_locs[0]);

                let (yes, no) = {
                    let (yes_style, no_style) = if *yes {
                        (styles.selected_text_style, styles.text_style)
                    } else {
                        (styles.text_style, styles.selected_text_style)
                    };

                    (
                        Paragraph::new(Span::styled("Yes", yes_style)),
                        Paragraph::new(Span::styled("No", no_style)),
                    )
                };

                let button_locs = Layout::horizontal([Constraint::Length(3 + 2); 2])
                    .flex(Flex::SpaceAround)
                    .areas::<2>(draw_locs[1]);

                f.render_widget(yes, button_locs[0]);
                f.render_widget(no, button_locs[1]);
            }
        }
    }

    #[inline]
    fn draw_no_button_dialog(
        &self, f: &mut Frame<'_>, draw_loc: Rect, styles: &Styles, text: Text<'_>, title: Line<'_>,
    ) {
        let text = Paragraph::new(text)
            .style(styles.text_style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let mut block = dialog_block(styles.border_type)
            .title_top(title)
            .title_top(Line::styled(" Esc to close ", styles.widget_title_style).right_aligned())
            .style(styles.border_style)
            .border_style(styles.border_style);

        let num_lines = text.line_count(block.inner(draw_loc).width) as u16;

        // Also calculate how big of a draw loc we actually need. For this
        // one, we want it to be shorter if possible.
        //
        // Note the +2 is for the margin, and another +2 for border.
        let draw_loc = Layout::vertical([Constraint::Max(num_lines + 2 + 2)])
            .flex(Flex::Center)
            .areas::<1>(draw_loc)[0];

        // If there's enough room, add padding. I think this is also faster than doing another Layout
        // for this case since there's just one object anyway.
        if draw_loc.height > num_lines + 2 + 2 {
            block = block.padding(Padding::vertical(1));
        }

        f.render_widget(text.block(block), draw_loc);
    }

    /// Draw the [`ProcessKillDialog`].
    pub fn draw(&mut self, f: &mut Frame<'_>, draw_loc: Rect, styles: &Styles) {
        // The idea is:
        // - Use as big of a dialog box as needed (within the maximal draw loc)
        //  - So the non-button ones are going to be smaller... probably
        //    whatever the height of the text is.
        //  - Meanwhile for the button one, it'll likely be full height if it's
        //    "advanced" kill.

        match &mut self.state {
            ProcessKillDialogState::NotEnabled => {}
            ProcessKillDialogState::Selecting(name, pids, button_state) => {
                // Draw a text box. If buttons are yes/no, fit it, otherwise, use max space.
                Self::draw_selecting(f, draw_loc, styles, name, pids, button_state);
            }
            ProcessKillDialogState::Killing(pids) => {
                // Only draw a text box the size of the text + any margins if possible
                let text = Text::from(format!("Killing {} processes...", pids.len()));
                let title = Line::styled(" Killing Process ", styles.widget_title_style);

                self.draw_no_button_dialog(f, draw_loc, styles, text, title);
            }
            ProcessKillDialogState::Error(err) => {
                let text = Text::from(vec![
                    "Failed to kill process:".into(),
                    err.clone().into(),
                    "Please press ENTER or ESC to close this dialog.".into(),
                ]);
                let title = Line::styled(" Error ", styles.widget_title_style);

                self.draw_no_button_dialog(f, draw_loc, styles, text, title);
            }
        }
    }
}
