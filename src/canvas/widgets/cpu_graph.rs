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
    symbols::Marker,
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph, Row, Table, Text},
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
    fn draw_multi_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
        avg_on_left: bool,
    );
}

impl CpuGraphWidget for Painter {
    fn draw_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let cpu_widget_state = match app_state.cpu_state.widget_states.get_mut(&widget_id) {
            Some(it) => it,
            _ => return,
        };
        let is_multi_graph = cpu_widget_state.is_multi_graph_mode;
        if is_multi_graph {
            self.draw_multi_cpu_graph(f, app_state, draw_loc, widget_id, true);
        } else if draw_loc.width as f64 * 0.15 <= 6.0 {
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
        use std::convert::TryFrom;

        let cpu_widget_state = match app_state.cpu_state.widget_states.get_mut(&widget_id) {
            Some(it) => it,
            _ => return,
        };
        let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;
        let border_style = if app_state.current_widget.widget_id == widget_id {
            self.colours.highlighted_border_style
        } else {
            self.colours.border_style
        };
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
        } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
            Axis::default().bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
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
        let dataset_vector: Vec<Dataset<'_>> = if let Ok(current_scroll_position) =
            usize::try_from(cpu_widget_state.scroll_state.current_scroll_position)
        {
            if current_scroll_position == ALL_POSITION {
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
            }
        } else {
            vec![]
        };

        let title = if app_state.is_expanded {
            const TITLE_BASE: &str = " CPU ── Esc to go back ";
            format!(
                " CPU ─{}─ Esc to go back ",
                "─".repeat(
                    usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2)
                )
            )
        } else {
            " CPU ".to_string()
        };

        f.render_widget(
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
                .datasets(&dataset_vector),
            draw_loc,
        );
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
                            // .title_style(border_and_title_style)
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

    fn draw_multi_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
        avg_on_left: bool,
    ) {
        let cpu_widget_state = match app_state.cpu_state.widget_states.get_mut(&widget_id) {
            Some(it) => it,
            _ => return,
        };
        let cpu_data: &mut [ConvertedCpuData] = &mut app_state.canvas_data.cpu_data;
        let avg_data: &mut ConvertedCpuData = &mut app_state.canvas_data.avg_cpu_data;
        let load_avg = &app_state.canvas_data.load_avg;
        let cpu_info = &app_state.canvas_data.cpu_info;
        let border_style = if app_state.current_widget.widget_id == widget_id {
            self.colours.highlighted_border_style
        } else {
            self.colours.border_style
        };
        let title = if app_state.is_expanded {
            const TITLE_BASE: &str = " CPU ── Esc to go back ";
            format!(
                " CPU ─{}─ Esc to go back ",
                "─".repeat(
                    usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2)
                )
            )
        } else {
            " CPU ".to_string()
        };
        let block = Block::default()
            .title(&title)
            .title_style(if app_state.is_expanded {
                border_style
            } else {
                self.colours.widget_title_style
            })
            .borders(Borders::ALL)
            .border_style(border_style);
        f.render_widget(block, draw_loc);

        const SMALL_SIDE: Constraint = Constraint::Percentage(25);
        const LARGE_SIDE: Constraint = Constraint::Percentage(75);
        let draw_locs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if avg_on_left {
                [LARGE_SIDE, SMALL_SIDE]
            } else {
                [SMALL_SIDE, LARGE_SIDE]
            })
            .split(draw_loc);

        let x_axis: Axis<'_, &str> = Axis::default()
            .bounds([-(cpu_widget_state.current_display_time as f64), 0.0])
            .style(self.colours.graph_style)
            .labels(&["", ""]);
        let y_axis: Axis<'_, &str> = Axis::default()
            .bounds([-0.5, 100.5])
            .style(self.colours.graph_style)
            .labels(&["", ""]);

        // Now let's create n-datasets
        let use_dot = app_state.app_config_fields.use_dot;
        // let show_avg_cpu = app_state.app_config_fields.show_average_cpu;
        // let hidden_cpu_offset = if show_avg_cpu { 2 } else { 1 };
        // let sliced_data = cpu_data[hidden_cpu_offset..].as_ref();
        // let dataset_set: Vec<(&str, [Dataset<'_>; 1])> = sliced_data
        //     .iter()
        //     .enumerate()
        //     .map(|(itx, cpu)| {
        //         (
        //             cpu.cpu_name.as_str(),
        //             [Dataset::default()
        //                 .marker(if use_dot {
        //                     Marker::Dot
        //                 } else {
        //                     Marker::Braille
        //                 })
        //                 .style(
        //                     self.colours.cpu_colour_styles
        //                         [itx % self.colours.cpu_colour_styles.len()],
        //                 )
        //                 .data(&cpu.cpu_data[..])
        //                 .graph_type(tui::widgets::GraphType::Line)],
        //         )
        //     })
        //     .collect();

        // const CPUS_PER_ROW: usize = 4;
        // let num_rows = (dataset_set.len() - 1) / CPUS_PER_ROW + 1;

        let (chart_draw_loc, side_draw_loc) = if avg_on_left {
            (draw_locs[0], draw_locs[1])
        } else {
            (draw_locs[1], draw_locs[0])
        };

        // let chart_draw_locs = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints(vec![Constraint::Ratio(1, num_rows as u32); num_rows].as_ref())
        //     .vertical_margin(1)
        //     .split(chart_draw_loc)
        //     .into_iter()
        //     .map(|area| {
        //         Layout::default()
        //             .direction(Direction::Horizontal)
        //             .constraints(vec![Constraint::Percentage(25); 4].as_ref())
        //             .split(area)
        //             .into_iter()
        //             .map(|rect| {
        //                 Layout::default()
        //                     .constraints([Constraint::Percentage(100)].as_ref())
        //                     .horizontal_margin(1)
        //                     .split(rect)[0]
        //             })
        //             .collect()
        //     })
        //     .collect::<Vec<Vec<Rect>>>();

        // for (row, row_draw_loc) in dataset_set[..].chunks(CPUS_PER_ROW).zip(chart_draw_locs) {
        //     for (col, col_draw_loc) in row.iter().zip(row_draw_loc) {
        //         f.render_widget(
        //             Chart::default()
        //                 .x_axis(x_axis.clone())
        //                 .y_axis(y_axis.clone())
        //                 .datasets(&col.1),
        //             col_draw_loc,
        //         );
        //     }
        // }

        let side_draw_loc = Layout::default()
            .vertical_margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(side_draw_loc)[0];
        let side_block = Block::default().borders(if avg_on_left {
            Borders::LEFT
        } else {
            Borders::RIGHT
        });
        f.render_widget(side_block, side_draw_loc);

        // TODO: Remove newlines if not enough space.
        // TODO: What is shown should be configurable eventually... but evaluating what is shown on each run kinda sucks.  Is there a way to determine what is needed to display, ONCE?
        let cpu_items = [
            Text::Styled(
                format!("AVG: {}\n", &avg_data.legend_value).into(),
                self.colours.avg_colour_style,
            ),
            Text::Styled(
                format!("CPU: {}\n", cpu_info.name).into(),
                self.colours.text_style,
            ),
            Text::Styled(format!("Utilization:\n").into(), self.colours.text_style),
            Text::Styled(format!("Frequency:\n").into(), self.colours.text_style),
            // Text::Styled(format!("Processes").into(), self.colours.text_style),
            // Text::Styled(format!("Threads").into(), self.colours.text_style),
            // Text::Styled(format!("Cores").into(), self.colours.text_style),
            // Text::Styled(format!("Logical Cores").into(), self.colours.text_style),
            Text::Raw("\n".into()),
            Text::Styled(format!("Uptime:\n").into(), self.colours.text_style),
            Text::Styled(
                format!(
                    "Load AVG: {}, {}, {}", // TODO: Test this on Windows...
                    load_avg.one, load_avg.five, load_avg.fifteen
                )
                .into(),
                self.colours.text_style,
            ),
        ];

        // Now let's also draw the average CPU chart and details...
        let side_locs = Layout::default()
            .horizontal_margin(2)
            .constraints([
                // Just for future reference (or for anyone else wondering what the
                // heck this does), this trick allows you to basically set a flex length
                // with another hard constraint.
                Constraint::Min(0),
                Constraint::Length(cpu_items.len() as u16),
            ])
            .split(side_draw_loc);
        let avg_dataset = (
            avg_data.cpu_name.as_str(),
            [Dataset::default()
                .marker(if use_dot {
                    Marker::Dot
                } else {
                    Marker::Braille
                })
                .style(self.colours.avg_colour_style)
                .data(&avg_data.cpu_data[..])
                .graph_type(tui::widgets::GraphType::Line)],
        );
        f.render_widget(
            Chart::default()
                .x_axis(x_axis.clone())
                .y_axis(y_axis.clone())
                .datasets(&avg_dataset.1),
            side_locs[0],
        );

        f.render_widget(Paragraph::new(cpu_items.iter()).wrap(true), side_locs[1]);
    }
}
