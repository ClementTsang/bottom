//! A dialog box to handle killing processes.

use std::time::Instant;

use cfg_if::cfg_if;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
use tui::widgets::ListState;
use tui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Position, Rect},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::{
    canvas::drawing_utils::dialog_block, collection::processes::Pid, options::config::style::Styles,
};

// Configure signal text based on the target OS.
cfg_if! {
    if #[cfg(target_os = "linux")] {
        const DEFAULT_KILL_SIGNAL: usize = 15;
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
        const DEFAULT_KILL_SIGNAL: usize = 15;
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
        const DEFAULT_KILL_SIGNAL: usize = 15;
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
#[derive(Debug)]
pub(crate) enum ButtonState {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
    Signals {
        state: ListState,
        last_button_draw_area: Rect,
    },
    Simple {
        yes: bool,
        last_yes_button_area: Rect,
        last_no_button_area: Rect,
    },
}

#[derive(Debug)]
struct ProcessKillSelectingInner {
    process_name: String,
    pids: Vec<Pid>,
    button_state: ButtonState,
}

/// The current state of the process kill dialog.
#[derive(Default, Debug)]
enum ProcessKillDialogState {
    #[default]
    NotEnabled,
    Selecting(ProcessKillSelectingInner),
    Error {
        process_name: String,
        pid: Option<Pid>,
        err: String,
    },
}

/// Process kill dialog.
#[derive(Default, Debug)]
pub(crate) struct ProcessKillDialog {
    state: ProcessKillDialogState,
    last_char: Option<(char, Instant)>,
}

impl ProcessKillDialog {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        !(matches!(self.state, ProcessKillDialogState::NotEnabled))
    }

    pub fn on_esc(&mut self) {
        self.reset();
    }

    pub fn on_enter(&mut self) {
        // We do this to get around borrow issues.
        let mut current = ProcessKillDialogState::NotEnabled;
        std::mem::swap(&mut self.state, &mut current);

        if let ProcessKillDialogState::Selecting(state) = current {
            let process_name = state.process_name;
            let button_state = state.button_state;
            let pids = state.pids;

            match button_state {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
                ButtonState::Signals { state, .. } => {
                    use crate::utils::process_killer;

                    if let Some(selected) = state.selected() {
                        if selected != 0 {
                            // On Linux, we need to skip 32 and 33.
                            let signal = if cfg!(target_os = "linux")
                                && (selected == 32 || selected == 33)
                            {
                                selected + 2
                            } else {
                                selected
                            };

                            for pid in pids {
                                if let Err(err) =
                                    process_killer::kill_process_given_pid(pid, signal)
                                {
                                    self.state = ProcessKillDialogState::Error {
                                        process_name,
                                        pid: Some(pid),
                                        err: err.to_string(),
                                    };
                                    return;
                                }
                            }
                        }
                    }
                }
                ButtonState::Simple { yes, .. } => {
                    if yes {
                        cfg_if! {
                            if #[cfg(target_os = "windows")] {
                                use crate::utils::process_killer;

                                for pid in pids {
                                    if let Err(err) = process_killer::kill_process_given_pid(pid) {
                                        self.state = ProcessKillDialogState::Error { process_name, pid: Some(pid), err: err.to_string() };
                                        break;
                                    }
                                }
                            } else if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))] {
                                use crate::utils::process_killer;

                                for pid in pids {
                                    // Send a SIGTERM by default.
                                    if let Err(err) = process_killer::kill_process_given_pid(pid, DEFAULT_KILL_SIGNAL) {
                                        self.state = ProcessKillDialogState::Error { process_name, pid: Some(pid), err: err.to_string() };
                                        break;
                                    }
                                }
                            } else {
                                self.state = ProcessKillDialogState::Error { process_name, pid: None, err: "Killing processes is not supported on this platform.".into() };

                            }
                        }
                    }
                }
            }
        }

        // Fall through behaviour is just to close the dialog.
        self.last_char = None;
    }

    pub fn on_char(&mut self, c: char) {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        const MAX_KEY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

        match c {
            'h' => self.on_left_key(),
            'j' => self.on_down_key(),
            'k' => self.on_up_key(),
            'l' => self.on_right_key(),
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
                if let Some(value) = c.to_digit(10) {
                    if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
                        button_state: ButtonState::Signals { state, .. },
                        ..
                    }) = &mut self.state
                    {
                        if let Some((prev, last_press)) = self.last_char {
                            if prev.is_ascii_digit() && last_press.elapsed() <= MAX_KEY_TIMEOUT {
                                let current = state.selected().unwrap_or(0);
                                let new = {
                                    let new = current * 10 + value as usize;

                                    // Note that 32 and 33 are skipped on linux.
                                    if cfg!(target_os = "linux") {
                                        if new == 32 || new == 33 {
                                            value as usize
                                        } else if new >= 34 {
                                            new - 2
                                        } else {
                                            new
                                        }
                                    } else {
                                        new
                                    }
                                };

                                if new >= SIGNAL_TEXT.len() {
                                    // If the new value is too large, then just assume we instead want the value itself.
                                    state.select(Some(value as usize));
                                    self.last_char = Some((c, Instant::now()));
                                } else {
                                    state.select(Some(new));
                                    self.last_char = None;
                                }
                            } else {
                                state.select(Some(value as usize));
                                self.last_char = Some((c, Instant::now()));
                            }
                        } else {
                            state.select(Some(value as usize));
                            self.last_char = Some((c, Instant::now()));
                        }

                        return; // Needed to avoid accidentally clearing last_char.
                    }
                }
            }
            'g' => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
                {
                    if let Some(('g', last_press)) = self.last_char {
                        if last_press.elapsed() <= MAX_KEY_TIMEOUT {
                            self.go_to_first();
                            self.last_char = None;
                        } else {
                            self.last_char = Some(('g', Instant::now()));
                        }
                    } else {
                        self.last_char = Some(('g', Instant::now()));
                    }
                    return;
                }
            }
            'G' => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
                self.go_to_last();
            }
            _ => {}
        }

        self.last_char = None;
    }

    /// Handle a click at the given coordinates. Returns true if the click was
    /// handled, false otherwise.
    pub fn on_click(&mut self, x: u16, y: u16) -> bool {
        if let ProcessKillDialogState::Selecting(state) = &mut self.state {
            match &mut state.button_state {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
                ButtonState::Signals {
                    state,
                    last_button_draw_area,
                } => {
                    if last_button_draw_area.contains(Position { x, y }) {
                        let relative_y =
                            y.saturating_sub(last_button_draw_area.y) as usize + state.offset();
                        if relative_y < SIGNAL_TEXT.len() {
                            state.select(Some(relative_y));
                        }
                    }
                }
                ButtonState::Simple {
                    yes,
                    last_yes_button_area,
                    last_no_button_area,
                } => {
                    if last_yes_button_area.contains(Position { x, y }) {
                        *yes = true;
                    } else if last_no_button_area.contains(Position { x, y }) {
                        *yes = false;
                    }
                }
            }
        }

        false
    }

    /// Scroll up in the signal list.
    pub fn on_scroll_up(&mut self) {
        self.on_up_key();
    }

    /// Scroll down in the signal list.
    pub fn on_scroll_down(&mut self) {
        self.on_down_key();
    }

    /// Handle a left key press.
    pub fn on_left_key(&mut self) {
        self.last_char = None;

        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state: ButtonState::Simple { yes, .. },
            ..
        }) = &mut self.state
        {
            *yes = true;
        }
    }

    /// Handle a right key press.
    pub fn on_right_key(&mut self) {
        self.last_char = None;

        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state: ButtonState::Simple { yes, .. },
            ..
        }) = &mut self.state
        {
            *yes = false;
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
    fn scroll_up_by(state: &mut ListState, amount: usize) {
        if let Some(selected) = state.selected() {
            if let Some(new_position) = selected.checked_sub(amount) {
                state.select(Some(new_position));
            } else {
                state.select(Some(0));
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
    fn scroll_down_by(state: &mut ListState, amount: usize) {
        if let Some(selected) = state.selected() {
            let new_position = selected + amount;
            if new_position < SIGNAL_TEXT.len() {
                state.select(Some(new_position));
            } else {
                state.select(Some(SIGNAL_TEXT.len() - 1));
            }
        }
    }

    /// Handle an up key press.
    pub fn on_up_key(&mut self) {
        self.last_char = None;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state: ButtonState::Signals { state, .. },
            ..
        }) = &mut self.state
        {
            Self::scroll_up_by(state, 1);
        }
    }

    /// Handle a down key press.
    pub fn on_down_key(&mut self) {
        self.last_char = None;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state: ButtonState::Signals { state, .. },
            ..
        }) = &mut self.state
        {
            Self::scroll_down_by(state, 1);
        }
    }

    // Handle page up.
    pub fn on_page_up(&mut self) {
        self.last_char = None;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state:
                ButtonState::Signals {
                    state,
                    last_button_draw_area,
                    ..
                },
            ..
        }) = &mut self.state
        {
            Self::scroll_up_by(state, last_button_draw_area.height as usize);
        }
    }

    /// Handle page down.
    pub fn on_page_down(&mut self) {
        self.last_char = None;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state:
                ButtonState::Signals {
                    state,
                    last_button_draw_area,
                    ..
                },
            ..
        }) = &mut self.state
        {
            Self::scroll_down_by(state, last_button_draw_area.height as usize);
        }
    }

    pub fn go_to_first(&mut self) {
        self.last_char = None;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state: ButtonState::Signals { state, .. },
            ..
        }) = &mut self.state
        {
            state.select(Some(0));
        }
    }

    pub fn go_to_last(&mut self) {
        self.last_char = None;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            button_state: ButtonState::Signals { state, .. },
            ..
        }) = &mut self.state
        {
            state.select(Some(SIGNAL_TEXT.len() - 1));
        }
    }

    /// Enable the process kill process.
    pub fn start_process_kill(
        &mut self, process_name: String, pids: Vec<Pid>, use_simple_selection: bool,
    ) {
        let button_state = if use_simple_selection {
            ButtonState::Simple {
                yes: false,
                last_yes_button_area: Rect::default(),
                last_no_button_area: Rect::default(),
            }
        } else {
            cfg_if! {
                if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))] {
                    ButtonState::Signals { state: ListState::default().with_selected(Some(DEFAULT_KILL_SIGNAL)), last_button_draw_area: Rect::default() }
                } else {
                    ButtonState::Simple { yes: false, last_yes_button_area: Rect::default(), last_no_button_area: Rect::default()}
                }
            }
        };

        if pids.is_empty() {
            self.state = ProcessKillDialogState::Error {
                process_name,
                pid: None,
                err: "No PIDs found for the given process name.".into(),
            };
            return;
        }

        self.state = ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
            process_name,
            pids,
            button_state,
        });
    }

    pub fn handle_redraw(&mut self) {
        // FIXME: Not sure if we need this. We can probably handle this better in the draw function later.

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        {
            if let ProcessKillDialogState::Selecting(ProcessKillSelectingInner {
                button_state: ButtonState::Signals { state, .. },
                ..
            }) = &mut self.state
            {
                // Fix the button offset state when we do things like resize.
                *state.offset_mut() = 0;
            }
        }
    }

    #[inline]
    fn draw_selecting(
        f: &mut Frame<'_>, draw_area: Rect, styles: &Styles, state: &mut ProcessKillSelectingInner,
    ) {
        let ProcessKillSelectingInner {
            process_name,
            pids,
            button_state,
            ..
        } = state;

        // FIXME: Add some colour to this!
        let text = {
            const MAX_PROCESS_NAME_WIDTH: usize = 20;

            if let Some(first_pid) = pids.first() {
                let truncated_process_name =
                    unicode_ellipsis::truncate_str(process_name, MAX_PROCESS_NAME_WIDTH);

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

        let text: Paragraph<'_> = Paragraph::new(text)
            .style(styles.text_style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let title = match button_state {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
            ButtonState::Signals { .. } => {
                Line::styled(" Select Signal ", styles.widget_title_style)
            }
            ButtonState::Simple { .. } => {
                Line::styled(" Confirm Kill Process ", styles.widget_title_style)
            }
        };

        let block = dialog_block(styles.border_type)
            .title_top(title)
            .title_top(Line::styled(" Esc to close ", styles.widget_title_style).right_aligned())
            .style(styles.border_style)
            .border_style(styles.border_style);

        let num_lines = text.line_count(block.inner(draw_area).width) as u16;

        match button_state {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
            ButtonState::Signals {
                state,
                last_button_draw_area,
            } => {
                use tui::widgets::List;

                // A list of options, displayed vertically.
                const SIGNAL_TEXT_LEN: u16 = SIGNAL_TEXT.len() as u16;

                // Make the rect only as big as it needs to be, which is the height of the text,
                // the buttons, and up to 2 spaces (margin and space between), and the size of the block.
                let [draw_area] =
                    Layout::vertical([Constraint::Max(num_lines + SIGNAL_TEXT_LEN + 2 + 3)])
                        .flex(Flex::Center)
                        .areas(draw_area);

                // Now we need to divide the block into one area for the paragraph,
                // and one for the buttons.
                let [text_draw_area, button_draw_area] = Layout::vertical([
                    Constraint::Max(num_lines),
                    Constraint::Max(SIGNAL_TEXT_LEN),
                ])
                .flex(Flex::SpaceAround)
                .areas(block.inner(draw_area));

                // Render the block.
                f.render_widget(block, draw_area);

                // Now render the text.
                f.render_widget(text, text_draw_area);

                // And the tricky part, rendering the buttons.
                let selected = state
                    .selected()
                    .expect("the list state should always be initialized with a selection!");

                let buttons = List::new(SIGNAL_TEXT.iter().enumerate().map(|(index, &signal)| {
                    let style = if index == selected {
                        styles.selected_text_style
                    } else {
                        styles.text_style
                    };

                    Span::styled(signal, style)
                }));

                // This is kinda dumb how you have to set the constraint, but ok.
                const LONGEST_SIGNAL_TEXT_LENGTH: u16 = const {
                    let mut i = 0;
                    let mut max = 0;
                    while i < SIGNAL_TEXT.len() {
                        if SIGNAL_TEXT[i].len() > max {
                            max = SIGNAL_TEXT[i].len();
                        }
                        i += 1;
                    }

                    max as u16
                };
                let [button_draw_area] =
                    Layout::horizontal([Constraint::Length(LONGEST_SIGNAL_TEXT_LENGTH)])
                        .flex(Flex::Center)
                        .areas(button_draw_area);

                *last_button_draw_area = button_draw_area;
                f.render_stateful_widget(buttons, button_draw_area, state);
            }
            ButtonState::Simple {
                yes,
                last_yes_button_area,
                last_no_button_area,
            } => {
                // Make the rect only as big as it needs to be, which is the height of the text,
                // the buttons, and up to 3 spaces (margin and space between) + 2 for block.
                let [draw_area] = Layout::vertical([Constraint::Max(num_lines + 1 + 3 + 2)])
                    .flex(Flex::Center)
                    .areas(draw_area);

                // Now we need to divide the block into one area for the paragraph,
                // and one for the buttons.
                let [text_area, button_area] =
                    Layout::vertical([Constraint::Max(num_lines), Constraint::Length(1)])
                        .flex(Flex::SpaceAround)
                        .areas(block.inner(draw_area));

                // Render things, starting from the block.
                f.render_widget(block, draw_area);
                f.render_widget(text, text_area);

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

                let [yes_area, no_area] = Layout::horizontal([Constraint::Length(3); 2])
                    .flex(Flex::SpaceAround)
                    .areas(button_area);

                *last_yes_button_area = yes_area;
                *last_no_button_area = no_area;

                f.render_widget(yes, yes_area);
                f.render_widget(no, no_area);
            }
        }
    }

    #[inline]
    fn draw_no_button_dialog(
        &self, f: &mut Frame<'_>, draw_area: Rect, styles: &Styles, text: Text<'_>, title: Line<'_>,
    ) {
        let text = Paragraph::new(text)
            .style(styles.text_style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let block = dialog_block(styles.border_type)
            .title_top(title)
            .title_top(Line::styled(" Esc to close ", styles.widget_title_style).right_aligned())
            .style(styles.border_style)
            .border_style(styles.border_style);

        let num_lines = text.line_count(block.inner(draw_area).width) as u16;

        // Also calculate how big of a draw loc we actually need. For this
        // one, we want it to be shorter if possible.
        //
        // Note the +2 is for the margin, and another +2 for border.
        let [draw_area] = Layout::vertical([Constraint::Max(num_lines + 2 + 2)])
            .flex(Flex::Center)
            .areas(draw_area);

        let [text_draw_area] = Layout::vertical([Constraint::Length(num_lines)])
            .flex(Flex::Center)
            .areas(block.inner(draw_area));

        f.render_widget(block, draw_area);
        f.render_widget(text, text_draw_area);
    }

    /// Draw the [`ProcessKillDialog`].
    pub fn draw(&mut self, f: &mut Frame<'_>, draw_area: Rect, styles: &Styles) {
        // The idea is:
        // - Use as big of a dialog box as needed (within the maximal draw loc)
        //  - So the non-button ones are going to be smaller... probably
        //    whatever the height of the text is.
        //  - Meanwhile for the button one, it'll likely be full height if it's
        //    "advanced" kill.

        const MAX_DIALOG_WIDTH: u16 = 100;
        let [draw_area] = Layout::horizontal([Constraint::Max(MAX_DIALOG_WIDTH)])
            .flex(Flex::Center)
            .areas(draw_area);

        // FIXME: Add some colour to this!
        match &mut self.state {
            ProcessKillDialogState::NotEnabled => {}
            ProcessKillDialogState::Selecting(state) => {
                // Draw a text box. If buttons are yes/no, fit it, otherwise, use max space.
                Self::draw_selecting(f, draw_area, styles, state);
            }
            ProcessKillDialogState::Error {
                process_name,
                pid,
                err,
            } => {
                let text = Text::from(vec![
                    if let Some(pid) = pid {
                        format!("Failed to kill process {process_name} ({pid}):").into()
                    } else {
                        format!("Failed to kill process '{process_name}':").into()
                    },
                    err.to_owned().into(),
                    "Please press ENTER or ESC to close this dialog.".into(),
                ])
                .alignment(Alignment::Center);
                let title = Line::styled(" Error ", styles.widget_title_style);

                self.draw_no_button_dialog(f, draw_area, styles, text, title);
            }
        }
    }
}
