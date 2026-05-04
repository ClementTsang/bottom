use tui::{
    Frame,
    layout::{Constraint, Rect},
    symbols::Marker,
};

use crate::{
    app::{App, AppConfigFields},
    canvas::{
        Painter,
        components::timeseries::{
            AxisBound, ChartScaling, GraphData, LegendConstraints, TimeGraph,
        },
        drawing_utils::should_hide_x_label,
    },
};

impl Painter {
    pub fn draw_temperature_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(widget_state) = app_state
            .states
            .temp_graph_state
            .get_mut_widget_state(widget_id)
        {
            let shared_data = app_state.data_store.get_data();
            let points = &(shared_data.timeseries_data.temperature);
            let times = &(shared_data.timeseries_data.time);
            let time_start = -(widget_state.timeseries_state.current_display_time() as f64);

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                widget_state.timeseries_state.get_autohide_timer_mut(),
                draw_loc,
            );

            let y_max = {
                if let Some(last_time) = times.last() {
                    let cache = &mut widget_state.height_cache;
                    cache.get_or_update(
                        last_time,
                        widget_state.timeseries_state.current_display_time(),
                        points.values(),
                        times,
                    )
                } else {
                    0.0
                }
            };
            let (adjusted_y_max, y_labels) =
                adjust_temp_data_point(y_max, widget_state.max_temp, &app_state.app_config_fields);
            let y_bounds = AxisBound::Max(adjusted_y_max);

            // Hide the legend if the width is 90% of the total widget width
            // or the height is greater than 50% of the total widget height.
            let legend_constraints = LegendConstraints {
                width: Constraint::Ratio(9, 10),
                height: Constraint::Ratio(1, 2),
            };

            let unit = app_state.app_config_fields.temperature_type.unit();
            let graph_data: Vec<GraphData<'_, f32>> = points
                .iter()
                .enumerate()
                .map(|(itx, (source, values))| {
                    // TODO: Maybe align the value later.
                    let name = match values.last() {
                        Some(latest) => format!("{source}: {latest:.0}{unit}").into(),
                        None => source.as_str().into(),
                    };
                    GraphData::default()
                        .name(name)
                        .style(
                            self.styles.temp_graph_colour_styles
                                [itx % self.styles.temp_graph_colour_styles.len()],
                        )
                        .time(times)
                        .values(values)
                })
                .collect();

            let marker = if app_state.app_config_fields.use_dot {
                Marker::Dot
            } else {
                Marker::Braille
            };

            TimeGraph {
                x_min: time_start,
                hide_x_labels,
                y_bounds,
                y_labels: &(y_labels.into_iter().map(Into::into).collect::<Vec<_>>()),
                graph_style: self.styles.graph_style,
                general_widget_style: self.styles.general_widget_style,
                border_style,
                border_type: self.styles.border_type,
                title: " Temperature ".into(),
                is_selected: app_state.current_widget.widget_id == widget_id,
                is_expanded: app_state.is_expanded,
                title_style: self.styles.widget_title_style,
                legend_position: app_state.app_config_fields.temperature_legend_position,
                legend_constraints: Some(legend_constraints),
                marker,
                scaling: ChartScaling::Linear,
            }
            .draw(f, draw_loc, graph_data);
        }

        // Update draw loc in widget map.
        if app_state.should_get_widget_bounds() {
            if let Some(temperature_graph_widget) = app_state.widget_map.get_mut(&widget_id) {
                temperature_graph_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                temperature_graph_widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}

/// Returns the required labels.
fn adjust_temp_data_point(
    max_entry: f64, upper_limit: Option<f32>, config: &AppConfigFields,
) -> (f64, [String; 3]) {
    let default_upper: f64 = config
        .temperature_type
        .convert_temp_unit_float(100.0)
        .into();
    let unit = config.temperature_type.unit();

    let max_entry = if let Some(limit) = upper_limit {
        limit as f64
    } else if max_entry < default_upper {
        default_upper
    } else {
        max_entry
    };

    let halfway_label = (max_entry / 2.0).ceil() as u32;
    let max_entry_label = max_entry.ceil() as u32;

    let labels = [
        format!("0{unit}"),
        format!("{halfway_label}{unit}"),
        format!("{max_entry_label}{unit}"),
    ];

    (max_entry, labels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::data::TemperatureType;

    fn config(temperature_type: TemperatureType) -> AppConfigFields {
        AppConfigFields {
            temperature_type,
            ..Default::default()
        }
    }

    #[test]
    fn floors_to_default_upper_when_below() {
        let cfg = config(TemperatureType::Celsius);
        let (max, labels) = adjust_temp_data_point(40.0, None, &cfg);
        assert_eq!(max, 100.0);
        assert_eq!(labels, ["0°C", "50°C", "100°C"]);
    }

    #[test]
    fn keeps_actual_max_when_above_default() {
        let cfg = config(TemperatureType::Celsius);
        let (max, labels) = adjust_temp_data_point(120.0, None, &cfg);
        assert_eq!(max, 120.0);
        assert_eq!(labels, ["0°C", "60°C", "120°C"]);
    }

    #[test]
    fn upper_limit_overrides_actual_max() {
        let cfg = config(TemperatureType::Celsius);
        let (max, labels) = adjust_temp_data_point(150.0, Some(80.0), &cfg);
        assert_eq!(max, 80.0);
        assert_eq!(labels, ["0°C", "40°C", "80°C"]);
    }

    #[test]
    fn fahrenheit_default_is_212() {
        let cfg = config(TemperatureType::Fahrenheit);
        let (max, labels) = adjust_temp_data_point(100.0, None, &cfg);
        assert_eq!(max, 212.0);
        assert_eq!(labels, ["0°F", "106°F", "212°F"]);
    }

    #[test]
    fn kelvin_default_is_373() {
        let cfg = config(TemperatureType::Kelvin);
        let (max, labels) = adjust_temp_data_point(100.0, None, &cfg);
        assert!((max - 373.15).abs() < 1e-3);
        assert_eq!(labels, ["0K", "187K", "374K"]);
    }
}
