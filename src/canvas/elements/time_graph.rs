#![allow(dead_code)]

use std::{borrow::Cow, time::Instant};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    symbols::Marker,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::AppState,
    canvas::{canvas_colours::CanvasColours, drawing_utils::interpolate_points},
    constants::{AUTOHIDE_TIMEOUT_MILLISECONDS, TIME_LABEL_HEIGHT_LIMIT},
};

use super::element::{Element, ElementBounds};

/// Struct representing the state of a [`TimeGraph`].
#[derive(Default)]
struct TimeGraphState {
    pub current_max_time_ms: u32,
    pub autohide_timer: Option<Instant>,
}

/// A stateful graph widget graphing between a time x-axis and some y-axis, supporting time zooming.
pub struct TimeGraph<'d> {
    state: TimeGraphState,
    bounds: ElementBounds,
    name: Cow<'static, str>,
    legend_constraints: (Constraint, Constraint),
    selected: bool,
    data: &'d [(Cow<'static, str>, Style, Vec<(f64, f64)>)],
    y_axis_legend: Axis<'d>,
    marker: Marker,
}

impl<'d> TimeGraph<'d> {
    /// Creates a new [`TimeGraph`].
    pub fn new(
        data: &'d [(Cow<'static, str>, Style, Vec<(f64, f64)>)], legend_bounds: &[f64; 2],
        labels: Vec<Span<'d>>,
    ) -> Self {
        Self {
            state: TimeGraphState::default(),
            bounds: ElementBounds::Unset,
            name: Cow::default(),
            legend_constraints: (Constraint::Ratio(1, 1), Constraint::Ratio(3, 4)),
            selected: false,
            data,
            y_axis_legend: Axis::default().bounds(*legend_bounds).labels(labels),
            marker: Marker::Braille,
        }
    }

    /// Sets the legend status for a [`TimeGraph`].
    pub fn enable_legend(mut self, enable_legend: bool) -> Self {
        if enable_legend {
            self.legend_constraints = (Constraint::Ratio(1, 1), Constraint::Ratio(3, 4));
        } else {
            self.legend_constraints = (Constraint::Ratio(0, 1), Constraint::Ratio(0, 1));
        }

        self
    }

    /// Sets the marker type for a [`TimeGraph`].
    pub fn marker(mut self, use_dot: bool) -> Self {
        self.marker = if use_dot {
            Marker::Dot
        } else {
            Marker::Braille
        };

        self
    }
}

impl<'d> Element for TimeGraph<'d> {
    fn draw<B: Backend>(
        &mut self, f: &mut Frame<'_, B>, app_state: &AppState, draw_loc: Rect,
        style: &CanvasColours,
    ) -> anyhow::Result<()> {
        let time_start = -(f64::from(self.state.current_max_time_ms));
        let display_time_labels = vec![
            Span::styled(
                format!("{}s", self.state.current_max_time_ms / 1000),
                style.graph_style,
            ),
            Span::styled("0s".to_string(), style.graph_style),
        ];

        let x_axis = if app_state.app_config_fields.hide_time
            || (app_state.app_config_fields.autohide_time && self.state.autohide_timer.is_none())
        {
            Axis::default().bounds([time_start, 0.0])
        } else if let Some(time) = self.state.autohide_timer {
            if std::time::Instant::now().duration_since(time).as_millis()
                < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
            {
                Axis::default()
                    .bounds([time_start, 0.0])
                    .style(style.graph_style)
                    .labels(display_time_labels)
            } else {
                self.state.autohide_timer = None;
                Axis::default().bounds([time_start, 0.0])
            }
        } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
            Axis::default().bounds([time_start, 0.0])
        } else {
            Axis::default()
                .bounds([time_start, 0.0])
                .style(style.graph_style)
                .labels(display_time_labels)
        };

        let y_axis = self.y_axis_legend.clone(); // Not sure how else to do this right now.
        let border_style = if self.selected {
            style.highlighted_border_style
        } else {
            style.border_style
        };

        let title = if app_state.is_expanded {
            Spans::from(vec![
                Span::styled(format!(" {} ", self.name), style.widget_title_style),
                Span::styled(
                    format!(
                        "─{}─ Esc to go back ",
                        "─".repeat(
                            usize::from(draw_loc.width).saturating_sub(
                                UnicodeSegmentation::graphemes(
                                    format!(" {} ── Esc to go back", self.name).as_str(),
                                    true
                                )
                                .count()
                                    + 2
                            )
                        )
                    ),
                    border_style,
                ),
            ])
        } else {
            Spans::from(Span::styled(
                format!(" {} ", self.name),
                style.widget_title_style,
            ))
        };

        // We unfortunately must store the data at least once, otherwise we get issues with local referencing.
        let processed_data = self
            .data
            .iter()
            .map(|(name, style, dataset)| {
                // Match time + interpolate; we assume all the datasets are sorted.

                if let Some(end_pos) = dataset.iter().position(|(time, _data)| *time >= time_start)
                {
                    if end_pos > 0 {
                        // We can interpolate.

                        let old = dataset[end_pos - 1];
                        let new = dataset[end_pos];

                        let interpolated_point = interpolate_points(&old, &new, time_start);

                        (
                            (name, *style, &dataset[end_pos..]),
                            Some([(time_start, interpolated_point), new]),
                        )
                    } else {
                        ((name, *style, &dataset[end_pos..]), None)
                    }
                } else {
                    // No need to interpolate, just return the entire thing.
                    ((name, *style, &dataset[..]), None)
                }
            })
            .collect::<Vec<_>>();

        let datasets = processed_data
            .iter()
            .map(|((name, style, cut_data), interpolated_data)| {
                if let Some(interpolated_data) = interpolated_data {
                    vec![
                        Dataset::default()
                            .data(cut_data)
                            .style(*style)
                            .name(name.as_ref())
                            .marker(self.marker)
                            .graph_type(tui::widgets::GraphType::Line),
                        Dataset::default()
                            .data(interpolated_data.as_ref())
                            .style(*style)
                            .marker(self.marker)
                            .graph_type(tui::widgets::GraphType::Line),
                    ]
                } else {
                    vec![Dataset::default()
                        .data(cut_data)
                        .style(*style)
                        .name(name.as_ref())
                        .marker(self.marker)
                        .graph_type(tui::widgets::GraphType::Line)]
                }
            })
            .flatten()
            .collect();

        f.render_widget(
            Chart::new(datasets)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(if self.selected {
                            style.highlighted_border_style
                        } else {
                            style.border_style
                        }),
                )
                .x_axis(x_axis)
                .y_axis(y_axis)
                .hidden_legend_constraints(self.legend_constraints),
            draw_loc,
        );

        Ok(())
    }

    fn recalculate_click_bounds(&mut self) {}

    fn click_bounds(&self) -> super::element::ElementBounds {
        self.bounds
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn select(&mut self) {
        self.selected = true;
    }

    fn unselect(&mut self) {
        self.selected = false;
    }
}
