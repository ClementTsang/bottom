use std::borrow::Cow;

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    Frame,
};

use crate::{
    app::{layout_manager::WidgetDirection, App},
    canvas::{
        components::{
            data_table::{DrawInfo, SelectionState},
            time_graph::{GraphData, TimeGraph},
        },
        drawing_utils::should_hide_x_label,
        Painter,
    },
    data_conversion::CpuWidgetData,
    widgets::CpuWidgetState,
};

const AVG_POSITION: usize = 1;
const ALL_POSITION: usize = 0;

impl Painter {
    pub fn draw_cpu(&self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64) {
        let legend_width = (draw_loc.width as f64 * 0.15) as u16;

        if legend_width < 6 {
            // Skip drawing legend
            if app_state.current_widget.widget_id == (widget_id + 1) {
                if app_state.app_config_fields.cpu_left_legend {
                    app_state.move_widget_selection(&WidgetDirection::Right);
                } else {
                    app_state.move_widget_selection(&WidgetDirection::Left);
                }
            }
            self.draw_cpu_graph(f, app_state, draw_loc, widget_id);
            if let Some(cpu_widget_state) =
                app_state.states.cpu_state.widget_states.get_mut(&widget_id)
            {
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
                if app_state.app_config_fields.cpu_left_legend {
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
        &self, cpu_widget_state: &CpuWidgetState, cpu_data: &'a [CpuWidgetData], show_avg_cpu: bool,
    ) -> Vec<GraphData<'a>> {
        let show_avg_offset = if show_avg_cpu { AVG_POSITION } else { 0 };

        let current_scroll_position = cpu_widget_state.table.state.current_index;
        if current_scroll_position == ALL_POSITION {
            // This case ensures the other cases cannot have the position be equal to 0.
            cpu_data
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(itx, cpu)| {
                    match &cpu {
                        CpuWidgetData::All => None,
                        CpuWidgetData::Entry { data, .. } => {
                            let style = if show_avg_cpu && itx == AVG_POSITION {
                                self.colours.avg_cpu_colour
                            } else if itx == ALL_POSITION {
                                self.colours.all_cpu_colour
                            } else {
                                let offset_position = itx - 1; // Because of the all position
                                self.colours.cpu_colour_styles[(offset_position - show_avg_offset)
                                    % self.colours.cpu_colour_styles.len()]
                            };

                            Some(GraphData {
                                points: &data[..],
                                style,
                                name: None,
                            })
                        }
                    }
                })
                .collect::<Vec<_>>()
        } else if let Some(CpuWidgetData::Entry { data, .. }) =
            cpu_data.get(current_scroll_position)
        {
            let style = if show_avg_cpu && current_scroll_position == AVG_POSITION {
                self.colours.avg_cpu_colour
            } else {
                let offset_position = current_scroll_position - 1; // Because of the all position
                self.colours.cpu_colour_styles
                    [(offset_position - show_avg_offset) % self.colours.cpu_colour_styles.len()]
            };

            vec![GraphData {
                points: &data[..],
                style,
                name: None,
            }]
        } else {
            vec![]
        }
    }

    fn draw_cpu_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        const Y_BOUNDS: [f64; 2] = [0.0, 100.5];
        const Y_LABELS: [Cow<'static, str>; 2] = [Cow::Borrowed("  0%"), Cow::Borrowed("100%")];

        if let Some(cpu_widget_state) = app_state.states.cpu_state.widget_states.get_mut(&widget_id)
        {
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
            let title = {
                #[cfg(target_family = "unix")]
                {
                    let load_avg = app_state.converted_data.load_avg_data;
                    let load_avg_str = format!(
                        "─ {:.2} {:.2} {:.2} ",
                        load_avg[0], load_avg[1], load_avg[2]
                    );

                    concat_string::concat_string!(" CPU ", load_avg_str).into()
                }
                #[cfg(not(target_family = "unix"))]
                {
                    " CPU ".into()
                }
            };

            let marker = if app_state.app_config_fields.use_dot {
                Marker::Dot
            } else {
                Marker::Braille
            };

            TimeGraph {
                x_bounds,
                hide_x_labels,
                y_bounds: Y_BOUNDS,
                y_labels: &Y_LABELS,
                graph_style: self.colours.graph_style,
                border_style,
                title,
                is_expanded: app_state.is_expanded,
                title_style: self.colours.widget_title_style,
                legend_position: None,
                legend_constraints: None,
                marker,
            }
            .draw_time_graph(f, draw_loc, &points);
        }
    }

    fn draw_cpu_legend(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(cpu_widget_state) = app_state
            .states
            .cpu_state
            .widget_states
            .get_mut(&(widget_id - 1))
        {
            // TODO: This line (and the one above, see caller) is pretty dumb but I guess
            // needed for now. Refactor if possible!
            cpu_widget_state.is_legend_hidden = false;

            let is_on_widget = widget_id == app_state.current_widget.widget_id;

            let draw_info = DrawInfo {
                loc: draw_loc,
                force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                selection_state: SelectionState::new(app_state.is_expanded, is_on_widget),
            };

            cpu_widget_state.table.draw(
                f,
                &draw_info,
                app_state.widget_map.get_mut(&widget_id),
                self,
            );
        }
    }
}
