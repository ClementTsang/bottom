use std::time::Instant;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style as TuiStyle,
    text::Span,
    widgets::{Axis, Chart, Dataset},
    Frame,
};

use crate::{
    constants::DEFAULT_TIME_MILLISECONDS,
    drawing::{Event, EventStatus, Node, Point, Widget},
};

/// Style for a [`TimeGraph`].
#[derive(Default)]
pub struct Style {
    x_axis: TuiStyle,
    y_axis: TuiStyle,
    graph: TuiStyle,
}

/// Represents how to display the time (x-axis).
pub enum TimeStatus {
    /// Disable the time, always hiding it.
    Disabled,

    /// Enable the time, always showing it.
    Enabled,

    /// Show until the duration elapsed from the `start_instant` exceeds `end_time_ms`.
    Timed {
        start_instant: Instant,
        end_time_ms: u128,
    },
}

/// Represents the state of a [`TimeGraph`].
pub struct State {
    /// The current start of the time range in milliseconds.  Defaults to 60s, or 60 * 1000ms.
    time_start: u64,

    /// Represents how we should display the x-axis.
    time_status: TimeStatus,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time_start: DEFAULT_TIME_MILLISECONDS,
            time_status: TimeStatus::Enabled,
        }
    }
}

/// A [`TimeGraph`] serves as a chart that uses time in the x-axis, and allows setting
/// a custom y-axis range and legend.
///
/// A [`TimeGraph`] also supports handling zoom for time ranges as an event.
pub struct TimeGraph<'a> {
    state: &'a mut State,
    data: &'a [(&'a [Point], TuiStyle, &'a str)],
    y_bounds: &'a [f64; 2],
    y_labels: &'a [String],
    style: Style,

    width: Constraint,
    height: Constraint,
}

impl<'a> TimeGraph<'a> {
    /// Creates a new [`TimeGraph`].
    pub fn new(
        state: &'a mut State, data: &'a [(&'a [Point], TuiStyle, &'a str)], y_bounds: &'a [f64; 2],
        y_labels: &'a [String],
    ) -> Self {
        Self {
            state,
            data,
            y_bounds,
            y_labels,
            style: Style::default(),
            width: Constraint::Min(0),
            height: Constraint::Min(0),
        }
    }

    /// Sets the style of the [`TimeGraph`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the width of the [`TimeGraph`].

    pub fn width(mut self, width: Constraint) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`TimeGraph`].

    pub fn height(mut self, height: Constraint) -> Self {
        self.height = height;
        self
    }
}

impl<'a, B: Backend> Widget<B> for TimeGraph<'a> {
    fn draw(&mut self, ctx: &mut Frame<'_, B>, node: &'_ Node) {
        let time_start = self.state.time_start as f64;

        let x_axis = {
            let enabled = match self.state.time_status {
                TimeStatus::Disabled => false,
                TimeStatus::Enabled => true,
                TimeStatus::Timed {
                    start_instant,
                    end_time_ms,
                } => {
                    if start_instant.elapsed().as_millis() > end_time_ms {
                        self.state.time_status = TimeStatus::Disabled;
                        false
                    } else {
                        true
                    }
                }
            };

            if enabled {
                Axis::default()
                    .bounds([-time_start, 0.0])
                    .style(self.style.graph)
                    .labels(vec![
                        Span::styled(format!("{}s", self.state.time_start), self.style.x_axis),
                        Span::styled("0s", self.style.x_axis),
                    ])
            } else {
                Axis::default()
            }
        };

        let y_axis = Axis::default().bounds(*self.y_bounds).labels(
            self.y_labels
                .iter()
                .map(|label| Span::styled(label, self.style.y_axis))
                .collect(),
        );

        let interpolated_points = self
            .data
            .into_iter()
            .filter_map(|(dataset, style, _)| {
                if let Some(end_pos) = dataset.iter().position(|(time, _data)| *time >= time_start)
                {
                    if end_pos > 0 {
                        let old = dataset[end_pos - 1];
                        let new = dataset[end_pos];

                        let interpolated_point = interpolate_points(&old, &new, time_start);

                        return Some((style, [(time_start, interpolated_point), new]));
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        let mut datasets: Vec<Dataset<'_>> = self
            .data
            .iter()
            .map(|(dataset, style, name)| {
                Dataset::default()
                    .data(dataset)
                    .style(*style)
                    .name(*name)
                    .graph_type(tui::widgets::GraphType::Line)
            })
            .collect();

        for (style, interpolated_point) in &interpolated_points {
            datasets.push(
                Dataset::default()
                    .data(interpolated_point)
                    .style(*(*style))
                    .graph_type(tui::widgets::GraphType::Line),
            );
        }

        ctx.render_widget(
            Chart::new(datasets).x_axis(x_axis).y_axis(y_axis),
            node.bounds(),
        );
    }

    fn layout(&self, bounds: Rect) -> Node {
        Node::new(bounds, vec![])
    }

    fn width(&self) -> Constraint {
        self.width
    }

    fn height(&self) -> Constraint {
        self.height
    }

    fn on_event(&mut self, event: Event) -> EventStatus {
        crate::drawing::EventStatus::Ignored
        // match event {
        //     Event::Mouse(event) => todo!(),
        //     Event::Keyboard(event) => todo!(),
        // }
    }
}

/// Interpolates between two points.  Mainly used to help fill in tui-rs blanks in certain situations.
/// It is expected point_one is "further left" compared to point_two.
/// A point is two floats, in (x, y) form.  x is time, y is value.
pub fn interpolate_points(point_one: &(f64, f64), point_two: &(f64, f64), time: f64) -> f64 {
    let delta_x = point_two.0 - point_one.0;
    let delta_y = point_two.1 - point_one.1;
    let slope = delta_y / delta_x;

    (point_one.1 + (time - point_one.0) * slope).max(0.0)
}
