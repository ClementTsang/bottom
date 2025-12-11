//! A [`tui::widgets::Chart`] but slightly more specialized to show
//! right-aligned timeseries data.
//!
//! Generally should be updated to be in sync with [`chart.rs`](https://github.com/ratatui-org/ratatui/blob/main/src/widgets/chart.rs);
//! the specializations are factored out to `time_chart/points.rs`.

mod canvas;
mod grid;
mod points;

use std::{cmp::max, str::FromStr, time::Instant};

use canvas::*;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Styled},
    symbols::{self, Marker},
    text::{Line, Span},
    widgets::{Block, BlockExt, Borders, GraphType, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::data::Values,
    utils::general::{saturating_log2, saturating_log10},
};

pub const DEFAULT_LEGEND_CONSTRAINTS: (Constraint, Constraint) =
    (Constraint::Ratio(1, 4), Constraint::Length(4));

/// A single graph point.
pub type Point = (f64, f64);

/// An axis bound type. Allows us to save a f64 since we know that we are
/// usually bound from some values [0.0, a], or [-b, 0.0].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AxisBound {
    /// Just 0.
    #[default]
    Zero,
    /// Bound by a minimum value to 0.
    Min(f64),
    /// Bound by 0 and a max value.
    Max(f64),
    /// Bound by a min and max value.
    MinMax(f64, f64),
}

impl AxisBound {
    fn get_bounds(&self) -> [f64; 2] {
        match self {
            AxisBound::Zero => [0.0, 0.0],
            AxisBound::Min(min) => [*min, 0.0],
            AxisBound::Max(max) => [0.0, *max],
            AxisBound::MinMax(min, max) => [*min, *max],
        }
    }
}

/// An X or Y axis for the [`TimeChart`] widget
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Axis<'a> {
    /// Title displayed next to axis end
    pub(crate) title: Option<Line<'a>>,
    /// Bounds for the axis (all data points outside these limits will not be
    /// represented)
    pub(crate) bounds: AxisBound,
    /// A list of labels to put to the left or below the axis
    pub(crate) labels: Option<Vec<Span<'a>>>,
    /// The style used to draw the axis itself
    pub(crate) style: Style,
    /// The alignment of the labels of the Axis
    pub(crate) labels_alignment: Alignment,
}

impl<'a> Axis<'a> {
    /// Sets the axis title
    ///
    /// It will be displayed at the end of the axis. For an X axis this is the
    /// right, for a Y axis, this is the top.
    #[must_use = "method moves the value of self and returns the modified value"]
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn title<T>(mut self, title: T) -> Axis<'a>
    where
        T: Into<Line<'a>>,
    {
        self.title = Some(title.into());
        self
    }

    /// Sets the bounds of this axis.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn bounds(mut self, bounds: AxisBound) -> Axis<'a> {
        self.bounds = bounds;
        self
    }

    /// Sets the axis labels
    ///
    /// - For the X axis, the labels are displayed left to right.
    /// - For the Y axis, the labels are displayed bottom to top.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn labels(mut self, labels: Vec<Span<'a>>) -> Axis<'a> {
        self.labels = Some(labels);
        self
    }

    /// Sets the axis style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Axis<'a> {
        self.style = style.into();
        self
    }

    /// Sets the labels alignment of the axis
    ///
    /// The alignment behaves differently based on the axis:
    /// - Y axis: The labels are aligned within the area on the left of the axis
    /// - X axis: The first X-axis label is aligned relative to the Y-axis
    ///
    /// On the X axis, this parameter only affects the first label.
    #[must_use = "method moves the value of self and returns the modified value"]
    #[expect(dead_code)]
    pub fn labels_alignment(mut self, alignment: Alignment) -> Axis<'a> {
        self.labels_alignment = alignment;
        self
    }
}

/// Allow users to specify the position of a legend in a [`TimeChart`]
///
/// See [`TimeChart::legend_position`]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum LegendPosition {
    /// Legend is centered on top
    Top,
    /// Legend is in the top-right corner. This is the **default**.
    #[default]
    TopRight,
    /// Legend is in the top-left corner
    TopLeft,
    /// Legend is centered on the left
    Left,
    /// Legend is centered on the right
    Right,
    /// Legend is centered on the bottom
    Bottom,
    /// Legend is in the bottom-right corner
    BottomRight,
    /// Legend is in the bottom-left corner
    BottomLeft,
}

impl LegendPosition {
    fn layout(
        &self, area: Rect, legend_width: u16, legend_height: u16, x_title_width: u16,
        y_title_width: u16,
    ) -> Option<Rect> {
        let mut height_margin = (area.height - legend_height) as i32;
        if x_title_width != 0 {
            height_margin -= 1;
        }
        if y_title_width != 0 {
            height_margin -= 1;
        }
        if height_margin < 0 {
            return None;
        };

        let (x, y) = match self {
            Self::TopRight => {
                if legend_width + y_title_width > area.width {
                    (area.right() - legend_width, area.top() + 1)
                } else {
                    (area.right() - legend_width, area.top())
                }
            }
            Self::TopLeft => {
                if y_title_width != 0 {
                    (area.left(), area.top() + 1)
                } else {
                    (area.left(), area.top())
                }
            }
            Self::Top => {
                let x = (area.width - legend_width) / 2;
                if area.left() + y_title_width > x {
                    (area.left() + x, area.top() + 1)
                } else {
                    (area.left() + x, area.top())
                }
            }
            Self::Left => {
                let mut y = (area.height - legend_height) / 2;
                if y_title_width != 0 {
                    y += 1;
                }
                if x_title_width != 0 {
                    y = y.saturating_sub(1);
                }
                (area.left(), area.top() + y)
            }
            Self::Right => {
                let mut y = (area.height - legend_height) / 2;
                if y_title_width != 0 {
                    y += 1;
                }
                if x_title_width != 0 {
                    y = y.saturating_sub(1);
                }
                (area.right() - legend_width, area.top() + y)
            }
            Self::BottomLeft => {
                if x_title_width + legend_width > area.width {
                    (area.left(), area.bottom() - legend_height - 1)
                } else {
                    (area.left(), area.bottom() - legend_height)
                }
            }
            Self::BottomRight => {
                if x_title_width != 0 {
                    (
                        area.right() - legend_width,
                        area.bottom() - legend_height - 1,
                    )
                } else {
                    (area.right() - legend_width, area.bottom() - legend_height)
                }
            }
            Self::Bottom => {
                let x = area.left() + (area.width - legend_width) / 2;
                if x + legend_width > area.right() - x_title_width {
                    (x, area.bottom() - legend_height - 1)
                } else {
                    (x, area.bottom() - legend_height)
                }
            }
        };

        Some(Rect::new(x, y, legend_width, legend_height))
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseLegendPositionError;

impl FromStr for LegendPosition {
    type Err = ParseLegendPositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "top" => Ok(Self::Top),
            "top-left" => Ok(Self::TopLeft),
            "top-right" => Ok(Self::TopRight),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "bottom-left" => Ok(Self::BottomLeft),
            "bottom" => Ok(Self::Bottom),
            "bottom-right" => Ok(Self::BottomRight),
            _ => Err(ParseLegendPositionError),
        }
    }
}

#[derive(Debug, Default, Clone)]
enum Data<'a> {
    Some {
        times: &'a [Instant],
        values: &'a Values,
    },
    Custom {
        times: &'a [Instant],
        values: &'a [f64],
    },
    #[default]
    None,
}

/// A group of data points
///
/// This is the main element composing a [`TimeChart`].
///
/// A dataset can be [named](Dataset::name). Only named datasets will be
/// rendered in the legend.
#[derive(Debug, Default, Clone)]
pub struct Dataset<'a> {
    /// Name of the dataset (used in the legend if shown)
    name: Option<Line<'a>>,
    /// A reference to data.
    data: Data<'a>,
    /// Symbol used for each points of this dataset
    marker: symbols::Marker,
    /// Determines graph type used for drawing points
    graph_type: GraphType,
    /// Style used to plot this dataset
    style: Style,
    /// Whether to fill the dataset.
    filled: bool,
    /// Whether to invert the dataset (negate values).
    inverted: bool,
}

impl<'a> Dataset<'a> {
    /// Sets the name of the dataset.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn name<S>(mut self, name: S) -> Dataset<'a>
    where
        S: Into<Line<'a>>,
    {
        self.name = Some(name.into());
        self
    }

    /// Sets whether the dataset is filled.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn filled(mut self, filled: bool) -> Dataset<'a> {
        self.filled = filled;
        self
    }

    /// Sets whether the dataset is inverted.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn inverted(mut self, inverted: bool) -> Dataset<'a> {
        self.inverted = inverted;
        self
    }

    /// Sets the data points of this dataset
    ///
    /// Points will then either be rendered as scattered points or with lines
    /// between them depending on [`Dataset::graph_type`].
    ///
    /// Data consist in an array of `f64` tuples (`(f64, f64)`), the first
    /// element being X and the second Y. It's also worth noting that,
    /// unlike the [`Rect`], here the Y axis is bottom to top, as in math.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn data(mut self, times: &'a [Instant], values: &'a Values) -> Dataset<'a> {
        self.data = Data::Some { times, values };
        self
    }

    /// Sets data using a custom slice.
    pub fn data_custom(mut self, times: &'a [Instant], values: &'a [f64]) -> Dataset<'a> {
        self.data = Data::Custom { times, values };
        self
    }

    /// Sets the kind of character to use to display this dataset
    ///
    /// You can use dots (`•`), blocks (`█`), bars (`▄`), braille (`⠓`, `⣇`,
    /// `⣿`) or half-blocks (`█`, `▄`, and `▀`). See [symbols::Marker] for
    /// more details.
    ///
    /// Note [`Marker::Braille`] requires a font that supports Unicode Braille
    /// Patterns.
    #[must_use = "method moves the value of self and returns the modified value"]
    #[expect(dead_code)]
    pub fn marker(mut self, marker: symbols::Marker) -> Dataset<'a> {
        self.marker = marker;
        self
    }

    /// Sets how the dataset should be drawn
    ///
    /// [`TimeChart`] can draw either a [scatter](GraphType::Scatter) or
    /// [line](GraphType::Line) charts. A scatter will draw only the points
    /// in the dataset while a line will also draw a line between them. See
    /// [`GraphType`] for more details
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn graph_type(mut self, graph_type: GraphType) -> Dataset<'a> {
        self.graph_type = graph_type;
        self
    }

    /// Sets the style of this dataset
    ///
    /// The given style will be used to draw the legend and the data points.
    /// Currently the legend will use the entire style whereas the data
    /// points will only use the foreground.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g.
    /// [`Style`], [`Color`], or your own type that implements
    /// [`Into<Style>`]).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Dataset<'a> {
        self.style = style.into();
        self
    }
}

/// A container that holds all the infos about where to display each elements of
/// the chart (axis, labels, legend, ...).
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
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

/// Whether to additionally scale all values before displaying them. Defaults to none.
#[derive(Default, Debug, Clone, Copy)]
pub(crate) enum ChartScaling {
    #[default]
    Linear,
    Log10,
    Log2,
}

impl ChartScaling {
    /// Scale a value.
    pub(super) fn scale(&self, value: f64) -> f64 {
        // Remember to do saturating log checks as otherwise 0.0 becomes inf, and you get
        // gaps!
        match self {
            ChartScaling::Linear => value,
            ChartScaling::Log10 => saturating_log10(value),
            ChartScaling::Log2 => saturating_log2(value),
        }
    }
}

/// A "custom" chart, just a slightly tweaked [`tui::widgets::Chart`] from
/// ratatui, but with greater control over the legend, and built with the idea
/// of drawing data points relative to a time-based x-axis.
///
/// Main changes:
/// - Styling option for the legend box
/// - Automatically trimming out redundant draws in the x-bounds.
/// - Automatic interpolation to points that fall *just* outside of the screen.
///
/// TODO: Support for putting the legend on the left side.
#[derive(Debug, Default, Clone)]
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
    /// The legend's style.
    legend_style: Style,
    /// Constraints used to determine whether the legend should be shown or not
    hidden_legend_constraints: (Constraint, Constraint),
    /// The position determining whether the length is shown or hidden, regardless
    /// of `hidden_legend_constraints`
    legend_position: Option<LegendPosition>,
    /// The marker type.
    pub marker: Marker,
    /// Whether to scale the values differently.
    scaling: ChartScaling,
}

impl<'a> TimeChart<'a> {
    /// Creates a chart with the given [datasets](Dataset).
    pub fn new(datasets: Vec<Dataset<'a>>) -> TimeChart<'a> {
        TimeChart {
            block: None,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            style: Style::default(),
            legend_style: Style::default(),
            datasets,
            hidden_legend_constraints: (Constraint::Ratio(1, 4), Constraint::Ratio(1, 4)),
            legend_position: Some(LegendPosition::default()),
            marker: Marker::Braille,
            scaling: ChartScaling::default(),
        }
    }

    /// Wraps the chart with the given [`Block`]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> TimeChart<'a> {
        self.block = Some(block);
        self
    }

    /// Sets the style of the entire chart
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g.
    /// [`Style`], [`Color`], or your own type that implements
    /// [`Into<Style>`]).
    ///
    /// Styles of [`Axis`] and [`Dataset`] will have priority over this style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> TimeChart<'a> {
        self.style = style.into();
        self
    }

    /// Sets the legend's style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn legend_style(mut self, legend_style: Style) -> TimeChart<'a> {
        self.legend_style = legend_style;
        self
    }

    /// Sets the X [`Axis`]
    ///
    /// The default is an empty [`Axis`], i.e. only a line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn x_axis(mut self, axis: Axis<'a>) -> TimeChart<'a> {
        self.x_axis = axis;
        self
    }

    /// Sets the Y [`Axis`]
    ///
    /// The default is an empty [`Axis`], i.e. only a line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn y_axis(mut self, axis: Axis<'a>) -> TimeChart<'a> {
        self.y_axis = axis;
        self
    }

    /// Sets the marker type.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn marker(mut self, marker: Marker) -> TimeChart<'a> {
        self.marker = marker;
        self
    }

    /// Sets the constraints used to determine whether the legend should be
    /// shown or not.
    ///
    /// The tuple's first constraint is used for the width and the second for
    /// the height. If the legend takes more space than what is allowed by
    /// any constraint, the legend is hidden. [`Constraint::Min`] is an
    /// exception and will always show the legend.
    ///
    /// If this is not set, the default behavior is to hide the legend if it is
    /// greater than 25% of the chart, either horizontally or vertically.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn hidden_legend_constraints(
        mut self, constraints: (Constraint, Constraint),
    ) -> TimeChart<'a> {
        self.hidden_legend_constraints = constraints;
        self
    }

    /// Sets the position of a legend or hide it.
    ///
    /// The default is [`LegendPosition::TopRight`].
    ///
    /// If [`None`] is given, hide the legend even if
    /// [`hidden_legend_constraints`] determines it should be shown. In
    /// contrast, if `Some(...)` is given, [`hidden_legend_constraints`] might
    /// still decide whether to show the legend or not.
    ///
    /// See [`LegendPosition`] for all available positions.
    ///
    /// [`hidden_legend_constraints`]: Self::hidden_legend_constraints
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn legend_position(mut self, position: Option<LegendPosition>) -> TimeChart<'a> {
        self.legend_position = position;
        self
    }

    /// Set chart scaling.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn scaling(mut self, scaling: ChartScaling) -> TimeChart<'a> {
        self.scaling = scaling;
        self
    }

    /// Compute the internal layout of the chart given the area. If the area is
    /// too small some elements may be automatically hidden
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
        x += self.max_width_of_labels_left_of_y_axis(area, self.y_axis.labels.is_some());

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

        if let Some(legend_position) = self.legend_position {
            let legends = self
                .datasets
                .iter()
                .filter_map(|d| Some(d.name.as_ref()?.width() as u16));

            if let Some(inner_width) = legends.clone().max() {
                let legend_width = inner_width + 2;
                let legend_height = legends.count() as u16 + 2;

                let [max_legend_width] = Layout::horizontal([self.hidden_legend_constraints.0])
                    .flex(Flex::Start)
                    .areas(layout.graph_area);

                let [max_legend_height] = Layout::vertical([self.hidden_legend_constraints.1])
                    .flex(Flex::Start)
                    .areas(layout.graph_area);

                if inner_width > 0
                    && legend_width <= max_legend_width.width
                    && legend_height <= max_legend_height.height
                {
                    layout.legend_area = legend_position.layout(
                        layout.graph_area,
                        legend_width,
                        legend_height,
                        layout
                            .title_x
                            .and(self.x_axis.title.as_ref())
                            .map(|t| t.width() as u16)
                            .unwrap_or_default(),
                        layout
                            .title_y
                            .and(self.y_axis.title.as_ref())
                            .map(|t| t.width() as u16)
                            .unwrap_or_default(),
                    );
                }
            }
        }
        layout
    }

    fn max_width_of_labels_left_of_y_axis(&self, area: Rect, has_y_axis: bool) -> u16 {
        let mut max_width = self
            .y_axis
            .labels
            .as_ref()
            .map(|l| l.iter().map(Span::width).max().unwrap_or_default() as u16)
            .unwrap_or_default();

        if let Some(first_x_label) = self
            .x_axis
            .labels
            .as_ref()
            .and_then(|labels| labels.first())
        {
            let first_label_width = first_x_label.content.width() as u16;
            let width_left_of_y_axis = match self.x_axis.labels_alignment {
                Alignment::Left => {
                    // The last character of the label should be below the Y-Axis when it exists,
                    // not on its left
                    let y_axis_offset = u16::from(has_y_axis);
                    first_label_width.saturating_sub(y_axis_offset)
                }
                Alignment::Center => first_label_width / 2,
                Alignment::Right => 0,
            };
            max_width = max(max_width, width_left_of_y_axis);
        }
        // labels of y axis and first label of x axis can take at most 1/3rd of the
        // total width
        max_width.min(area.width / 3)
    }

    fn render_x_labels(
        &self, buf: &mut Buffer, layout: &ChartLayout, chart_area: Rect, graph_area: Rect,
    ) {
        let Some(y) = layout.label_x else { return };
        let Some(labels) = self.x_axis.labels.as_ref() else {
            return;
        };
        let labels_len = labels.len() as u16;
        if labels_len < 2 {
            return;
        }

        let first_label = labels.first().expect("must have at least 2 labels");
        let last_label = labels.last().expect("must have at least 2 labels");

        let width_between_ticks = graph_area.width / labels_len;

        let label_area = self.first_x_label_area(
            y,
            first_label.width() as u16,
            width_between_ticks,
            chart_area,
            graph_area,
        );

        let label_alignment = match self.x_axis.labels_alignment {
            Alignment::Left => Alignment::Right,
            Alignment::Center => Alignment::Center,
            Alignment::Right => Alignment::Left,
        };

        Self::render_label(buf, first_label, label_area, label_alignment);

        for (i, label) in labels[1..labels.len() - 1].iter().enumerate() {
            // We add 1 to x (and width-1 below) to leave at least one space before each
            // intermediate labels
            let x = graph_area.left() + (i + 1) as u16 * width_between_ticks + 1;
            let label_area = Rect::new(x, y, width_between_ticks.saturating_sub(1), 1);

            Self::render_label(buf, label, label_area, Alignment::Center);
        }

        let x = graph_area.right() - width_between_ticks;
        let label_area = Rect::new(x, y, width_between_ticks, 1);
        // The last label should be aligned Right to be at the edge of the graph area
        Self::render_label(buf, last_label, label_area, Alignment::Right);
    }

    fn first_x_label_area(
        &self, y: u16, label_width: u16, max_width_after_y_axis: u16, chart_area: Rect,
        graph_area: Rect,
    ) -> Rect {
        let (min_x, max_x) = match self.x_axis.labels_alignment {
            Alignment::Left => (chart_area.left(), graph_area.left()),
            Alignment::Center => (
                chart_area.left(),
                graph_area.left() + max_width_after_y_axis.min(label_width),
            ),
            Alignment::Right => (
                graph_area.left().saturating_sub(1),
                graph_area.left() + max_width_after_y_axis,
            ),
        };

        Rect::new(min_x, y, max_x - min_x, 1)
    }

    fn render_label(buf: &mut Buffer, label: &Span<'_>, label_area: Rect, alignment: Alignment) {
        let label_width = label.width() as u16;
        let bounded_label_width = label_area.width.min(label_width);

        let x = match alignment {
            Alignment::Left => label_area.left(),
            Alignment::Center => label_area.left() + label_area.width / 2 - bounded_label_width / 2,
            Alignment::Right => label_area.right() - bounded_label_width,
        };

        buf.set_span(x, label_area.top(), label, bounded_label_width);
    }

    fn render_y_labels(
        &self, buf: &mut Buffer, layout: &ChartLayout, chart_area: Rect, graph_area: Rect,
    ) {
        // FIXME: Control how many y-axis labels are rendered based on height.

        let Some(x) = layout.label_y else { return };
        let Some(labels) = self.y_axis.labels.as_ref() else {
            return;
        };
        let labels_len = labels.len() as u16;

        for (i, label) in labels.iter().enumerate() {
            let dy = i as u16 * (graph_area.height - 1) / (labels_len - 1);
            if dy < graph_area.bottom() {
                let label_area = Rect::new(
                    x,
                    graph_area.bottom().saturating_sub(1) - dy,
                    (graph_area.left() - chart_area.left()).saturating_sub(1),
                    1,
                );
                Self::render_label(buf, label, label_area, self.y_axis.labels_alignment);
            }
        }
    }
}

impl Widget for TimeChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        self.block.as_ref().render(area, buf);
        let chart_area = self.block.inner_if_some(area);
        if chart_area.is_empty() {
            return;
        }

        // Sample the style of the entire widget. This sample will be used to reset the
        // style of the cells that are part of the components put on top of the
        // graph area (i.e legend and axis names).
        let Some(original_style) = buf.cell((area.left(), area.top())).map(|cell| cell.style())
        else {
            return;
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
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(symbols::line::HORIZONTAL)
                        .set_style(self.x_axis.style);
                }
            }
        }

        if let Some(x) = layout.axis_y {
            for y in graph_area.top()..graph_area.bottom() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(symbols::line::VERTICAL)
                        .set_style(self.y_axis.style);
                }
            }
        }

        if let Some(y) = layout.axis_x {
            if let Some(x) = layout.axis_y {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(symbols::line::BOTTOM_LEFT)
                        .set_style(self.x_axis.style);
                }
            }
        }

        let x_bounds = self.x_axis.bounds.get_bounds();
        let y_bounds = self.y_axis.bounds.get_bounds();

        Canvas::default()
            .background_color(self.style.bg.unwrap_or(Color::Reset))
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .marker(self.marker)
            .paint(|ctx| {
                self.draw_points(ctx);
            })
            .render(graph_area, buf);

        if let Some((x, y)) = layout.title_x {
            if let Some(title) = self.x_axis.title.as_ref() {
                let width = graph_area
                    .right()
                    .saturating_sub(x)
                    .min(title.width() as u16);
                buf.set_style(
                    Rect {
                        x,
                        y,
                        width,
                        height: 1,
                    },
                    original_style,
                );
                buf.set_line(x, y, title, width);
            }
        }

        if let Some((x, y)) = layout.title_y {
            if let Some(title) = self.y_axis.title.as_ref() {
                let width = graph_area
                    .right()
                    .saturating_sub(x)
                    .min(title.width() as u16);
                buf.set_style(
                    Rect {
                        x,
                        y,
                        width,
                        height: 1,
                    },
                    original_style,
                );
                buf.set_line(x, y, title, width);
            }
        }

        if let Some(legend_area) = layout.legend_area {
            buf.set_style(legend_area, original_style);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.legend_style);
            for pos in block.inner(legend_area).positions() {
                if let Some(cell) = buf.cell_mut(pos) {
                    cell.set_symbol(" ");
                }
            }
            block.render(legend_area, buf);

            for (i, (dataset_name, dataset_style)) in self
                .datasets
                .iter()
                .filter_map(|ds| Some((ds.name.as_ref()?, ds.style())))
                .enumerate()
            {
                let name = dataset_name.clone().patch_style(dataset_style);
                name.render(
                    Rect {
                        x: legend_area.x + 1,
                        y: legend_area.y + 1 + i as u16,
                        width: legend_area.width - 2,
                        height: 1,
                    },
                    buf,
                );
            }
        }
    }
}

impl<'a> Styled for Axis<'a> {
    type Item = Axis<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> Styled for Dataset<'a> {
    type Item = Dataset<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> Styled for TimeChart<'a> {
    type Item = TimeChart<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

/// Tests taken from ratatui.
#[cfg(test)]
mod tests {
    macro_rules! assert_buffer_eq {
        ($actual_expr:expr, $expected_expr:expr) => {
            match (&$actual_expr, &$expected_expr) {
                (actual, expected) => {
                    if actual.area != expected.area {
                        panic!(
                            indoc::indoc!(
                                "
                                buffer areas not equal
                                expected:  {:?}
                                actual:    {:?}"
                            ),
                            expected, actual
                        );
                    }
                    let diff = expected.diff(&actual);
                    if !diff.is_empty() {
                        let nice_diff = diff
                            .iter()
                            .enumerate()
                            .map(|(i, (x, y, cell))| {
                                let expected_cell = expected.cell((*x, *y)).unwrap();
                                indoc::formatdoc! {"
                                    {i}: at ({x}, {y})
                                      expected: {expected_cell:?}
                                      actual:   {cell:?}
                                "}
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        panic!(
                            indoc::indoc!(
                                "
                                buffer contents not equal
                                expected: {:?}
                                actual: {:?}
                                diff:
                                {}"
                            ),
                            expected, actual, nice_diff
                        );
                    }
                    // shouldn't get here, but this guards against future behavior
                    // that changes equality but not area or content
                    assert_eq!(actual, expected, "buffers not equal");
                }
            }
        };
    }

    use std::time::Duration;

    use tui::style::{Modifier, Stylize};

    use super::*;

    struct LegendTestCase {
        chart_area: Rect,
        hidden_legend_constraints: (Constraint, Constraint),
        legend_area: Option<Rect>,
    }

    #[test]
    fn it_should_hide_the_legend() {
        let now = Instant::now();
        let times = [
            now,
            now.checked_add(Duration::from_secs(1)).unwrap(),
            now.checked_add(Duration::from_secs(2)).unwrap(),
        ];
        let mut values = Values::default();
        values.push(5.0);
        values.push(6.0);
        values.push(7.0);

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
                    let name = format!("Dataset #{i}");
                    Dataset::default().name(name).data(&times, &values)
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

    #[test]
    fn axis_can_be_stylized() {
        assert_eq!(
            Axis::default().black().on_white().bold().not_dim().style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM)
        )
    }

    #[test]
    fn dataset_can_be_stylized() {
        assert_eq!(
            Dataset::default().black().on_white().bold().not_dim().style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM)
        )
    }

    #[test]
    fn chart_can_be_stylized() {
        assert_eq!(
            TimeChart::new(vec![])
                .black()
                .on_white()
                .bold()
                .not_dim()
                .style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM)
        )
    }

    #[test]
    fn graph_type_to_string() {
        assert_eq!(GraphType::Scatter.to_string(), "Scatter");
        assert_eq!(GraphType::Line.to_string(), "Line");
    }

    #[test]
    fn graph_type_from_str() {
        assert_eq!("Scatter".parse::<GraphType>(), Ok(GraphType::Scatter));
        assert_eq!("Line".parse::<GraphType>(), Ok(GraphType::Line));
        assert!("".parse::<GraphType>().is_err());
    }

    #[test]
    fn it_does_not_panic_if_title_is_wider_than_buffer() {
        let widget = TimeChart::default()
            .y_axis(Axis::default().title("xxxxxxxxxxxxxxxx"))
            .x_axis(Axis::default().title("xxxxxxxxxxxxxxxx"));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 4));
        widget.render(buffer.area, &mut buffer);

        assert_eq!(buffer, Buffer::with_lines(vec![" ".repeat(8); 4]))
    }

    #[test]
    fn datasets_without_name_do_not_contribute_to_legend_height() {
        let data_named_1 = Dataset::default().name("data1"); // must occupy a row in legend
        let data_named_2 = Dataset::default().name(""); // must occupy a row in legend, even if name is empty
        let data_unnamed = Dataset::default(); // must not occupy a row in legend
        let widget = TimeChart::new(vec![data_named_1, data_unnamed, data_named_2]);
        let buffer = Buffer::empty(Rect::new(0, 0, 50, 25));
        let layout = widget.layout(buffer.area);

        assert!(layout.legend_area.is_some());
        assert_eq!(layout.legend_area.unwrap().height, 4); // 2 for borders, 2
        // for rows
    }

    #[test]
    fn no_legend_if_no_named_datasets() {
        let dataset = Dataset::default();
        let widget = TimeChart::new(vec![dataset; 3]);
        let buffer = Buffer::empty(Rect::new(0, 0, 50, 25));
        let layout = widget.layout(buffer.area);

        assert!(layout.legend_area.is_none());
    }

    #[test]
    fn dataset_legend_style_is_patched() {
        let long_dataset_name = Dataset::default().name("Very long name");
        let short_dataset =
            Dataset::default().name(Line::from("Short name").alignment(Alignment::Right));
        let widget = TimeChart::new(vec![long_dataset_name, short_dataset])
            .hidden_legend_constraints((100.into(), 100.into()));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 5));

        widget.render(buffer.area, &mut buffer);

        let expected = Buffer::with_lines(vec![
            "    ┌──────────────┐",
            "    │Very long name│",
            "    │    Short name│",
            "    └──────────────┘",
            "                    ",
        ]);
        assert_buffer_eq!(buffer, expected);
    }

    #[test]
    fn test_chart_have_a_topleft_legend() {
        let chart = TimeChart::new(vec![Dataset::default().name("Ds1")])
            .legend_position(Some(LegendPosition::TopLeft));

        let area = Rect::new(0, 0, 30, 20);
        let mut buffer = Buffer::empty(area);

        chart.render(buffer.area, &mut buffer);

        let expected = Buffer::with_lines(vec![
            "┌───┐                         ",
            "│Ds1│                         ",
            "└───┘                         ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
        ]);

        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_chart_have_a_long_y_axis_title_overlapping_legend() {
        let chart = TimeChart::new(vec![Dataset::default().name("Ds1")])
            .y_axis(Axis::default().title("The title overlap a legend."));

        let area = Rect::new(0, 0, 30, 20);
        let mut buffer = Buffer::empty(area);

        chart.render(buffer.area, &mut buffer);

        let expected = Buffer::with_lines(vec![
            "The title overlap a legend.   ",
            "                         ┌───┐",
            "                         │Ds1│",
            "                         └───┘",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
        ]);

        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_chart_have_overflowed_y_axis() {
        let chart = TimeChart::new(vec![Dataset::default().name("Ds1")])
            .y_axis(Axis::default().title("The title overlap a legend."));

        let area = Rect::new(0, 0, 10, 10);
        let mut buffer = Buffer::empty(area);

        chart.render(buffer.area, &mut buffer);

        let expected = Buffer::with_lines(vec![
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
            "          ",
        ]);

        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_legend_area_can_fit_same_chart_area() {
        let name = "Data";
        let chart = TimeChart::new(vec![Dataset::default().name(name)])
            .hidden_legend_constraints((Constraint::Percentage(100), Constraint::Percentage(100)));

        let area = Rect::new(0, 0, name.len() as u16 + 2, 3);
        let mut buffer = Buffer::empty(area);

        let expected = Buffer::with_lines(vec!["┌────┐", "│Data│", "└────┘"]);

        [
            LegendPosition::TopLeft,
            LegendPosition::Top,
            LegendPosition::TopRight,
            LegendPosition::Left,
            LegendPosition::Right,
            LegendPosition::Bottom,
            LegendPosition::BottomLeft,
            LegendPosition::BottomRight,
        ]
        .iter()
        .for_each(|&position| {
            let chart = chart.clone().legend_position(Some(position));
            buffer.reset();
            chart.render(buffer.area, &mut buffer);
            assert_eq!(buffer, expected);
        });
    }

    #[test]
    fn test_legend_of_chart_have_odd_margin_size() {
        let name = "Data";
        let base_chart = TimeChart::new(vec![Dataset::default().name(name)])
            .hidden_legend_constraints((Constraint::Percentage(100), Constraint::Percentage(100)));

        let area = Rect::new(0, 0, name.len() as u16 + 2 + 3, 3 + 3);
        let mut buffer = Buffer::empty(area);

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::TopLeft));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌────┐   ",
                "│Data│   ",
                "└────┘   ",
                "         ",
                "         ",
                "         ",
            ])
        );
        buffer.reset();

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::Top));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                " ┌────┐  ",
                " │Data│  ",
                " └────┘  ",
                "         ",
                "         ",
                "         ",
            ])
        );

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::TopRight));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "   ┌────┐",
                "   │Data│",
                "   └────┘",
                "         ",
                "         ",
                "         ",
            ])
        );

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::Left));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "         ",
                "┌────┐   ",
                "│Data│   ",
                "└────┘   ",
                "         ",
                "         ",
            ])
        );
        buffer.reset();

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::Right));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "         ",
                "   ┌────┐",
                "   │Data│",
                "   └────┘",
                "         ",
                "         ",
            ])
        );

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::BottomLeft));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "         ",
                "         ",
                "         ",
                "┌────┐   ",
                "│Data│   ",
                "└────┘   ",
            ])
        );

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::Bottom));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "         ",
                "         ",
                "         ",
                " ┌────┐  ",
                " │Data│  ",
                " └────┘  ",
            ])
        );

        let chart = base_chart
            .clone()
            .legend_position(Some(LegendPosition::BottomRight));
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "         ",
                "         ",
                "         ",
                "   ┌────┐",
                "   │Data│",
                "   └────┘",
            ])
        );

        let chart = base_chart.clone().legend_position(None);
        buffer.reset();
        chart.render(buffer.area, &mut buffer);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "         ",
                "         ",
                "         ",
                "         ",
                "         ",
                "         ",
            ])
        );
    }
}
