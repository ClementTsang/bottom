use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use crossterm::event::{KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    symbols::Marker,
    text::Span,
    widgets::{Block, GraphType},
};

use crate::{
    app::{
        event::EventResult,
        widgets::tui_widgets::{
            custom_legend_chart::{Axis, Dataset},
            TimeChart,
        },
        AppConfigFields, Component,
    },
    canvas::Painter,
    constants::{AUTOHIDE_TIMEOUT_MILLISECONDS, STALE_MAX_MILLISECONDS, STALE_MIN_MILLISECONDS},
};

#[derive(Clone)]
pub enum AutohideTimerState {
    Hidden,
    Running(Instant),
}

#[derive(Clone)]
pub enum AutohideTimer {
    AlwaysShow,
    AlwaysHide,
    Enabled {
        state: AutohideTimerState,
        show_duration: Duration,
    },
}

// TODO: [AUTOHIDE] Not a fan of how this is done, as this should really "trigger" a draw when it's done.
impl AutohideTimer {
    fn start_display_timer(&mut self) {
        match self {
            AutohideTimer::AlwaysShow | AutohideTimer::AlwaysHide => {
                // Do nothing.
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
            AutohideTimer::AlwaysShow | AutohideTimer::AlwaysHide => {
                // Do nothing.
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

    pub fn is_showing(&mut self) -> bool {
        self.update_display_timer();
        match self {
            AutohideTimer::AlwaysShow => true,
            AutohideTimer::AlwaysHide => false,
            AutohideTimer::Enabled {
                state,
                show_duration: _,
            } => match state {
                AutohideTimerState::Hidden => false,
                AutohideTimerState::Running(_) => true,
            },
        }
    }
}

pub struct TimeGraphData<'d> {
    pub data: &'d [(f64, f64)],
    pub label: Option<Cow<'static, str>>,
    pub style: Style,
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

    use_dot: bool,
}

impl TimeGraph {
    /// Creates a new [`TimeGraph`].  All time values are in milliseconds.
    pub fn new(
        start_value: u64, autohide_timer: AutohideTimer, min_duration: u64, max_duration: u64,
        time_interval: u64, use_dot: bool,
    ) -> Self {
        Self {
            current_display_time: start_value,
            autohide_timer,
            default_time_value: start_value,
            min_duration,
            max_duration,
            time_interval,
            bounds: Rect::default(),
            use_dot,
        }
    }

    /// Creates a new [`TimeGraph`] given an [`AppConfigFields`].
    pub fn from_config(app_config_fields: &AppConfigFields) -> Self {
        Self::new(
            app_config_fields.default_time_value,
            if app_config_fields.hide_time {
                AutohideTimer::AlwaysHide
            } else if app_config_fields.autohide_time {
                AutohideTimer::Enabled {
                    state: AutohideTimerState::Running(Instant::now()),
                    show_duration: Duration::from_millis(AUTOHIDE_TIMEOUT_MILLISECONDS),
                }
            } else {
                AutohideTimer::AlwaysShow
            },
            STALE_MIN_MILLISECONDS,
            STALE_MAX_MILLISECONDS,
            app_config_fields.time_interval,
            app_config_fields.use_dot,
        )
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
            self.autohide_timer.start_display_timer();

            EventResult::Redraw
        } else if new_time != self.min_duration {
            self.current_display_time = self.min_duration;
            self.autohide_timer.start_display_timer();

            EventResult::Redraw
        } else {
            EventResult::NoRedraw
        }
    }

    fn zoom_out(&mut self) -> EventResult {
        let new_time = self.current_display_time + self.time_interval;

        if new_time <= self.max_duration {
            self.current_display_time = new_time;
            self.autohide_timer.start_display_timer();

            EventResult::Redraw
        } else if new_time != self.max_duration {
            self.current_display_time = self.max_duration;
            self.autohide_timer.start_display_timer();

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
            self.autohide_timer.start_display_timer();
            EventResult::Redraw
        }
    }

    fn get_x_axis_labels(&self, painter: &Painter) -> Vec<Span<'_>> {
        vec![
            Span::styled(
                format!("{}s", self.current_display_time / 1000),
                painter.colours.graph_style,
            ),
            Span::styled("0s", painter.colours.graph_style),
        ]
    }

    /// Returns the current display time boundary.
    pub fn get_current_display_time(&self) -> u64 {
        self.current_display_time
    }

    /// Creates a [`Chart`].
    ///
    /// The `reverse_order` parameter is mostly used for cases where you want the first entry to be drawn on
    /// top - note that this will also reverse the naturally generated legend, if shown!
    pub fn draw_tui_chart<B: Backend>(
        &mut self, painter: &Painter, f: &mut tui::Frame<'_, B>, data: &'_ [TimeGraphData<'_>],
        y_bound_labels: &[Cow<'static, str>], y_bounds: [f64; 2], reverse_order: bool,
        block: Block<'_>, block_area: Rect,
    ) {
        let inner_area = block.inner(block_area);

        self.set_bounds(inner_area);

        let time_start = -(self.current_display_time as f64);
        let x_axis = {
            let x_axis = Axis::default()
                .bounds([time_start, 0.0])
                .style(painter.colours.graph_style);
            if self.autohide_timer.is_showing() {
                x_axis.labels(self.get_x_axis_labels(painter))
            } else {
                x_axis
            }
        };
        let y_axis = Axis::default()
            .bounds(y_bounds)
            .style(painter.colours.graph_style)
            .labels(
                y_bound_labels
                    .into_iter()
                    .map(|label| Span::styled(label.clone(), painter.colours.graph_style))
                    .collect(),
            );

        let mut datasets: Vec<Dataset<'_>> = data
            .iter()
            .map(|time_graph_data| {
                let mut dataset = Dataset::default()
                    .data(time_graph_data.data)
                    .style(time_graph_data.style)
                    .marker(if self.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .graph_type(GraphType::Line);

                if let Some(label) = &time_graph_data.label {
                    dataset = dataset.name(label.clone());
                }

                dataset
            })
            .collect();

        if reverse_order {
            datasets.reverse();
        }

        let chart = TimeChart::new(datasets)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .style(painter.colours.graph_style)
            .legend_style(painter.colours.graph_style)
            .hidden_legend_constraints((Constraint::Ratio(3, 4), Constraint::Ratio(3, 4)));

        f.render_widget(chart.block(block), block_area);
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
            MouseEventKind::ScrollDown => self.zoom_out(),
            MouseEventKind::ScrollUp => self.zoom_in(),
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
