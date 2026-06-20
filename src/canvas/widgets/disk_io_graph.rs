use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use rustc_hash::FxHashMap;
use timeless::data::ChunkedData;
use tui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
};

use crate::{
    app::App,
    canvas::{
        Painter,
        components::time_series::{AxisBound, ChartScaling, GraphData, LegendConstraints},
        drawing_utils::should_hide_x_label,
    },
    components::time_series::GraphDrawCtx,
    options::config::disk_io_graph::DiskGraphLegend,
    utils::data_units::*,
};

impl Painter {
    pub fn draw_disk_io_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(widget_state) = app_state
            .states
            .disk_io_graph_state
            .get_mut_widget_state(widget_id)
        {
            let shared_data = app_state.data_store.get_data();
            let read_data = &shared_data.time_series_data.disk_io_read;
            let write_data = &shared_data.time_series_data.disk_io_write;
            let times = &shared_data.time_series_data.time;

            let mount_map: FxHashMap<&str, &str> = shared_data
                .disk_harvest
                .iter()
                .map(|d| (d.name.as_str(), d.mount_point.as_str()))
                .collect();

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let graph_state = widget_state.graph.state_mut();
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                graph_state.autohide_timer_mut(),
                draw_loc,
            );
            let current_display_time = graph_state.current_display_time();

            let show_read = widget_state.show_read;
            let show_write = widget_state.show_write;
            let use_log = widget_state.use_log;
            let legend_type = &widget_state.legend;

            let mut device_names: Vec<&String> = read_data
                .keys()
                .filter(|name| {
                    let has_read_data = || {
                        read_data
                            .get(*name)
                            .is_some_and(|d| has_data_in_window(d, times, current_display_time))
                    };

                    // If there is a mount point and we're in mount legend mode, it must be non-empty
                    // (i.e. actually mounted), or we will short-circuit and ignore it.
                    if let Some(mount_point) = mount_map.get(name.as_str()) {
                        match legend_type {
                            DiskGraphLegend::Disk => true,
                            DiskGraphLegend::Mount => !mount_point.is_empty() && has_read_data(),
                        }
                    } else {
                        match legend_type {
                            DiskGraphLegend::Disk => {
                                // Otherwise, it may have _previously_ been a valid mount point, so keep showing it until it ages out.
                                has_read_data()
                            }
                            DiskGraphLegend::Mount => {
                                // Since it would be misleading in this case, just skip it in mount mode.
                                false
                            }
                        }
                    }
                })
                .collect();
            device_names.sort_unstable();

            // Compute y_max across all visible series.
            let mut visible_sources: Vec<&_> = Vec::new();
            for name in &device_names {
                if show_read {
                    if let Some(data) = read_data.get(*name) {
                        visible_sources.push(data);
                    }
                }
                if show_write {
                    if let Some(data) = write_data.get(*name) {
                        visible_sources.push(data);
                    }
                }
            }
            let y_max = widget_state.graph.y_max(visible_sources.into_iter(), times);

            let (adjusted_y_max, y_labels) = if use_log {
                adjust_disk_io_log(y_max)
            } else {
                adjust_disk_io_linear(y_max)
            };

            let legend_constraints = LegendConstraints {
                width: Constraint::Ratio(9, 10),
                height: Constraint::Ratio(1, 2),
            };

            // Removed devices still visible in the window show "N/A".
            // TODO: Maybe should make it so the colour is based on entry name? As then it may shift.
            let read_colours = &self.styles.disk_io_read_colour_styles;
            let write_colours = &self.styles.disk_io_write_colour_styles;

            // Pad the device/mount labels to the widest visible one so the rate columns
            // line up in the legend (the rate itself is already fixed-width).
            let name_width = device_names
                .iter()
                .map(|name| match legend_type {
                    DiskGraphLegend::Disk => name.len(),
                    DiskGraphLegend::Mount => mount_map
                        .get(name.as_str())
                        .map(|mount| mount.len())
                        .unwrap_or(0),
                })
                .max()
                .unwrap_or(0);

            let mut graph_data: Vec<GraphData<'_, f64>> =
                Vec::with_capacity(device_names.len() * 2);

            for (idx, name) in device_names.iter().enumerate() {
                let display_name = match legend_type {
                    DiskGraphLegend::Disk => name.as_str(),
                    DiskGraphLegend::Mount => match mount_map.get(name.as_str()).copied() {
                        Some(mount) => mount,
                        // This wouldn't trigger anyway, we filter out devices without mount points in mount legend mode.
                        None => continue,
                    },
                };

                let is_active = mount_map.contains_key(name.as_str());

                let read_values = show_read.then(|| read_data.get(*name)).flatten();
                let write_values = show_write.then(|| write_data.get(*name)).flatten();

                let read_style = if read_colours.is_empty() {
                    Style::default()
                } else {
                    read_colours[idx % read_colours.len()]
                };
                let write_style = if write_colours.is_empty() {
                    Style::default()
                } else {
                    write_colours[idx % write_colours.len()]
                };

                // TODO: Combine into one line; probably need to add some kind of multi-styled GraphData.
                if let Some(values) = read_values {
                    let rate = if is_active {
                        format_rate_fixed(values.last().copied().unwrap_or(0.0))
                    } else {
                        "        N/A".to_string()
                    };
                    graph_data.push(
                        GraphData::default()
                            .name(format!("{display_name:<name_width$} R:{rate}").into())
                            .style(read_style)
                            .time(times)
                            .values(values),
                    );
                }

                if let Some(values) = write_values {
                    let rate = if is_active {
                        format_rate_fixed(values.last().copied().unwrap_or(0.0))
                    } else {
                        "        N/A".to_string()
                    };
                    graph_data.push(
                        GraphData::default()
                            .name(format!("{display_name:<name_width$} W:{rate}").into())
                            .style(write_style)
                            .time(times)
                            .values(values),
                    );
                }
            }

            let marker = self.get_marker(app_state.app_config_fields.use_dot);
            let y_bounds = AxisBound::Max(adjusted_y_max);
            let scaling = if use_log {
                ChartScaling::Log2
            } else {
                ChartScaling::Linear
            };
            let y_labels: Vec<Cow<'_, str>> = y_labels.into_iter().map(Into::into).collect();

            widget_state.graph.draw(
                f,
                draw_loc,
                GraphDrawCtx {
                    title: " Disk I/O ".into(),
                    border_style,
                    title_style: self.styles.widget_title_style,
                    graph_style: self.styles.graph_style,
                    general_widget_style: self.styles.general_widget_style,
                    border_type: self.styles.border_type,
                    marker,
                    hide_x_labels,
                    is_selected: app_state.current_widget.widget_id == widget_id,
                    is_expanded: app_state.is_expanded,
                    legend_position: app_state.app_config_fields.disk_io_legend_position,
                    legend_constraints: Some(legend_constraints),
                },
                y_bounds,
                &y_labels,
                scaling,
                graph_data,
            );
        }

        if app_state.should_get_widget_bounds() {
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}

/// Returns true if `data` has at least one real (non-gap) data point within the
/// visible time window defined by `current_display_time` milliseconds from the end
/// of `times`.
fn has_data_in_window<F: Copy + Default + Into<f64>>(
    data: &ChunkedData<F>, times: &[Instant], current_display_time: u64,
) -> bool {
    let Some(&last_time) = times.last() else {
        return false;
    };
    let display_duration = Duration::from_millis(current_display_time);
    let oldest = last_time.checked_sub(display_duration).unwrap_or(last_time);
    data.iter_along_base(times)
        .next_back()
        .is_some_and(|(t, _)| *t >= oldest)
}

/// Format a byte/s rate as a fixed-width string (always 11 chars, right-aligned)
/// to keep legend labels a stable width and prevent legend box shifting between frames.
fn format_rate_fixed(bytes_per_sec: f64) -> String {
    let (val, unit) = if bytes_per_sec < KIBI_LIMIT_F64 {
        (bytes_per_sec, "B/s")
    } else if bytes_per_sec < MEBI_LIMIT_F64 {
        (bytes_per_sec / KIBI_LIMIT_F64, "KiB/s")
    } else if bytes_per_sec < GIBI_LIMIT_F64 {
        (bytes_per_sec / MEBI_LIMIT_F64, "MiB/s")
    } else if bytes_per_sec < TEBI_LIMIT_F64 {
        (bytes_per_sec / GIBI_LIMIT_F64, "GiB/s")
    } else {
        (bytes_per_sec / TEBI_LIMIT_F64, "TiB/s")
    };
    format!("{:>11}", format!("{val:.1}{unit}"))
}

fn adjust_disk_io_linear(max_entry: f64) -> (f64, Vec<String>) {
    let max_entry_upper = if max_entry == 0.0 {
        1.0
    } else {
        max_entry * 1.5
    };

    let (max_value_scaled, unit) = if max_entry_upper < KIBI_LIMIT_F64 {
        (max_entry, "B/s")
    } else if max_entry_upper < MEBI_LIMIT_F64 {
        (max_entry / KIBI_LIMIT_F64, "KiB/s")
    } else if max_entry_upper < GIBI_LIMIT_F64 {
        (max_entry / MEBI_LIMIT_F64, "MiB/s")
    } else if max_entry_upper < TEBI_LIMIT_F64 {
        (max_entry / GIBI_LIMIT_F64, "GiB/s")
    } else {
        (max_entry / TEBI_LIMIT_F64, "TiB/s")
    };

    let labels = vec![
        format!("0{unit}"),
        format!("{:.1}", max_value_scaled * 0.5),
        format!("{:.1}", max_value_scaled),
        format!("{:.1}", max_value_scaled * 1.5),
    ]
    .into_iter()
    .map(|s| format!("{s:>8}"))
    .collect();

    (max_entry_upper, labels)
}

fn adjust_disk_io_log(max_entry: f64) -> (f64, Vec<String>) {
    use crate::utils::general::saturating_log2;

    let log_max = saturating_log2(max_entry);

    if log_max < LOG_MEBI_LIMIT {
        (
            LOG_MEBI_LIMIT,
            vec!["  0B/s".into(), "1KiB/s".into(), "1MiB/s".into()],
        )
    } else if log_max < LOG_GIBI_LIMIT {
        (
            LOG_GIBI_LIMIT,
            vec![
                "  0B/s".into(),
                "1KiB/s".into(),
                "1MiB/s".into(),
                "1GiB/s".into(),
            ],
        )
    } else if log_max < LOG_TEBI_LIMIT {
        (
            LOG_TEBI_LIMIT,
            vec![
                "  0B/s".into(),
                "1KiB/s".into(),
                "1MiB/s".into(),
                "1GiB/s".into(),
                "1TiB/s".into(),
            ],
        )
    } else {
        (
            LOG_PEBI_LIMIT,
            vec![
                "  0B/s".into(),
                "1KiB/s".into(),
                "1MiB/s".into(),
                "1GiB/s".into(),
                "1TiB/s".into(),
                "1PiB/s".into(),
            ],
        )
    }
}
