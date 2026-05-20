use std::time::Instant;

use tui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
};

use crate::{
    app::{App, data::Values},
    canvas::{
        Painter,
        components::time_series::{GraphData, LegendConstraints},
        drawing_utils::should_hide_x_label,
    },
    collection::memory::MemData,
    components::time_series::GraphDrawCtx,
    get_binary_unit_and_denominator,
};

/// Convert memory info into a combined memory label.
#[inline]
fn memory_legend_label(name: &str, data: Option<&MemData>) -> String {
    if let Some(data) = data {
        let total_bytes = data.total_bytes.get();
        let percentage = data.used_bytes as f64 / total_bytes as f64 * 100.0;
        let (unit, denominator) = get_binary_unit_and_denominator(total_bytes);
        let used = data.used_bytes as f64 / denominator;
        let total = total_bytes as f64 / denominator;

        format!("{name}:{percentage:3.0}%   {used:.1}{unit}/{total:.1}{unit}")
    } else {
        format!("{name}:   0%   0.0B/0.0B")
    }
}

/// Get graph data.
#[inline]
fn graph_data<'a>(
    out: &mut Vec<GraphData<'a>>, name: &str, last_harvest: Option<&'a MemData>,
    time: &'a [Instant], values: &'a Values, style: Style,
) {
    if !values.no_elements() {
        let label = memory_legend_label(name, last_harvest).into();

        out.push(
            GraphData::default()
                .name(label)
                .time(time)
                .values(values)
                .style(style),
        );
    }
}

impl Painter {
    pub fn draw_memory_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(mem_state) = app_state.states.mem_state.widget_states.get_mut(&widget_id) {
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                mem_state.graph.state_mut().autohide_timer_mut(),
                draw_loc,
            );
            let graph_data = {
                let mut size = 1;
                let data = app_state.data_store.get_data();

                // TODO: is this optimization really needed...? This just pre-allocates a vec,
                // but it'll probably never be that big...

                if data.swap_harvest.is_some() {
                    size += 1; // add capacity for SWAP
                }
                #[cfg(feature = "zfs")]
                {
                    if data.arc_harvest.is_some() {
                        size += 1; // add capacity for ARC
                    }
                }
                #[cfg(feature = "gpu")]
                {
                    size += data.gpu_harvest.len(); // add row(s) for gpu
                }

                let mut points = Vec::with_capacity(size);
                let time_series = &data.time_series_data;
                let time = &time_series.time;

                // TODO: Add a "no data" option here/to time graph if there is no entries
                graph_data(
                    &mut points,
                    "RAM",
                    data.ram_harvest.as_ref(),
                    time,
                    &time_series.ram,
                    self.styles.ram_style,
                );

                graph_data(
                    &mut points,
                    "SWP",
                    data.swap_harvest.as_ref(),
                    time,
                    &time_series.swap,
                    self.styles.swap_style,
                );

                #[cfg(not(target_os = "windows"))]
                {
                    graph_data(
                        &mut points,
                        "CACHE", // TODO: Figure out how to line this up better
                        data.cache_harvest.as_ref(),
                        time,
                        &time_series.cache_mem,
                        self.styles.cache_style,
                    );
                }

                #[cfg(feature = "zfs")]
                {
                    graph_data(
                        &mut points,
                        "ARC",
                        data.arc_harvest.as_ref(),
                        time,
                        &time_series.arc_mem,
                        self.styles.arc_style,
                    );
                }

                #[cfg(feature = "gpu")]
                {
                    let mut colour_index = 0;
                    let gpu_styles = &self.styles.gpu_colours;

                    for (name, harvest) in &data.gpu_harvest {
                        if let Some(gpu_data) = data.time_series_data.gpu_mem.get(name) {
                            let style = {
                                if gpu_styles.is_empty() {
                                    Style::default()
                                } else {
                                    let colour = gpu_styles[colour_index % gpu_styles.len()];
                                    colour_index += 1;

                                    colour
                                }
                            };

                            let display_name = if data.gpu_harvest.len() == 1 {
                                &"GPU".to_string()
                            } else {
                                name // TODO: REALLY figure out how to line this up better
                            };

                            graph_data(
                                &mut points,
                                display_name,
                                Some(harvest),
                                time,
                                gpu_data,
                                style,
                            );
                        }
                    }
                }

                points
            };

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let marker = self.get_marker(app_state.app_config_fields.use_dot);

            mem_state.graph.draw(
                f,
                draw_loc,
                GraphDrawCtx {
                    title: " Memory ".into(),
                    border_style,
                    title_style: self.styles.widget_title_style,
                    graph_style: self.styles.graph_style,
                    general_widget_style: self.styles.general_widget_style,
                    border_type: self.styles.border_type,
                    marker,
                    hide_x_labels,
                    is_selected: app_state.current_widget.widget_id == widget_id,
                    is_expanded: app_state.is_expanded,
                    legend_position: app_state.app_config_fields.memory_legend_position,
                    legend_constraints: Some(LegendConstraints {
                        width: Constraint::Ratio(3, 4),
                        height: Constraint::Ratio(3, 4),
                    }),
                },
                graph_data,
            );
        }

        if app_state.should_get_widget_bounds() {
            // Update draw loc in widget map
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}
