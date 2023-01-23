mod canvas;

use std::{borrow::Cow, cmp::max};

use canvas::*;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    symbols::{self, Marker},
    text::{Span, Spans},
    widgets::{
        canvas::{Line, Points},
        Block, Borders, GraphType, Widget,
    },
};
use unicode_width::UnicodeWidthStr;

use crate::utils::gen_util::partial_ordering;

/// A single graph point.
pub type Point = (f64, f64);

/// An X or Y axis for the chart widget
#[derive(Debug, Clone)]
pub struct Axis<'a> {
    /// Title displayed next to axis end
    pub title: Option<Spans<'a>>,
    /// Bounds for the axis (all data points outside these limits will not be represented)
    pub bounds: [f64; 2],
    /// A list of labels to put to the left or below the axis
    pub labels: Option<Vec<Span<'a>>>,
    /// The style used to draw the axis itself - NOT The labels.
    pub style: Style,
}

impl<'a> Default for Axis<'a> {
    fn default() -> Axis<'a> {
        Axis {
            title: None,
            bounds: [0.0, 0.0],
            labels: None,
            style: Default::default(),
        }
    }
}

#[allow(dead_code)]
impl<'a> Axis<'a> {
    pub fn title<T>(mut self, title: T) -> Axis<'a>
    where
        T: Into<Spans<'a>>,
    {
        self.title = Some(title.into());
        self
    }

    pub fn bounds(mut self, bounds: [f64; 2]) -> Axis<'a> {
        self.bounds = bounds;
        self
    }

    pub fn labels(mut self, labels: Vec<Span<'a>>) -> Axis<'a> {
        self.labels = Some(labels);
        self
    }

    pub fn style(mut self, style: Style) -> Axis<'a> {
        self.style = style;
        self
    }
}

/// A group of data points
#[derive(Debug, Clone)]
pub struct Dataset<'a> {
    /// Name of the dataset (used in the legend if shown)
    name: Cow<'a, str>,
    /// A reference to the actual data
    data: &'a [Point],
    /// Determines graph type used for drawing points
    graph_type: GraphType,
    /// Style used to plot this dataset
    style: Style,
}

impl<'a> Default for Dataset<'a> {
    fn default() -> Dataset<'a> {
        Dataset {
            name: Cow::from(""),
            data: &[],
            graph_type: GraphType::Scatter,
            style: Style::default(),
        }
    }
}

#[allow(dead_code)]
impl<'a> Dataset<'a> {
    pub fn name<S>(mut self, name: S) -> Dataset<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        self.name = name.into();
        self
    }

    pub fn data(mut self, data: &'a [Point]) -> Dataset<'a> {
        self.data = data;
        self
    }

    pub fn graph_type(mut self, graph_type: GraphType) -> Dataset<'a> {
        self.graph_type = graph_type;
        self
    }

    pub fn style(mut self, style: Style) -> Dataset<'a> {
        self.style = style;
        self
    }
}

/// A container that holds all the infos about where to display each elements of the chart (axis,
/// labels, legend, ...).
#[derive(Default, Debug, Clone, PartialEq)]
struct ChartLayout {
    /// Location of the title of the x axis
    title_x: Option<(u16, u16)>,
    /// Location of the title of the y axis
    title_y: Option<(u16, u16)>,
    /// Location of the first label of the x axis
    label_x: Option<u16>,
    /// Location of the first label of the y axis
    label_y: Option<u16>,
    /// Y coordinate of the horizontal axis
    axis_x: Option<u16>,
    /// X coordinate of the vertical axis
    axis_y: Option<u16>,
    /// Area of the legend
    legend_area: Option<Rect>,
    /// Area of the graph
    graph_area: Rect,
}

/// A "custom" chart, just a slightly tweaked [`tui::widgets::Chart`] from tui-rs, but with greater control over the
/// legend, and built with the idea of drawing data points relative to a time-based x-axis.
///
/// Main changes:
/// - Styling option for the legend box
/// - Automatically trimming out redundant draws in the x-bounds.
/// - Automatic interpolation to points that fall *just* outside of the screen.
///
/// TODO: Support for putting the legend on the left side.
#[derive(Debug, Clone)]
pub struct TimeChart<'a> {
    /// A block to display around the widget eventually
    block: Option<Block<'a>>,
    /// The horizontal axis
    x_axis: Axis<'a>,
    /// The vertical axis
    y_axis: Axis<'a>,
    /// A reference to the datasets
    datasets: Vec<Dataset<'a>>,
    /// The widget base style
    style: Style,
    /// The legend's style
    legend_style: Style,
    /// Constraints used to determine whether the legend should be shown or not
    hidden_legend_constraints: (Constraint, Constraint),
    /// The marker type.
    marker: Marker,
}

pub const DEFAULT_LEGEND_CONSTRAINTS: (Constraint, Constraint) =
    (Constraint::Ratio(1, 4), Constraint::Length(4));

#[allow(dead_code)]
impl<'a> TimeChart<'a> {
    /// Creates a new [`TimeChart`].
    ///
    /// **Note:** `datasets` **must** be sorted!
    pub fn new(datasets: Vec<Dataset<'a>>) -> TimeChart<'a> {
        TimeChart {
            block: None,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            style: Default::default(),
            legend_style: Default::default(),
            datasets,
            hidden_legend_constraints: DEFAULT_LEGEND_CONSTRAINTS,
            marker: Marker::Braille,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> TimeChart<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> TimeChart<'a> {
        self.style = style;
        self
    }

    pub fn legend_style(mut self, legend_style: Style) -> TimeChart<'a> {
        self.legend_style = legend_style;
        self
    }

    pub fn x_axis(mut self, axis: Axis<'a>) -> TimeChart<'a> {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis<'a>) -> TimeChart<'a> {
        self.y_axis = axis;
        self
    }

    pub fn marker(mut self, marker: Marker) -> TimeChart<'a> {
        self.marker = marker;
        self
    }

    /// Set the constraints used to determine whether the legend should be shown or not.
    pub fn hidden_legend_constraints(
        mut self, constraints: (Constraint, Constraint),
    ) -> TimeChart<'a> {
        self.hidden_legend_constraints = constraints;
        self
    }

    /// Compute the internal layout of the chart given the area. If the area is too small some
    /// elements may be automatically hidden
    fn layout(&self, area: Rect) -> ChartLayout {
        let mut layout = ChartLayout::default();
        if area.height == 0 || area.width == 0 {
            return layout;
        }
        let mut x = area.left();
        let mut y = area.bottom() - 1;

        if self.x_axis.labels.is_some() && y > area.top() {
            layout.label_x = Some(y);
            y -= 1;
        }

        layout.label_y = self.y_axis.labels.as_ref().and(Some(x));
        x += self.max_width_of_labels_left_of_y_axis(area);

        if self.x_axis.labels.is_some() && y > area.top() {
            layout.axis_x = Some(y);
            y -= 1;
        }

        if self.y_axis.labels.is_some() && x + 1 < area.right() {
            layout.axis_y = Some(x);
            x += 1;
        }

        if x < area.right() && y > 1 {
            layout.graph_area = Rect::new(x, area.top(), area.right() - x, y - area.top() + 1);
        }

        if let Some(ref title) = self.x_axis.title {
            let w = title.width() as u16;
            if w < layout.graph_area.width && layout.graph_area.height > 2 {
                layout.title_x = Some((x + layout.graph_area.width - w, y));
            }
        }

        if let Some(ref title) = self.y_axis.title {
            let w = title.width() as u16;
            if w + 1 < layout.graph_area.width && layout.graph_area.height > 2 {
                layout.title_y = Some((x, area.top()));
            }
        }

        if let Some(inner_width) = self.datasets.iter().map(|d| d.name.width() as u16).max() {
            let legend_width = inner_width + 2;
            let legend_height = self.datasets.len() as u16 + 2;
            let max_legend_width = self
                .hidden_legend_constraints
                .0
                .apply(layout.graph_area.width);
            let max_legend_height = self
                .hidden_legend_constraints
                .1
                .apply(layout.graph_area.height);
            if inner_width > 0
                && legend_width < max_legend_width
                && legend_height < max_legend_height
            {
                layout.legend_area = Some(Rect::new(
                    layout.graph_area.right() - legend_width,
                    layout.graph_area.top(),
                    legend_width,
                    legend_height,
                ));
            }
        }
        layout
    }

    fn max_width_of_labels_left_of_y_axis(&self, area: Rect) -> u16 {
        let mut max_width = self
            .y_axis
            .labels
            .as_ref()
            .map(|l| l.iter().map(Span::width).max().unwrap_or_default() as u16)
            .unwrap_or_default();
        if let Some(ref x_labels) = self.x_axis.labels {
            if !x_labels.is_empty() {
                max_width = max(max_width, x_labels[0].content.width() as u16);
            }
        }
        // labels of y axis and first label of x axis can take at most 1/3rd of the total width
        max_width.min(area.width / 3)
    }

    fn render_x_labels(
        &mut self, buf: &mut Buffer, layout: &ChartLayout, chart_area: Rect, graph_area: Rect,
    ) {
        let y = match layout.label_x {
            Some(y) => y,
            None => return,
        };
        let labels = self.x_axis.labels.as_ref().unwrap();
        let labels_len = labels.len() as u16;
        if labels_len < 2 {
            return;
        }
        let width_between_ticks = graph_area.width / (labels_len - 1);
        for (i, label) in labels.iter().enumerate() {
            let label_width = label.width() as u16;
            let label_width = if i == 0 {
                // the first label is put between the left border of the chart and the y axis.
                graph_area
                    .left()
                    .saturating_sub(chart_area.left())
                    .min(label_width)
            } else {
                // other labels are put on the left of each tick on the x axis
                width_between_ticks.min(label_width)
            };
            buf.set_span(
                graph_area.left() + i as u16 * width_between_ticks - label_width,
                y,
                label,
                label_width,
            );
        }
    }

    fn render_y_labels(
        &mut self, buf: &mut Buffer, layout: &ChartLayout, chart_area: Rect, graph_area: Rect,
    ) {
        let x = match layout.label_y {
            Some(x) => x,
            None => return,
        };
        let labels = self.y_axis.labels.as_ref().unwrap();
        let labels_len = labels.len() as u16;
        let label_width = graph_area.left().saturating_sub(chart_area.left());
        for (i, label) in labels.iter().enumerate() {
            let dy = i as u16 * (graph_area.height - 1) / (labels_len - 1);
            if dy < graph_area.bottom() {
                buf.set_span(x, graph_area.bottom() - 1 - dy, label, label_width);
            }
        }
    }
}

impl<'a> Widget for TimeChart<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }
        buf.set_style(area, self.style);
        // Sample the style of the entire widget. This sample will be used to reset the style of
        // the cells that are part of the components put on top of the graph area (i.e legend and
        // axis names).
        let original_style = buf.get(area.left(), area.top()).style();

        let chart_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let layout = self.layout(chart_area);
        let graph_area = layout.graph_area;
        if graph_area.width < 1 || graph_area.height < 1 {
            return;
        }

        self.render_x_labels(buf, &layout, chart_area, graph_area);
        self.render_y_labels(buf, &layout, chart_area, graph_area);

        if let Some(y) = layout.axis_x {
            for x in graph_area.left()..graph_area.right() {
                buf.get_mut(x, y)
                    .set_symbol(symbols::line::HORIZONTAL)
                    .set_style(self.x_axis.style);
            }
        }

        if let Some(x) = layout.axis_y {
            for y in graph_area.top()..graph_area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(symbols::line::VERTICAL)
                    .set_style(self.y_axis.style);
            }
        }

        if let Some(y) = layout.axis_x {
            if let Some(x) = layout.axis_y {
                buf.get_mut(x, y)
                    .set_symbol(symbols::line::BOTTOM_LEFT)
                    .set_style(self.x_axis.style);
            }
        }

        Canvas::default()
            .background_color(self.style.bg.unwrap_or(Color::Reset))
            .x_bounds(self.x_axis.bounds)
            .y_bounds(self.y_axis.bounds)
            .marker(self.marker)
            .paint(|ctx| {
                // Idea is to:
                // - Go over all datasets, determine *where* a point will be drawn.
                // - We take the topmost (last) point first.
                // - After we determine all points, then we paint them all.
                // This helps relieve the issue where normally, braille grids are painted via |=, when we want
                // an exclusive replacement.

                for dataset in &self.datasets {
                    let color = dataset.style.fg.unwrap_or(Color::Reset);

                    let start_bound = self.x_axis.bounds[0];
                    let end_bound = self.x_axis.bounds[1];

                    let (start_index, interpolate_start) = get_start(dataset, start_bound);
                    let (end_index, interpolate_end) = get_end(dataset, end_bound);

                    let data_slice = &dataset.data[start_index..end_index];

                    if let Some(interpolate_start) = interpolate_start {
                        if let (Some(older_point), Some(newer_point)) = (
                            dataset.data.get(interpolate_start),
                            dataset.data.get(interpolate_start + 1),
                        ) {
                            let interpolated_point = (
                                self.x_axis.bounds[0],
                                interpolate_point(older_point, newer_point, self.x_axis.bounds[0]),
                            );

                            if let GraphType::Line = dataset.graph_type {
                                ctx.draw(&Line {
                                    x1: interpolated_point.0,
                                    y1: interpolated_point.1,
                                    x2: newer_point.0,
                                    y2: newer_point.1,
                                    color,
                                });
                            } else {
                                ctx.draw(&Points {
                                    coords: &[interpolated_point],
                                    color,
                                });
                            }
                        }
                    }

                    if let GraphType::Line = dataset.graph_type {
                        for data in data_slice.windows(2) {
                            ctx.draw(&Line {
                                x1: data[0].0,
                                y1: data[0].1,
                                x2: data[1].0,
                                y2: data[1].1,
                                color,
                            });
                        }
                    } else {
                        ctx.draw(&Points {
                            coords: data_slice,
                            color,
                        });
                    }

                    if let Some(interpolate_end) = interpolate_end {
                        if let (Some(older_point), Some(newer_point)) = (
                            dataset.data.get(interpolate_end - 1),
                            dataset.data.get(interpolate_end),
                        ) {
                            let interpolated_point = (
                                self.x_axis.bounds[1],
                                interpolate_point(older_point, newer_point, self.x_axis.bounds[1]),
                            );

                            if let GraphType::Line = dataset.graph_type {
                                ctx.draw(&Line {
                                    x1: older_point.0,
                                    y1: older_point.1,
                                    x2: interpolated_point.0,
                                    y2: interpolated_point.1,
                                    color,
                                });
                            } else {
                                ctx.draw(&Points {
                                    coords: &[interpolated_point],
                                    color,
                                });
                            }
                        }
                    }
                }
            })
            .render(graph_area, buf);

        if let Some(legend_area) = layout.legend_area {
            buf.set_style(legend_area, original_style);
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.legend_style)
                .render(legend_area, buf);
            for (i, dataset) in self.datasets.iter().enumerate() {
                buf.set_string(
                    legend_area.x + 1,
                    legend_area.y + 1 + i as u16,
                    &dataset.name,
                    dataset.style,
                );
            }
        }

        if let Some((x, y)) = layout.title_x {
            let title = self.x_axis.title.unwrap();
            let width = graph_area.right().saturating_sub(x);
            buf.set_style(
                Rect {
                    x,
                    y,
                    width,
                    height: 1,
                },
                original_style,
            );
            buf.set_spans(x, y, &title, width);
        }

        if let Some((x, y)) = layout.title_y {
            let title = self.y_axis.title.unwrap();
            let width = graph_area.right().saturating_sub(x);
            buf.set_style(
                Rect {
                    x,
                    y,
                    width,
                    height: 1,
                },
                original_style,
            );
            buf.set_spans(x, y, &title, width);
        }
    }
}

/// Returns the start index and potential interpolation index given the start time and the dataset.
fn get_start(dataset: &Dataset<'_>, start_bound: f64) -> (usize, Option<usize>) {
    match dataset
        .data
        .binary_search_by(|(x, _y)| partial_ordering(x, &start_bound))
    {
        Ok(index) => (index, None),
        Err(index) => (index, index.checked_sub(1)),
    }
}

/// Returns the end position and potential interpolation index given the end time and the dataset.
fn get_end(dataset: &Dataset<'_>, end_bound: f64) -> (usize, Option<usize>) {
    match dataset
        .data
        .binary_search_by(|(x, _y)| partial_ordering(x, &end_bound))
    {
        // In the success case, this means we found an index. Add one since we want to include this index and we
        // expect to use the returned index as part of a (m..n) range.
        Ok(index) => (index.saturating_add(1), None),
        // In the fail case, this means we did not find an index, and the returned index is where one would *insert*
        // the location. This index is where one would insert to fit inside the dataset - and since this is an end
        // bound, index is, in a sense, already "+1" for our range later.
        Err(index) => (index, {
            let sum = index.checked_add(1);
            match sum {
                Some(s) if s < dataset.data.len() => sum,
                _ => None,
            }
        }),
    }
}

/// Returns the y-axis value for a given `x`, given two points to draw a line between.
fn interpolate_point(older_point: &Point, newer_point: &Point, x: f64) -> f64 {
    let delta_x = newer_point.0 - older_point.0;
    let delta_y = newer_point.1 - older_point.1;
    let slope = delta_y / delta_x;

    (older_point.1 + (x - older_point.0) * slope).max(0.0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_chart_test_interpolation() {
        let data = [(-3.0, 8.0), (-1.0, 6.0), (0.0, 5.0)];

        assert_eq!(interpolate_point(&data[1], &data[2], 0.0), 5.0);
        assert_eq!(interpolate_point(&data[1], &data[2], -0.25), 5.25);
        assert_eq!(interpolate_point(&data[1], &data[2], -0.5), 5.5);
        assert_eq!(interpolate_point(&data[0], &data[1], -1.0), 6.0);
        assert_eq!(interpolate_point(&data[0], &data[1], -1.5), 6.5);
        assert_eq!(interpolate_point(&data[0], &data[1], -2.0), 7.0);
        assert_eq!(interpolate_point(&data[0], &data[1], -2.5), 7.5);
        assert_eq!(interpolate_point(&data[0], &data[1], -3.0), 8.0);
    }

    #[test]
    fn time_chart_empty_dataset() {
        let data = [];
        let dataset = Dataset::default().data(&data);

        assert_eq!(get_start(&dataset, -100.0), (0, None));
        assert_eq!(get_start(&dataset, -3.0), (0, None));

        assert_eq!(get_end(&dataset, 0.0), (0, None));
        assert_eq!(get_end(&dataset, 100.0), (0, None));
    }

    #[test]
    fn time_chart_test_data_trimming() {
        let data = [
            (-3.0, 8.0),
            (-2.5, 15.0),
            (-2.0, 9.0),
            (-1.0, 6.0),
            (0.0, 5.0),
        ];
        let dataset = Dataset::default().data(&data);

        // Test start point cases (miss and hit)
        assert_eq!(get_start(&dataset, -100.0), (0, None));
        assert_eq!(get_start(&dataset, -3.0), (0, None));
        assert_eq!(get_start(&dataset, -2.8), (1, Some(0)));
        assert_eq!(get_start(&dataset, -2.5), (1, None));
        assert_eq!(get_start(&dataset, -2.4), (2, Some(1)));

        // Test end point cases (miss and hit)
        assert_eq!(get_end(&dataset, -2.5), (2, None));
        assert_eq!(get_end(&dataset, -2.4), (2, Some(3)));
        assert_eq!(get_end(&dataset, -1.4), (3, Some(4)));
        assert_eq!(get_end(&dataset, -1.0), (4, None));
        assert_eq!(get_end(&dataset, 0.0), (5, None));
        assert_eq!(get_end(&dataset, 1.0), (5, None));
        assert_eq!(get_end(&dataset, 100.0), (5, None));
    }

    struct LegendTestCase {
        chart_area: Rect,
        hidden_legend_constraints: (Constraint, Constraint),
        legend_area: Option<Rect>,
    }

    /// Test from the original tui-rs [`Chart`](tui::widgets::Chart).
    #[test]
    fn it_should_hide_the_legend() {
        let data = [(0.0, 5.0), (1.0, 6.0), (3.0, 7.0)];
        let cases = [
            LegendTestCase {
                chart_area: Rect::new(0, 0, 100, 100),
                hidden_legend_constraints: (Constraint::Ratio(1, 4), Constraint::Ratio(1, 4)),
                legend_area: Some(Rect::new(88, 0, 12, 12)),
            },
            LegendTestCase {
                chart_area: Rect::new(0, 0, 100, 100),
                hidden_legend_constraints: (Constraint::Ratio(1, 10), Constraint::Ratio(1, 4)),
                legend_area: None,
            },
        ];
        for case in &cases {
            let datasets = (0..10)
                .map(|i| {
                    let name = format!("Dataset #{}", i);
                    Dataset::default().name(name).data(&data)
                })
                .collect::<Vec<_>>();
            let chart = TimeChart::new(datasets)
                .x_axis(Axis::default().title("X axis"))
                .y_axis(Axis::default().title("Y axis"))
                .hidden_legend_constraints(case.hidden_legend_constraints);
            let layout = chart.layout(case.chart_area);
            assert_eq!(layout.legend_area, case.legend_area);
        }
    }
}
