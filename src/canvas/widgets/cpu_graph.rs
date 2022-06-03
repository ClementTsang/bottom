use std::{borrow::Cow, iter};

use crate::{
    app::{layout_manager::WidgetDirection, App, CpuWidgetState},
    canvas::{drawing_utils::should_hide_x_label, Painter},
    components::{
        text_table::{CellContent, TextTable},
        time_graph::{GraphData, TimeGraph},
    },
    data_conversion::{ConvertedCpuData, TableData, TableRow},
};

use concat_string::concat_string;

use itertools::Either;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
};

const AVG_POSITION: usize = 1;
const ALL_POSITION: usize = 0;

impl Painter {
    pub fn draw_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let legend_width = (draw_loc.width as f64 * 0.15) as u16;

        if legend_width < 6 {
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
            let graph_width = draw_loc.width - legend_width;
            let (graph_index, legend_index, constraints) =
                if app_state.app_config_fields.left_legend {
                    (
                        1,
                        0,
                        [
                            Constraint::Length(legend_width),
                            Constraint::Length(graph_width),
                        ],
                    )
                } else {
                    (
                        0,
                        1,
                        [
                            Constraint::Length(graph_width),
                            Constraint::Length(legend_width),
                        ],
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

    fn generate_points<'a>(
        &self, cpu_widget_state: &CpuWidgetState, cpu_data: &'a [ConvertedCpuData],
        show_avg_cpu: bool,
    ) -> Vec<GraphData<'a>> {
        let show_avg_offset = if show_avg_cpu { AVG_POSITION } else { 0 };

        let current_scroll_position = cpu_widget_state.table_state.current_scroll_position;
        if current_scroll_position == ALL_POSITION {
            // This case ensures the other cases cannot have the position be equal to 0.
            cpu_data
                .iter()
                .enumerate()
                .rev()
                .map(|(itx, cpu)| {
                    let style = if show_avg_cpu && itx == AVG_POSITION {
                        self.colours.avg_colour_style
                    } else if itx == ALL_POSITION {
                        self.colours.all_colour_style
                    } else {
                        let offset_position = itx - 1; // Because of the all position
                        self.colours.cpu_colour_styles[(offset_position - show_avg_offset)
                            % self.colours.cpu_colour_styles.len()]
                    };

                    GraphData {
                        points: &cpu.cpu_data[..],
                        style,
                        name: None,
                    }
                })
                .collect::<Vec<_>>()
        } else if let Some(cpu) = cpu_data.get(current_scroll_position) {
            let style = if show_avg_cpu && current_scroll_position == AVG_POSITION {
                self.colours.avg_colour_style
            } else {
                let offset_position = current_scroll_position - 1; // Because of the all position
                self.colours.cpu_colour_styles
                    [(offset_position - show_avg_offset) % self.colours.cpu_colour_styles.len()]
            };

            vec![GraphData {
                points: &cpu.cpu_data[..],
                style,
                name: None,
            }]
        } else {
            vec![]
        }
    }

    fn draw_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        const Y_BOUNDS: [f64; 2] = [0.0, 100.5];
        const Y_LABELS: [Cow<'static, str>; 2] = [Cow::Borrowed("  0%"), Cow::Borrowed("100%")];

        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&widget_id) {
            let cpu_data = &app_state.converted_data.cpu_data;
            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let x_bounds = [0, cpu_widget_state.current_display_time];
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut cpu_widget_state.autohide_timer,
                draw_loc,
            );

            let points = self.generate_points(
                cpu_widget_state,
                cpu_data,
                app_state.app_config_fields.show_average_cpu,
            );

            // TODO: Maybe hide load avg if too long? Or maybe the CPU part.
            let title = if cfg!(target_family = "unix") {
                let load_avg = app_state.converted_data.load_avg_data;
                let load_avg_str = format!(
                    "─ {:.2} {:.2} {:.2} ",
                    load_avg[0], load_avg[1], load_avg[2]
                );

                concat_string!(" CPU ", load_avg_str).into()
            } else {
                " CPU ".into()
            };

            TimeGraph {
                use_dot: app_state.app_config_fields.use_dot,
                x_bounds,
                hide_x_labels,
                y_bounds: Y_BOUNDS,
                y_labels: &Y_LABELS,
                graph_style: self.colours.graph_style,
                border_style,
                title,
                is_expanded: app_state.is_expanded,
                title_style: self.colours.widget_title_style,
                legend_constraints: None,
            }
            .draw_time_graph(f, draw_loc, &points);
        }
    }

    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(cpu_widget_state) = app_state.cpu_state.widget_states.get_mut(&(widget_id - 1))
        {
            // TODO: This line (and the one above, see caller) is pretty dumb but I guess needed.
            cpu_widget_state.is_legend_hidden = false;

            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;
            let cpu_data = {
                let col_widths = vec![1, 3]; // TODO: Should change this to take const generics (usize) and an array.
                let colour_iter = if show_avg_cpu {
                    Either::Left(
                        iter::once(&self.colours.all_colour_style)
                            .chain(iter::once(&self.colours.avg_colour_style))
                            .chain(self.colours.cpu_colour_styles.iter().cycle()),
                    )
                } else {
                    Either::Right(
                        iter::once(&self.colours.all_colour_style)
                            .chain(self.colours.cpu_colour_styles.iter().cycle()),
                    )
                };

                let data = {
                    let iter = app_state.converted_data.cpu_data.iter().zip(colour_iter);
                    const CPU_WIDTH_CHECK: u16 = 10; // This is hard-coded, it's terrible.
                    if draw_loc.width < CPU_WIDTH_CHECK {
                        Either::Left(iter.map(|(cpu, style)| {
                            let row = vec![
                                CellContent::Simple("".into()),
                                CellContent::Simple(if cpu.legend_value.is_empty() {
                                    cpu.cpu_name.clone().into()
                                } else {
                                    cpu.legend_value.clone().into()
                                }),
                            ];
                            TableRow::Styled(row, *style)
                        }))
                    } else {
                        Either::Right(iter.map(|(cpu, style)| {
                            let row = vec![
                                CellContent::HasAlt {
                                    alt: cpu.short_cpu_name.clone().into(),
                                    main: cpu.cpu_name.clone().into(),
                                },
                                CellContent::Simple(cpu.legend_value.clone().into()),
                            ];
                            TableRow::Styled(row, *style)
                        }))
                    }
                }
                .collect();

                TableData { data, col_widths }
            };

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let border_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            TextTable {
                table_gap: app_state.app_config_fields.table_gap,
                is_force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                header_style: self.colours.table_header_style,
                border_style,
                highlighted_text_style: self.colours.currently_selected_text_style, // We always highlight the selected CPU entry... not sure if I like this though.
                title: None,
                is_on_widget,
                draw_border: true,
                show_table_scroll_position: app_state.app_config_fields.show_table_scroll_position,
                title_style: self.colours.widget_title_style,
                text_style: self.colours.text_style,
                left_to_right: false,
            }
            .draw_text_table(
                f,
                draw_loc,
                &mut cpu_widget_state.table_state,
                &cpu_data,
                None,
            );
        }
    }
}
