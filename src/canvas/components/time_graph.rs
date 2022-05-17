use std::borrow::Cow;

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    symbols::Marker,
    text::{Span, Spans},
    widgets::{Block, Borders, GraphType},
    Frame,
};

use concat_string::concat_string;
use unicode_segmentation::UnicodeSegmentation;

use super::{Axis, Dataset, TimeChart};

/// A single graph point.
pub type Point = (f64, f64);

/// Represents the data required by the [`TimeGraph`].
pub struct GraphData<'a> {
    pub points: &'a [Point],
    pub style: Style,
    pub name: Option<Cow<'a, str>>,
}

pub struct TimeGraph<'a> {
    /// Whether to use a dot marker over the default braille markers.
    pub use_dot: bool,

    /// The min and max x boundaries. Expects a f64 representing the time range in milliseconds.
    pub x_bounds: [u64; 2],

    /// Whether to hide the time/x-labels.
    pub hide_x_labels: bool,

    /// The min and max y boundaries.
    pub y_bounds: [f64; 2],

    /// Any y-labels.
    pub y_labels: &'a [Cow<'a, str>],

    /// The graph style.
    pub graph_style: Style,

    /// The border style.
    pub border_style: Style,

    /// The graph title.
    pub title: Cow<'a, str>,

    /// Whether this graph is expanded.
    pub is_expanded: bool,

    /// The title style.
    pub title_style: Style,

    /// Any legend constraints.
    pub legend_constraints: Option<(Constraint, Constraint)>,
}

impl<'a> TimeGraph<'a> {
    /// Generates the [`Axis`] for the x-axis.
    fn generate_x_axis(&self) -> Axis<'_> {
        // Due to how we display things, we need to adjust the time bound values.
        let time_start = -(self.x_bounds[1] as f64);
        let adjusted_x_bounds = [time_start, 0.0];

        if self.hide_x_labels {
            Axis::default().bounds(adjusted_x_bounds)
        } else {
            let x_labels = vec![
                Span::raw(concat_string!((self.x_bounds[1] / 1000).to_string(), "s")),
                Span::raw(concat_string!((self.x_bounds[0] / 1000).to_string(), "s")),
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
                    .map(|label| Span::raw(label.clone()))
                    .collect(),
            )
    }

    /// Generates a title for the [`TimeGraph`] widget, given the available space.
    fn generate_title(&self, draw_loc: Rect) -> Spans<'_> {
        if self.is_expanded {
            let title_base = concat_string!(self.title, "── Esc to go back ");
            Spans::from(vec![
                Span::styled(self.title.as_ref(), self.title_style),
                Span::styled(
                    concat_string!(
                        "─",
                        "─".repeat(usize::from(draw_loc.width).saturating_sub(
                            UnicodeSegmentation::graphemes(title_base.as_str(), true).count() + 2
                        )),
                        "─ Esc to go back "
                    ),
                    self.border_style,
                ),
            ])
        } else {
            Spans::from(Span::styled(self.title.as_ref(), self.title_style))
        }
    }

    /// Draws a time graph at [`Rect`] location provided by `draw_loc`. A time graph is used to display data points
    /// throughout time in the x-axis.
    ///
    /// This time graph:
    /// - Draws with the higher time value on the left, and lower on the right.
    /// - Expects a [`TimeGraph`] to be passed in, which details how to draw the graph.
    /// - Expects `graph_data`, which represents *what* data to draw, and various details like style and optional legends.
    pub fn draw_time_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, draw_loc: Rect, graph_data: &[GraphData<'_>],
    ) {
        let x_axis = self.generate_x_axis();
        let y_axis = self.generate_y_axis();

        // This is some ugly manual loop unswitching. Maybe unnecessary.
        let data = if self.use_dot {
            graph_data
                .iter()
                .map(|data| create_dataset(data, Marker::Dot))
                .collect()
        } else {
            graph_data
                .iter()
                .map(|data| create_dataset(data, Marker::Braille))
                .collect()
        };

        let block = Block::default()
            .title(self.generate_title(draw_loc))
            .borders(Borders::ALL)
            .border_style(self.border_style);

        f.render_widget(
            TimeChart::new(data)
                .block(block)
                .x_axis(x_axis)
                .y_axis(y_axis)
                .legend_style(self.graph_style)
                .hidden_legend_constraints(
                    self.legend_constraints
                        .unwrap_or(super::DEFAULT_LEGEND_CONSTRAINTS),
                ),
            draw_loc,
        )
    }
}

/// Creates a new [`Dataset`].
fn create_dataset<'a>(data: &'a GraphData<'a>, marker: Marker) -> Dataset<'a> {
    let GraphData {
        points,
        style,
        name,
    } = data;

    let dataset = Dataset::default()
        .style(*style)
        .data(points)
        .graph_type(GraphType::Line)
        .marker(marker);

    if let Some(name) = name {
        dataset.name(name.as_ref())
    } else {
        dataset
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use tui::{
        layout::Rect,
        style::{Color, Style},
        text::{Span, Spans},
    };

    use crate::canvas::components::Axis;

    use super::TimeGraph;

    const Y_LABELS: [Cow<'static, str>; 3] = [
        Cow::Borrowed("0%"),
        Cow::Borrowed("50%"),
        Cow::Borrowed("100%"),
    ];

    fn create_time_graph() -> TimeGraph<'static> {
        TimeGraph {
            title: " Network ".into(),
            use_dot: true,
            x_bounds: [0, 15000],
            hide_x_labels: false,
            y_bounds: [0.0, 100.5],
            y_labels: &Y_LABELS,
            graph_style: Style::default().fg(Color::Red),
            border_style: Style::default().fg(Color::Blue),
            is_expanded: false,
            title_style: Style::default().fg(Color::Cyan),
            legend_constraints: None,
        }
    }

    #[test]
    fn time_graph_gen_x_axis() {
        let tg = create_time_graph();

        let x_axis = tg.generate_x_axis();
        let actual = Axis::default()
            .bounds([-15000.0, 0.0])
            .labels(vec![Span::raw("15s"), Span::raw("0s")])
            .style(Style::default().fg(Color::Red));
        assert_eq!(x_axis.bounds, actual.bounds);
        assert_eq!(x_axis.labels, actual.labels);
        assert_eq!(x_axis.style, actual.style);
    }

    #[test]
    fn time_graph_gen_y_axis() {
        let tg = create_time_graph();

        let y_axis = tg.generate_y_axis();
        let actual = Axis::default()
            .bounds([0.0, 100.5])
            .labels(vec![Span::raw("0%"), Span::raw("50%"), Span::raw("100%")])
            .style(Style::default().fg(Color::Red));

        assert_eq!(y_axis.bounds, actual.bounds);
        assert_eq!(y_axis.labels, actual.labels);
        assert_eq!(y_axis.style, actual.style);
    }

    #[test]
    fn time_graph_gen_title() {
        let mut tg = create_time_graph();
        let draw_loc = Rect::new(0, 0, 32, 100);

        let title = tg.generate_title(draw_loc);
        assert_eq!(
            title,
            Spans::from(Span::styled(" Network ", Style::default().fg(Color::Cyan)))
        );

        tg.is_expanded = true;
        let title = tg.generate_title(draw_loc);
        assert_eq!(
            title,
            Spans::from(vec![
                Span::styled(" Network ", Style::default().fg(Color::Cyan)),
                Span::styled("───── Esc to go back ", Style::default().fg(Color::Blue))
            ])
        );
    }
}
