use lazy_static::lazy_static;
use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::{layout_manager::WidgetDirection, App},
    canvas::{
        drawing_utils::{get_column_widths, get_start_position},
        Painter,
    },
    constants::*,
    data_conversion::ConvertedCpuData,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    terminal::Frame,
    text::Span,
    text::Spans,
    widgets::{Axis, Block, Borders, Chart, Dataset, Row, Table},
};

const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
const AVG_POSITION: usize = 1;
const ALL_POSITION: usize = 0;

lazy_static! {
    static ref CPU_LEGEND_HEADER_LENS: Vec<u16> = CPU_LEGEND_HEADER
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>();
}

pub trait CpuGraphWidget {
    fn draw_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
    fn draw_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl CpuGraphWidget for Painter {
    fn draw_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if draw_loc.width as f64 * 0.15 <= 6.0 {
            // Skip drawing legend
            if app_state.current_widget.widget_id == (widget_id + 1) {
                if app_state.app_config_fields.left_legend {
                    app_state.move_widget_selection(&WidgetDirection::Right);
                } else {
                    app_state.move_widget_selection(&WidgetDirection::Left);
                }
            }
            self.draw_cpu_graph(f, app_state, draw_loc, widget_id);
            if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&widget_id) {
                cpu_widget_state.is_legend_hidden = true;
            }

            // Update draw loc in widget map
            if app_state.should_get_widget_bounds() {
                if let Some(bottom_widget) = app_state.widget_map.get_mut(&widget_id) {
                    bottom_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                    bottom_widget.bottom_right_corner =
                        Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
                }
            }
        } else {
            let (graph_index, legend_index, constraints) =
                if app_state.app_config_fields.left_legend {
                    (
                        1,
                        0,
                        [Constraint::Percentage(15), Constraint::Percentage(85)],
                    )
                } else {
                    (
                        0,
                        1,
                        [Constraint::Percentage(85), Constraint::Percentage(15)],
                    )
                };

            let partitioned_draw_loc = Layout::default()
                .margin(0)
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(draw_loc);

            self.draw_cpu_graph(f, app_state, partitioned_draw_loc[graph_index], widget_id);
            self.draw_cpu_legend(
                f,
                app_state,
                partitioned_draw_loc[legend_index],
                widget_id + 1,
            );

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(cpu_widget) = app_state.widget_map.get_mut(&widget_id) {
                    cpu_widget.top_left_corner = Some((
                        partitioned_draw_loc[graph_index].x,
                        partitioned_draw_loc[graph_index].y,
                    ));
                    cpu_widget.bottom_right_corner = Some((
                        partitioned_draw_loc[graph_index].x
                            + partitioned_draw_loc[graph_index].width,
                        partitioned_draw_loc[graph_index].y
                            + partitioned_draw_loc[graph_index].height,
                    ));
                }

                if let Some(legend_widget) = app_state.widget_map.get_mut(&(widget_id + 1)) {
                    legend_widget.top_left_corner = Some((
                        partitioned_draw_loc[legend_index].x,
                        partitioned_draw_loc[legend_index].y,
                    ));
                    legend_widget.bottom_right_corner = Some((
                        partitioned_draw_loc[legend_index].x
                            + partitioned_draw_loc[legend_index].width,
                        partitioned_draw_loc[legend_index].y
                            + partitioned_draw_loc[legend_index].height,
                    ));
                }
            }
        }
    }

    fn draw_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&widget_id) {
            let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;

            let display_time_labels = vec![
                Span::styled(
                    format!("{}s", cpu_widget_state.current_display_time / 1000),
                    self.colours.graph_style,
                ),
                Span::styled("0s".to_string(), self.colours.graph_style),
            ];

            let y_axis_labels = vec![
                Span::styled("0%", self.colours.graph_style),
                Span::styled("100%", self.colours.graph_style),
            ];

            let x_axis = if app_state.app_config_fields.hide_time
                || (app_state.app_config_fields.autohide_time
                    && cpu_widget_state.autohide_timer.is_none())
            {
                Axis::default().bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
            } else if let Some(time) = cpu_widget_state.autohide_timer {
                if std::time::Instant::now().duration_since(time).as_millis()
                    < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
                {
                    Axis::default()
                        .bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
                        .style(self.colours.graph_style)
                        .labels(display_time_labels)
                } else {
                    cpu_widget_state.autohide_timer = None;
                    Axis::default().bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
                }
            } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
                Axis::default().bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
            } else {
                Axis::default()
                    .bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
                    .style(self.colours.graph_style)
                    .labels(display_time_labels)
            };

            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .bounds([0.0, 100.5])
                .labels(y_axis_labels);

            let use_dot = app_state.app_config_fields.use_dot;
            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;
            let current_scroll_position = cpu_widget_state.scroll_state.current_scroll_position;
            let dataset_vector: Vec<Dataset<'_>> = if current_scroll_position == ALL_POSITION {
                cpu_data
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(itx, cpu)| {
                        Dataset::default()
                            .marker(if use_dot {
                                Marker::Dot
                            } else {
                                Marker::Braille
                            })
                            .style(if show_avg_cpu && itx == AVG_POSITION {
                                self.colours.avg_colour_style
                            } else {
                                self.colours.cpu_colour_styles
                                    [itx % self.colours.cpu_colour_styles.len()]
                            })
                            .data(&cpu.cpu_data[..])
                            .graph_type(tui::widgets::GraphType::Line)
                    })
                    .collect()
            } else if let Some(cpu) = cpu_data.get(current_scroll_position) {
                vec![Dataset::default()
                    .marker(if use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(if show_avg_cpu && current_scroll_position == AVG_POSITION {
                        self.colours.avg_colour_style
                    } else {
                        self.colours.cpu_colour_styles[cpu_widget_state
                            .scroll_state
                            .current_scroll_position
                            % self.colours.cpu_colour_styles.len()]
                    })
                    .data(&cpu.cpu_data[..])
                    .graph_type(tui::widgets::GraphType::Line)]
            } else {
                vec![]
            };

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let border_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " CPU ── Esc to go back ";
                Spans::from(vec![
                    Span::styled(" CPU ", self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to go back ",
                            "─".repeat(usize::from(draw_loc.width).saturating_sub(
                                UnicodeSegmentation::graphemes(TITLE_BASE, true).count() + 2
                            ))
                        ),
                        border_style,
                    ),
                ])
            } else {
                Spans::from(Span::styled(" CPU ", self.colours.widget_title_style))
            };

            f.render_widget(
                Chart::new(dataset_vector)
                    .block(
                        Block::default()
                            .title(title)
                            .borders(Borders::ALL)
                            .border_style(border_style),
                    )
                    .x_axis(x_axis)
                    .y_axis(y_axis),
                draw_loc,
            );
        }
    }

    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&(widget_id - 1))
        {
            cpu_widget_state.is_legend_hidden = false;
            let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;
            let cpu_table_state = &mut cpu_widget_state.scroll_state.table_state;
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                app_state.app_config_fields.table_gap
            };
            let start_position = get_start_position(
                usize::from(
                    (draw_loc.height + (1 - table_gap)).saturating_sub(self.table_height_offset),
                ),
                &cpu_widget_state.scroll_state.scroll_direction,
                &mut cpu_widget_state.scroll_state.previous_scroll_position,
                cpu_widget_state.scroll_state.current_scroll_position,
                app_state.is_force_redraw,
            );
            cpu_table_state.select(Some(
                cpu_widget_state
                    .scroll_state
                    .current_scroll_position
                    .saturating_sub(start_position),
            ));

            let sliced_cpu_data = &cpu_data[start_position..];

            let mut offset_scroll_index = cpu_widget_state
                .scroll_state
                .current_scroll_position
                .saturating_sub(start_position);
            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;

            // Calculate widths
            if recalculate_column_widths {
                cpu_widget_state.table_width_state.desired_column_widths = vec![6, 4];
                cpu_widget_state.table_width_state.calculated_column_widths = get_column_widths(
                    draw_loc.width,
                    &[None, None],
                    &(CPU_LEGEND_HEADER_LENS
                        .iter()
                        .map(|width| Some(*width))
                        .collect::<Vec<_>>()),
                    &[Some(0.5), Some(0.5)],
                    &(cpu_widget_state
                        .table_width_state
                        .desired_column_widths
                        .iter()
                        .map(|width| Some(*width))
                        .collect::<Vec<_>>()),
                    false,
                );
            }

            let dcw = &cpu_widget_state.table_width_state.desired_column_widths;
            let ccw = &cpu_widget_state.table_width_state.calculated_column_widths;
            let cpu_rows = sliced_cpu_data.iter().enumerate().filter_map(|(itx, cpu)| {
                let truncated_name: Cow<'_, str> =
                    if let (Some(desired_column_width), Some(calculated_column_width)) =
                        (dcw.get(0), ccw.get(0))
                    {
                        if *desired_column_width > *calculated_column_width {
                            Cow::Borrowed(&cpu.short_cpu_name)
                        } else {
                            Cow::Borrowed(&cpu.cpu_name)
                        }
                    } else {
                        Cow::Borrowed(&cpu.cpu_name)
                    };
                let truncated_legend: Cow<'_, str> =
                    if let Some(calculated_column_width) = ccw.get(0) {
                        if *calculated_column_width == 0 && cpu.legend_value.is_empty() {
                            Cow::Borrowed("All")
                        } else {
                            Cow::Borrowed(&cpu.legend_value)
                        }
                    } else {
                        Cow::Borrowed(&cpu.legend_value)
                    };

                let cpu_string_row: Vec<Cow<'_, str>> = vec![truncated_name, truncated_legend];

                if cpu_string_row.is_empty() {
                    offset_scroll_index += 1;
                    None
                } else {
                    Some(Row::StyledData(
                        cpu_string_row.into_iter(),
                        if itx == offset_scroll_index {
                            self.colours.currently_selected_text_style
                        } else if itx + start_position == ALL_POSITION {
                            self.colours.all_colour_style
                        } else if show_avg_cpu {
                            if itx + start_position == AVG_POSITION {
                                self.colours.avg_colour_style
                            } else {
                                self.colours.cpu_colour_styles[itx + start_position
                                    - AVG_POSITION
                                    - 1 % self.colours.cpu_colour_styles.len()]
                            }
                        } else {
                            self.colours.cpu_colour_styles[itx + start_position
                                - ALL_POSITION
                                - 1 % self.colours.cpu_colour_styles.len()]
                        },
                    ))
                }
            });

            // Note we don't set highlight_style, as it should always be shown for this widget.
            let border_and_title_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            // Draw
            f.render_stateful_widget(
                Table::new(CPU_LEGEND_HEADER.iter(), cpu_rows)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(border_and_title_style),
                    )
                    .header_style(self.colours.table_header_style)
                    .highlight_style(self.colours.currently_selected_text_style)
                    .widths(
                        &(cpu_widget_state
                            .table_width_state
                            .calculated_column_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    )
                    .header_gap(table_gap),
                draw_loc,
                cpu_table_state,
            );
        }
    }
}
