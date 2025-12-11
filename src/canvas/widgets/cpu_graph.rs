use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    app::{App, GraphStyle, data::StoredData, layout_manager::WidgetDirection},
    canvas::{
        Painter,
        components::{
            data_table::{DrawInfo, SelectionState},
            time_graph::{GraphData, variants::percent::PercentTimeGraph},
        },
        drawing_utils::should_hide_x_label,
    },
    collection::cpu::CpuData,
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
        &self, cpu_widget_state: &'a CpuWidgetState, data: &'a StoredData, show_avg_cpu: bool,
        filled: bool,
    ) -> Vec<GraphData<'a>> {
        let show_avg_offset = if show_avg_cpu { AVG_POSITION } else { 0 };
        let current_scroll_position = cpu_widget_state.table.state.current_index;
        let cpu_entries = &data.cpu_harvest;
        let cpu_points = &data.timeseries_data.cpu;
        let time = &data.timeseries_data.time;

        if current_scroll_position == ALL_POSITION {
            // This case ensures the other cases cannot have the position be equal to 0.

            cpu_points
                .iter()
                .enumerate()
                .map(|(itx, values)| {
                    let is_avg = show_avg_cpu && itx == 0;
                    let style = if is_avg {
                        self.styles.avg_cpu_colour
                    } else {
                        self.styles.cpu_colour_styles
                            [(itx - show_avg_offset) % self.styles.cpu_colour_styles.len()]
                    };

                    GraphData::default()
                        .style(style)
                        .time(time)
                        .values(values)
                        .filled(filled && is_avg)
                })
                .rev()
                .collect()
        } else if let Some(CpuData { .. }) = cpu_entries.get(current_scroll_position - 1) {
            // We generally subtract one from current scroll position because of the all entry.
            // TODO: Do this a bit better (e.g. we can just do if let Some(_) = cpu_points.get())

            let style = if show_avg_cpu && current_scroll_position == AVG_POSITION {
                self.styles.avg_cpu_colour
            } else {
                let offset_position = current_scroll_position - 1;
                self.styles.cpu_colour_styles
                    [(offset_position - show_avg_offset) % self.styles.cpu_colour_styles.len()]
            };

            vec![
                GraphData::default()
                    .style(style)
                    .time(time)
                    .values(&cpu_points[current_scroll_position - 1])
                    .filled(filled),
            ]
        } else {
            vec![]
        }
    }

    fn draw_cpu_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(cpu_widget_state) = app_state.states.cpu_state.widget_states.get_mut(&widget_id)
        {
            let data = app_state.data_store.get_data();

            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut cpu_widget_state.autohide_timer,
                draw_loc,
            );

            let filled = matches!(app_state.app_config_fields.graph_style, GraphStyle::Filled);

            let graph_data = self.generate_points(
                cpu_widget_state,
                data,
                app_state.app_config_fields.show_average_cpu,
                filled,
            );

            // TODO: Maybe hide load avg if too long? Or maybe the CPU part.
            let title = {
                #[cfg(target_family = "unix")]
                {
                    let load_avg = &data.load_avg_harvest;
                    let load_avg_str = format!(
                        "─ {:.2} {:.2} {:.2} ",
                        load_avg[0], load_avg[1], load_avg[2]
                    );

                    if data.cpu_harvest.brand.is_empty() {
                        concat_string::concat_string!(" CPU ", load_avg_str).into()
                    } else {
                        concat_string::concat_string!(
                            " CPU: ",
                            data.cpu_harvest.brand,
                            " ",
                            load_avg_str
                        )
                        .into()
                    }
                }
                #[cfg(not(target_family = "unix"))]
                {
                    if data.cpu_harvest.brand.is_empty() {
                        " CPU ".into()
                    } else {
                        concat_string::concat_string!(" CPU: ", data.cpu_harvest.brand, " ").into()
                    }
                }
            };

            PercentTimeGraph {
                display_range: cpu_widget_state.current_display_time,
                hide_x_labels,
                app_config_fields: &app_state.app_config_fields,
                current_widget: app_state.current_widget.widget_id,
                is_expanded: app_state.is_expanded,
                title,
                styles: &self.styles,
                widget_id,
                legend_position: None,
                legend_constraints: None,
                borders: tui::widgets::Borders::ALL,
            }
            .build()
            .draw(f, draw_loc, graph_data);
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
