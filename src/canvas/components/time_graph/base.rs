mod time_chart;
use std::{borrow::Cow, time::Instant};

use concat_string::concat_string;
pub use time_chart::*;
use tui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    symbols::Marker,
    text::{Line, Span},
    widgets::{BorderType, GraphType},
};

use crate::{app::data::Values, canvas::drawing_utils::widget_block};

/// Represents the data required by the [`TimeGraph`].
///
/// TODO: We may be able to get rid of this intermediary data structure.
#[derive(Default)]
pub(crate) struct GraphData<'a> {
    time: &'a [Instant],
    values: Option<&'a Values>,
    custom_values: Option<&'a [f64]>,
    style: Style,
    name: Option<Cow<'a, str>>,
    filled: bool,
    inverted: bool,
}

impl<'a> GraphData<'a> {
    pub fn time(mut self, time: &'a [Instant]) -> Self {
        self.time = time;
        self
    }

    pub fn values(mut self, values: &'a Values) -> Self {
        self.values = Some(values);
        self
    }

    pub fn custom_values(mut self, values: &'a [f64]) -> Self {
        self.custom_values = Some(values);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn name(mut self, name: Cow<'a, str>) -> Self {
        self.name = Some(name);
        self
    }

    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }
}

pub struct TimeGraph<'a> {
    /// The min x value.
    pub x_min: f64,

    /// Whether to hide the time/x-labels.
    pub hide_x_labels: bool,

    /// The min and max y boundaries.
    pub y_bounds: AxisBound,

    /// Any y-labels.
    pub y_labels: &'a [Cow<'a, str>],

    /// The graph style.
    pub graph_style: Style,

    /// The border style.
    pub border_style: Style,

    /// The border type.
    pub border_type: BorderType,

    /// The graph title.
    pub title: Cow<'a, str>,

    /// Whether this graph is selected.
    pub is_selected: bool,

    /// Whether this graph is expanded.
    pub is_expanded: bool,

    /// The title style.
    pub title_style: Style,

    /// The legend position.
    pub legend_position: Option<LegendPosition>,

    /// Any legend constraints.
    pub legend_constraints: Option<(Constraint, Constraint)>,

    /// The marker type. Unlike ratatui's native charts, we assume
    /// only a single type of marker.
    pub marker: Marker,

    /// The chart scaling.
    pub scaling: ChartScaling,

    /// The borders to draw.
    pub borders: tui::widgets::Borders,
}

impl TimeGraph<'_> {
    /// Generates the [`Axis`] for the x-axis.
    fn generate_x_axis(&self) -> Axis<'_> {
        // Due to how we display things, we need to adjust the time bound values.
        let adjusted_x_bounds = AxisBound::Min(self.x_min);

        if self.hide_x_labels {
            Axis::default().bounds(adjusted_x_bounds)
        } else {
            let x_bound_left = ((-self.x_min) as u64 / 1000).to_string();
            let x_bound_right = "0s";

            let x_labels = vec![
                Span::styled(concat_string!(x_bound_left, "s"), self.graph_style),
                Span::styled(x_bound_right, self.graph_style),
            ];

            Axis::default()
                .bounds(adjusted_x_bounds)
                .labels(x_labels)
                .style(self.graph_style)
        }
    }

    /// Generates the [`Axis`] for the y-axis.
    fn generate_y_axis(&self) -> Axis<'_> {
        Axis::default()
            .bounds(self.y_bounds)
            .style(self.graph_style)
            .labels(
                self.y_labels
                    .iter()
                    .map(|label| Span::styled(label.clone(), self.graph_style))
                    .collect(),
            )
    }

    /// Draws a time graph at [`Rect`] location provided by `draw_loc`. A time
    /// graph is used to display data points throughout time in the x-axis.
    ///
    /// This time graph:
    /// - Draws with the higher time value on the left, and lower on the right.
    /// - Expects a [`TimeGraph`] to be passed in, which details how to draw the
    ///   graph.
    /// - Expects `graph_data`, which represents *what* data to draw, and
    ///   various details like style and optional legends.
    pub fn draw(&self, f: &mut Frame<'_>, draw_loc: Rect, graph_data: Vec<GraphData<'_>>) {
        // TODO: (points_rework_v1) can we reduce allocations in the underlying graph by saving some sort of state?

        let x_axis = self.generate_x_axis();
        let y_axis = self.generate_y_axis();
        let data = graph_data.into_iter().map(create_dataset).collect();

        let block = {
            let mut b = widget_block(false, self.is_selected, self.border_type)
                .border_style(self.border_style)
                .borders(self.borders)
                .title_top(Line::styled(self.title.as_ref(), self.title_style));

            if self.is_expanded {
                b = b.title_top(Line::styled(" Esc to go back ", self.title_style).right_aligned())
            }

            b
        };

        f.render_widget(
            TimeChart::new(data)
                .block(block)
                .x_axis(x_axis)
                .y_axis(y_axis)
                .marker(self.marker)
                .legend_style(self.graph_style)
                .legend_position(self.legend_position)
                .hidden_legend_constraints(
                    self.legend_constraints
                        .unwrap_or(DEFAULT_LEGEND_CONSTRAINTS),
                )
                .scaling(self.scaling),
            draw_loc,
        )
    }
}

/// Creates a new [`Dataset`].
fn create_dataset(data: GraphData<'_>) -> Dataset<'_> {
    let GraphData {
        time,
        values,
        custom_values,
        style,
        name,
        filled,
        inverted,
    } = data;

    let Some(values) = values else {
        if let Some(custom) = custom_values {
            let dataset = Dataset::default()
                .style(style)
                .data_custom(time, custom)
                .graph_type(GraphType::Line)
                .filled(filled)
                .inverted(inverted);

            return if let Some(name) = name {
                dataset.name(name)
            } else {
                dataset
            };
        }
        return Dataset::default();
    };

    let dataset = Dataset::default()
        .style(style)
        .data(time, values)
        .graph_type(GraphType::Line)
        .graph_type(GraphType::Line)
        .filled(filled)
        .inverted(inverted);

    let dataset = if let Some(name) = name {
        dataset.name(name)
    } else {
        dataset
    };

    dataset
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use tui::{
        style::{Color, Style},
        symbols::Marker,
        text::Span,
        widgets::BorderType,
    };

    use super::{AxisBound, ChartScaling, TimeGraph};
    use crate::canvas::components::time_graph::Axis;

    const Y_LABELS: [Cow<'static, str>; 3] = [
        Cow::Borrowed("0%"),
        Cow::Borrowed("50%"),
        Cow::Borrowed("100%"),
    ];

    fn create_time_graph() -> TimeGraph<'static> {
        TimeGraph {
            title: " Network ".into(),
            x_min: -15000.0,
            hide_x_labels: false,
            y_bounds: AxisBound::Max(100.5),
            y_labels: &Y_LABELS,
            graph_style: Style::default().fg(Color::Red),
            border_style: Style::default().fg(Color::Blue),
            border_type: BorderType::Plain,
            is_selected: false,
            is_expanded: false,
            title_style: Style::default().fg(Color::Cyan),
            legend_position: None,
            legend_constraints: None,
            marker: Marker::Braille,
            scaling: ChartScaling::Linear,
            borders: tui::widgets::Borders::ALL,
        }
    }

    #[test]
    fn time_graph_gen_x_axis() {
        let tg = create_time_graph();
        let style = Style::default().fg(Color::Red);
        let x_axis = tg.generate_x_axis();

        let actual = Axis::default()
            .bounds(AxisBound::Min(-15000.0))
            .labels(vec![Span::styled("15s", style), Span::styled("0s", style)])
            .style(style);
        assert_eq!(x_axis.bounds, actual.bounds);
        assert_eq!(x_axis.labels, actual.labels);
        assert_eq!(x_axis.style, actual.style);
    }

    #[test]
    fn time_graph_gen_y_axis() {
        let tg = create_time_graph();
        let style = Style::default().fg(Color::Red);
        let y_axis = tg.generate_y_axis();

        let actual = Axis::default()
            .bounds(AxisBound::Max(100.5))
            .labels(vec![
                Span::styled("0%", style),
                Span::styled("50%", style),
                Span::styled("100%", style),
            ])
            .style(style);

        assert_eq!(y_axis.bounds, actual.bounds);
        assert_eq!(y_axis.labels, actual.labels);
        assert_eq!(y_axis.style, actual.style);
    }
}
