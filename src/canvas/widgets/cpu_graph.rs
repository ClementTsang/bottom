use lazy_static::lazy_static;
use std::borrow::Cow;
use std::cmp::max;

use crate::{
    app::{layout_manager::WidgetDirection, App},
    canvas::{
        drawing_utils::{get_start_position, get_variable_intrinsic_widths},
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
    widgets::{Axis, Block, Borders, Chart, Dataset, Row, Table},
};

const CPU_SELECT_LEGEND_HEADER: [&str; 2] = ["CPU", "Show"];
const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
const AVG_POSITION: usize = 1;
const ALL_POSITION: usize = 0;

lazy_static! {
    static ref CPU_LEGEND_HEADER_LENS: Vec<usize> = CPU_LEGEND_HEADER
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref CPU_SELECT_LEGEND_HEADER_LENS: Vec<usize> = CPU_SELECT_LEGEND_HEADER
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
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
                .constraints(constraints.as_ref())
                .split(draw_loc);

            self.draw_cpu_graph(f, app_state, partitioned_draw_loc[graph_index], widget_id);
            self.draw_cpu_legend(
                f,
                app_state,
                partitioned_draw_loc[legend_index],
                widget_id + 1,
            );
        }
    }

    fn draw_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&widget_id) {
            let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;

            // let display_time_labels = vec![
            //     Text::styled(
            //         format!("{}s", cpu_widget_state.current_display_time / 1000),
            //         self.colours.graph_style,
            //     ),
            //     Text::styled("0s".to_string(), self.colours.graph_style),
            // ];

            let display_time_labels = [
                format!("{}s", cpu_widget_state.current_display_time / 1000),
                "0s".to_string(),
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
                        .labels(&display_time_labels)
                        .labels_style(self.colours.graph_style)
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
                    .labels(&display_time_labels)
                    .labels_style(self.colours.graph_style)
            };

            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .bounds([0.0, 100.5])
                .labels_style(self.colours.graph_style)
                .labels(&["0%", "100%"]);

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

            let border_style = if app_state.current_widget.widget_id == widget_id {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            // let title = if app_state.is_expanded {
            //     Span::styled(" CPU ".to_string(), border_style)
            // } else {
            //     Span::styled(" CPU ".to_string(), self.colours.widget_title_style)
            // };
            let title = " CPU ";
            let title_style = if app_state.is_expanded {
                border_style
            } else {
                self.colours.widget_title_style
            };

            f.render_widget(
                // Chart::new(dataset_vector)
                Chart::default()
                    .datasets(&dataset_vector)
                    .block(
                        Block::default()
                            .title(title)
                            .title_style(title_style)
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
        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&(widget_id - 1))
        {
            cpu_widget_state.is_legend_hidden = false;
            let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;

            let start_position = get_start_position(
                usize::from(draw_loc.height.saturating_sub(self.table_height_offset)),
                &cpu_widget_state.scroll_state.scroll_direction,
                &mut cpu_widget_state.scroll_state.previous_scroll_position,
                cpu_widget_state.scroll_state.current_scroll_position,
                app_state.is_force_redraw,
            );
            let is_on_widget = widget_id == app_state.current_widget.widget_id;

            let sliced_cpu_data = &cpu_data[start_position..];

            let mut offset_scroll_index =
                cpu_widget_state.scroll_state.current_scroll_position - start_position;
            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;

            let cpu_rows = sliced_cpu_data.iter().enumerate().filter_map(|(itx, cpu)| {
                let cpu_string_row: Vec<Cow<'_, str>> = vec![
                    Cow::Borrowed(&cpu.cpu_name),
                    Cow::Borrowed(&cpu.legend_value),
                ];

                if cpu_string_row.is_empty() {
                    offset_scroll_index += 1;
                    None
                } else {
                    Some(Row::StyledData(
                        cpu_string_row.into_iter(),
                        if itx == offset_scroll_index {
                            self.colours.currently_selected_text_style
                        } else if itx == ALL_POSITION {
                            self.colours.all_colour_style
                        } else if show_avg_cpu {
                            if itx == AVG_POSITION {
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

            // Calculate widths
            let width = f64::from(draw_loc.width);
            let width_ratios = vec![0.5, 0.5];

            let variable_intrinsic_results =
                get_variable_intrinsic_widths(width as u16, &width_ratios, &CPU_LEGEND_HEADER_LENS);
            let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

            let (border_and_title_style, highlight_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };

            // Draw
            f.render_widget(
                Table::new(CPU_LEGEND_HEADER.iter(), cpu_rows)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(border_and_title_style),
                    )
                    .header_style(self.colours.table_header_style)
                    .highlight_style(highlight_style)
                    .widths(
                        &(intrinsic_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    )
                    .header_gap(app_state.app_config_fields.table_gap),
                draw_loc,
            );
        }
    }
}
