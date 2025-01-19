use std::{borrow::Cow, time::Instant};

use tui::{
    layout::{Constraint, Rect},
    style::Style,
    symbols::Marker,
    Frame,
};

use crate::{
    app::{data::Values, App},
    canvas::{
        components::time_graph::{GraphData, TimeGraph},
        drawing_utils::should_hide_x_label,
        Painter,
    },
    data_collection::memory::MemHarvest,
    get_binary_unit_and_denominator,
};

/// Convert memory info into a combined memory label.
#[inline]
fn memory_legend_label(name: &str, data: Option<&MemHarvest>) -> String {
    if let Some(data) = data {
        if data.total_bytes > 0 {
            let percentage = data.used_bytes as f64 / data.total_bytes as f64 * 100.0;
            let (unit, denominator) = get_binary_unit_and_denominator(data.total_bytes);
            let used = data.used_bytes as f64 / denominator;
            let total = data.total_bytes as f64 / denominator;

            format!("{name}:{percentage:3.0}%   {used:.1}{unit}/{total:.1}{unit}")
        } else {
            format!("{name}:   0%   0.0B/0.0B")
        }
    } else {
        format!("{name}:   0%   0.0B/0.0B")
    }
}

/// FIXME: Glue code to convert from timeseries data to points. This does some automatic work such that it'll only keep
/// the needed points.
///
/// This should be slated to be removed and functionality moved to the graph drawing outright. We should also
/// just not cache and filter aggressively via the iter and bounds. We may also need to change the iter/graph to go
/// from current_time_in_ms - 60000 to current_time_in_ms, reducing the amount of work.
fn to_points(time: &[Instant], values: &Values, left_edge: f64) -> Vec<(f64, f64)> {
    let Some(iter) = values.iter_along_base(time) else {
        return vec![];
    };

    let Some(current_time) = time.last() else {
        return vec![];
    };

    let mut take_while_done = false;

    iter.rev()
        .map(|(&time, &val)| {
            let from_start: f64 = (current_time.duration_since(time).as_millis() as f64).floor();
            (-from_start, val)
        })
        .take_while(|(time, _)| {
            // We do things like this so we can take one extra value AFTER (needed for interpolation).
            if *time >= left_edge {
                true
            } else if !take_while_done {
                take_while_done = true;
                true
            } else {
                false
            }
        })
        .collect()
}

/// Get graph data from.
#[inline]
fn graph_data<'a>(
    out: &mut Vec<GraphData<'a>>, name: &str, last_harvest: Option<&'a MemHarvest>,
    points: &'a [(f64, f64)], style: Style,
) {
    if !points.is_empty() {
        let label = Some(memory_legend_label(name, last_harvest).into());

        out.push(GraphData {
            points,
            style,
            name: label,
        });
    }
}

impl Painter {
    pub fn draw_memory_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        const Y_BOUNDS: [f64; 2] = [0.0, 100.5];
        const Y_LABELS: [Cow<'static, str>; 2] = [Cow::Borrowed("  0%"), Cow::Borrowed("100%")];

        if let Some(mem_state) = app_state.states.mem_state.widget_states.get_mut(&widget_id) {
            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let x_min = -(mem_state.current_display_time as f64);
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut mem_state.autohide_timer,
                draw_loc,
            );
            let points = {
                let mut size = 1;
                let data = app_state.data_store.get_data();

                if data.swap_harvest.is_some() {
                    size += 1; // add capacity for SWAP
                }
                #[cfg(feature = "zfs")]
                {
                    if app_state.converted_data.arc_labels.is_some() {
                        size += 1; // add capacity for ARC
                    }
                }
                #[cfg(feature = "gpu")]
                {
                    if let Some(gpu_data) = &app_state.converted_data.gpu_data {
                        size += gpu_data.len(); // add row(s) for gpu
                    }
                }

                let data = app_state.data_store.get_data();
                let mut points = Vec::with_capacity(size);

                mem_state.ram_points_cache =
                    to_points(&data.timeseries_data.time, &data.timeseries_data.mem, x_min);
                graph_data(
                    &mut points,
                    "RAM",
                    Some(&data.ram_harvest),
                    &mem_state.ram_points_cache,
                    self.styles.ram_style,
                );

                mem_state.swap_points_cache = to_points(
                    &data.timeseries_data.time,
                    &data.timeseries_data.swap,
                    x_min,
                );
                graph_data(
                    &mut points,
                    "SWP",
                    data.swap_harvest.as_ref(),
                    &mem_state.swap_points_cache,
                    self.styles.swap_style,
                );

                #[cfg(not(target_os = "windows"))]
                graph_data(
                    &mut points,
                    "CHE",
                    data.cache_harvest.as_ref(),
                    &app_state.converted_data.cache_data,
                    self.styles.cache_style,
                );

                #[cfg(feature = "zfs")]
                if let Some((label_percent, label_frac)) = &app_state.converted_data.arc_labels {
                    let arc_label = format!("ARC:{label_percent}{label_frac}");
                    points.push(GraphData {
                        points: &app_state.converted_data.arc_data,
                        style: self.styles.arc_style,
                        name: Some(arc_label.into()),
                    });
                }
                #[cfg(feature = "gpu")]
                {
                    if let Some(gpu_data) = &app_state.converted_data.gpu_data {
                        let mut color_index = 0;
                        let gpu_styles = &self.styles.gpu_colours;
                        gpu_data.iter().for_each(|gpu| {
                            let gpu_label =
                                format!("{}:{}{}", gpu.name, gpu.mem_percent, gpu.mem_total);
                            let style = {
                                if gpu_styles.is_empty() {
                                    tui::style::Style::default()
                                } else if color_index >= gpu_styles.len() {
                                    // cycle styles
                                    color_index = 1;
                                    gpu_styles[color_index - 1]
                                } else {
                                    color_index += 1;
                                    gpu_styles[color_index - 1]
                                }
                            };
                            points.push(GraphData {
                                points: gpu.points.as_slice(),
                                style,
                                name: Some(gpu_label.into()),
                            });
                        });
                    }
                }

                points
            };

            let marker = if app_state.app_config_fields.use_dot {
                Marker::Dot
            } else {
                Marker::Braille
            };

            TimeGraph {
                x_min,
                hide_x_labels,
                y_bounds: Y_BOUNDS,
                y_labels: &Y_LABELS,
                graph_style: self.styles.graph_style,
                border_style,
                border_type: self.styles.border_type,
                title: " Memory ".into(),
                is_selected: app_state.current_widget.widget_id == widget_id,
                is_expanded: app_state.is_expanded,
                title_style: self.styles.widget_title_style,
                legend_position: app_state.app_config_fields.memory_legend_position,
                legend_constraints: Some((Constraint::Ratio(3, 4), Constraint::Ratio(3, 4))),
                marker,
            }
            .draw_time_graph(f, draw_loc, &points);
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
