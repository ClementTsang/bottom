use tui::{
    Frame,
    layout::{Constraint, Rect},
};

use crate::{
    app::App,
    canvas::{
        Painter,
        components::time_series::{GraphData, LegendConstraints},
        drawing_utils::should_hide_x_label,
    },
    components::time_series::GraphDrawCtx,
};

impl Painter {
    pub fn draw_disk_space_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(widget_state) = app_state
            .states
            .disk_space_graph_state
            .get_mut_widget_state(widget_id)
        {
            let shared_data = app_state.data_store.get_data();
            let points = &shared_data.time_series_data.disk_used_percent;
            let times = &shared_data.time_series_data.time;

            let mount_map = super::disk_mount_map(&shared_data.disk_harvest);

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let graph_state = widget_state.graph.state_mut();
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                graph_state.autohide_timer_mut(),
                draw_loc,
            );
            let current_display_time = graph_state.current_display_time();

            let legend_type = &widget_state.legend;

            // Removed devices still visible in the window show "N/A".
            let mut entries: Vec<_> = points
                .iter()
                .filter(|(name, values)| {
                    mount_map.contains_key(name.as_str())
                        || super::has_data_in_window(values, times, current_display_time)
                })
                .collect();
            entries.sort_unstable_by_key(|(name, _)| *name);

            let legend_constraints = LegendConstraints {
                width: Constraint::Ratio(9, 10),
                height: Constraint::Ratio(1, 2),
            };

            let colours = &self.styles.disk_space_graph_colour_styles;

            let graph_data: Vec<GraphData<'_, f64>> = entries
                .iter()
                .enumerate()
                .map(|(idx, (name, values))| {
                    let display_name =
                        legend_type.display_name(name, mount_map.get(name.as_str()).copied());

                    let is_active = mount_map.contains_key(name.as_str());
                    let label = if is_active {
                        let pct = values.last().copied().unwrap_or(0.0);
                        format!("{display_name}: {pct:.1}%").into()
                    } else {
                        format!("{display_name}: N/A").into()
                    };

                    GraphData::default()
                        .name(label)
                        .style(super::cycle_style(colours, idx))
                        .time(times)
                        .values(values)
                })
                .collect();

            let marker = self.get_marker(app_state.app_config_fields.use_dot);

            widget_state.graph.draw(
                f,
                draw_loc,
                GraphDrawCtx {
                    title: " Disk Space ".into(),
                    border_style,
                    title_style: self.styles.widget_title_style,
                    graph_style: self.styles.graph_style,
                    general_widget_style: self.styles.general_widget_style,
                    border_type: self.styles.border_type,
                    marker,
                    hide_x_labels,
                    is_selected: app_state.current_widget.widget_id == widget_id,
                    is_expanded: app_state.is_expanded,
                    legend_position: app_state.app_config_fields.disk_space_legend_position,
                    legend_constraints: Some(legend_constraints),
                },
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
