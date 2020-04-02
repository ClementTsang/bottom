use lazy_static::lazy_static;
use std::borrow::Cow;
use std::cmp::max;

use crate::{
    app::App,
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
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
};

const CPU_SELECT_LEGEND_HEADER: [&str; 2] = ["CPU", "Show"];
const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
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
                    app_state.move_widget_selection_right();
                } else {
                    app_state.move_widget_selection_left();
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
                        .labels_style(self.colours.graph_style)
                        .labels(&display_time_labels)
                } else {
                    cpu_widget_state.autohide_timer = None;
                    Axis::default().bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
                }
            } else {
                Axis::default()
                    .bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
                    .style(self.colours.graph_style)
                    .labels_style(self.colours.graph_style)
                    .labels(&display_time_labels)
            };

            // Note this is offset as otherwise the 0 value is not drawn!
            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .labels_style(self.colours.graph_style)
                .bounds([-0.5, 100.5])
                .labels(&["0%", "100%"]);

            let use_dot = app_state.app_config_fields.use_dot;
            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;
            let dataset_vector: Vec<Dataset<'_>> = cpu_data
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(itx, cpu)| {
                    if cpu_widget_state.core_show_vec[itx] {
                        Some(
                            Dataset::default()
                                .marker(if use_dot {
                                    Marker::Dot
                                } else {
                                    Marker::Braille
                                })
                                .style(if show_avg_cpu && itx == 0 {
                                    self.colours.avg_colour_style
                                } else {
                                    self.colours.cpu_colour_styles
                                        [itx % self.colours.cpu_colour_styles.len()]
                                })
                                .data(&cpu.cpu_data[..]),
                        )
                    } else {
                        None
                    }
                })
                .collect();

            let title = if app_state.is_expanded && !cpu_widget_state.is_showing_tray {
                const TITLE_BASE: &str = " CPU ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title =
                    format!(" CPU ─{}─ Esc to go back ", "─".repeat(repeat_num as usize));

                result_title
            } else {
                " CPU ".to_string()
            };

            let border_style = if app_state.current_widget.widget_id == widget_id {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            Chart::default()
                .block(
                    Block::default()
                        .title(&title)
                        .title_style(if app_state.is_expanded {
                            border_style
                        } else {
                            self.colours.widget_title_style
                        })
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .x_axis(x_axis)
                .y_axis(y_axis)
                .datasets(&dataset_vector)
                .render(f, draw_loc);
        }
    }

    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&(widget_id - 1))
        {
            cpu_widget_state.is_legend_hidden = false;
            let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;

            let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
            let start_position = get_start_position(
                num_rows,
                &cpu_widget_state.scroll_state.scroll_direction,
                &mut cpu_widget_state.scroll_state.previous_scroll_position,
                cpu_widget_state.scroll_state.current_scroll_position,
                app_state.is_resized,
            );

            let sliced_cpu_data = &cpu_data[start_position as usize..];

            let mut offset_scroll_index =
                (cpu_widget_state.scroll_state.current_scroll_position - start_position) as usize;
            let show_disabled_data = app_state.app_config_fields.show_disabled_data;
            let current_widget_id = app_state.current_widget.widget_id;
            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;

            let cpu_rows = sliced_cpu_data.iter().enumerate().filter_map(|(itx, cpu)| {
                let cpu_string_row: Vec<Cow<'_, str>> = if cpu_widget_state.is_showing_tray {
                    vec![
                        Cow::Borrowed(&cpu.cpu_name),
                        if cpu_widget_state.core_show_vec[itx + start_position as usize] {
                            "[*]".into()
                        } else {
                            "[ ]".into()
                        },
                    ]
                } else if show_disabled_data || cpu_widget_state.core_show_vec[itx] {
                    vec![
                        Cow::Borrowed(&cpu.cpu_name),
                        Cow::Borrowed(&cpu.legend_value),
                    ]
                } else {
                    Vec::new()
                };

                if cpu_string_row.is_empty() {
                    offset_scroll_index += 1;
                    None
                } else {
                    Some(Row::StyledData(
                        cpu_string_row.into_iter(),
                        if current_widget_id == widget_id {
                            if itx == offset_scroll_index {
                                self.colours.currently_selected_text_style
                            } else if show_avg_cpu && itx == 0 {
                                self.colours.avg_colour_style
                            } else {
                                self.colours.cpu_colour_styles[itx
                                    + start_position as usize
                                        % self.colours.cpu_colour_styles.len()]
                            }
                        } else if show_avg_cpu && itx == 0 {
                            self.colours.avg_colour_style
                        } else {
                            self.colours.cpu_colour_styles[itx
                                + start_position as usize % self.colours.cpu_colour_styles.len()]
                        },
                    ))
                }
            });

            // Calculate widths
            let width = f64::from(draw_loc.width);
            let width_ratios = vec![0.5, 0.5];

            let variable_intrinsic_results = get_variable_intrinsic_widths(
                width as u16,
                &width_ratios,
                if cpu_widget_state.is_showing_tray {
                    &CPU_SELECT_LEGEND_HEADER_LENS
                } else {
                    &CPU_LEGEND_HEADER_LENS
                },
            );
            let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

            let title = if cpu_widget_state.is_showing_tray {
                const TITLE_BASE: &str = " Esc to close ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!("{} Esc to close ", "─".repeat(repeat_num as usize));

                result_title
            } else {
                "".to_string()
            };

            let title_and_border_style = if app_state.current_widget.widget_id == widget_id {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            // Draw
            Table::new(
                if cpu_widget_state.is_showing_tray {
                    CPU_SELECT_LEGEND_HEADER
                } else {
                    CPU_LEGEND_HEADER
                }
                .iter(),
                cpu_rows,
            )
            .block(
                Block::default()
                    .title(&title)
                    .title_style(title_and_border_style)
                    .borders(Borders::ALL)
                    .border_style(title_and_border_style),
            )
            .header_style(self.colours.table_header_style)
            .widths(
                &(intrinsic_widths
                    .iter()
                    .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                    .collect::<Vec<_>>()),
            )
            .render(f, draw_loc);
        }
    }
}
