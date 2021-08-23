use std::time::{Duration, Instant};

use crossterm::event::{KeyEvent, KeyModifiers, MouseEvent};
use tui::layout::Rect;

use crate::app::{event::EventResult, Component};

#[derive(Clone)]
pub enum AutohideTimerState {
    Hidden,
    Running(Instant),
}

#[derive(Clone)]
pub enum AutohideTimer {
    Disabled,
    Enabled {
        state: AutohideTimerState,
        show_duration: Duration,
    },
}

impl AutohideTimer {
    fn trigger_display_timer(&mut self) {
        match self {
            AutohideTimer::Disabled => {
                // Does nothing.
            }
            AutohideTimer::Enabled {
                state,
                show_duration: _,
            } => {
                *state = AutohideTimerState::Running(Instant::now());
            }
        }
    }

    pub fn update_display_timer(&mut self) {
        match self {
            AutohideTimer::Disabled => {
                // Does nothing.
            }
            AutohideTimer::Enabled {
                state,
                show_duration,
            } => match state {
                AutohideTimerState::Hidden => {}
                AutohideTimerState::Running(trigger_instant) => {
                    if trigger_instant.elapsed() > *show_duration {
                        *state = AutohideTimerState::Hidden;
                    }
                }
            },
        }
    }
}

/// A graph widget with controllable time ranges along the x-axis.
pub struct TimeGraph {
    current_display_time: u64,
    autohide_timer: AutohideTimer,

    default_time_value: u64,

    min_duration: u64,
    max_duration: u64,
    time_interval: u64,

    bounds: Rect,
}

impl TimeGraph {
    pub fn new(
        start_value: u64, autohide_timer: AutohideTimer, min_duration: u64, max_duration: u64,
        time_interval: u64,
    ) -> Self {
        Self {
            current_display_time: start_value,
            autohide_timer,
            default_time_value: start_value,
            min_duration,
            max_duration,
            time_interval,
            bounds: Rect::default(),
        }
    }

    /// Handles a char `c`.
    fn handle_char(&mut self, c: char) -> EventResult {
        match c {
            '-' => self.zoom_out(),
            '+' => self.zoom_in(),
            '=' => self.reset_zoom(),
            _ => EventResult::NoRedraw,
        }
    }

    fn zoom_in(&mut self) -> EventResult {
        let new_time = self.current_display_time.saturating_sub(self.time_interval);

        if new_time >= self.min_duration {
            self.current_display_time = new_time;
            self.autohide_timer.trigger_display_timer();

            EventResult::Redraw
        } else if new_time != self.min_duration {
            self.current_display_time = self.min_duration;
            self.autohide_timer.trigger_display_timer();

            EventResult::Redraw
        } else {
            EventResult::NoRedraw
        }
    }

    fn zoom_out(&mut self) -> EventResult {
        let new_time = self.current_display_time + self.time_interval;

        if new_time <= self.max_duration {
            self.current_display_time = new_time;
            self.autohide_timer.trigger_display_timer();

            EventResult::Redraw
        } else if new_time != self.max_duration {
            self.current_display_time = self.max_duration;
            self.autohide_timer.trigger_display_timer();

            EventResult::Redraw
        } else {
            EventResult::NoRedraw
        }
    }

    fn reset_zoom(&mut self) -> EventResult {
        if self.current_display_time == self.default_time_value {
            EventResult::NoRedraw
        } else {
            self.current_display_time = self.default_time_value;
            self.autohide_timer.trigger_display_timer();
            EventResult::Redraw
        }
    }
}

impl Component for TimeGraph {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        use crossterm::event::KeyCode::Char;

        if event.modifiers == KeyModifiers::NONE || event.modifiers == KeyModifiers::SHIFT {
            match event.code {
                Char(c) => self.handle_char(c),
                _ => EventResult::NoRedraw,
            }
        } else {
            EventResult::NoRedraw
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        match event.kind {
            crossterm::event::MouseEventKind::ScrollDown => self.zoom_out(),
            crossterm::event::MouseEventKind::ScrollUp => self.zoom_in(),
            _ => EventResult::NoRedraw,
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}
