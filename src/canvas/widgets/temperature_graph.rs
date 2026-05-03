use tui::{
    Frame,
    layout::{Constraint, Rect},
    symbols::Marker,
};

use crate::{
    app::{App, AppConfigFields, data::TemperatureType},
    canvas::{
        Painter,
        components::time_graph::{
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
            .widget_states
            .get_mut(&widget_id)
        {
            let shared_data = app_state.data_store.get_data();
            let points = &(shared_data.timeseries_data.temperature);
            let times = &(shared_data.timeseries_data.time);
            let time_start = -(widget_state.current_display_time as f64);

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut widget_state.autohide_timer,
                draw_loc,
            );

            let y_max = {
                if let Some(last_time) = times.last() {
                    let cache = &mut widget_state.height_cache;
                    cache.get_or_update(
                        last_time,
                        widget_state.current_display_time,
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

            let graph_data: Vec<GraphData<'_, f32>> = points
                .iter()
                .enumerate()
                .map(|(itx, (source, values))| {
                    GraphData::default()
                        .name(source.into())
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

    let unit = match config.temperature_type {
        TemperatureType::Celsius => "°C",
        TemperatureType::Kelvin => "K",
        TemperatureType::Fahrenheit => "°F",
    };

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
