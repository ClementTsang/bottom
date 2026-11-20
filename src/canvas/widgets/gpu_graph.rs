use ratatui::{Frame, layout::Rect, style::Style};

use crate::{
    app::App,
    canvas::{Painter, components::time_series::GraphData, drawing_utils::should_hide_x_label},
    components::time_series::GraphDrawCtx,
};

impl Painter {
    /// Draw a GPU utilization graph.
    pub fn draw_gpu_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(gpu_state) = app_state
            .states
            .gpu_graph_state
            .widget_states
            .get_mut(&widget_id)
        {
            let data = app_state.data_store.get_data();
            let time = &data.time_series_data.time;

            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                gpu_state.graph.state_mut().autohide_timer_mut(),
                draw_loc,
            );

            let gpu_styles = &self.styles.gpu_colours;
            let short_gpu_names = app_state.app_config_fields.gpu_usage_short_names;
            let gpu_count = data.gpu_utilization.len();

            let mut graph_data: Vec<GraphData<'_>> = Vec::with_capacity(gpu_count);

            for (itx, (name, current_util)) in data.gpu_utilization.iter().enumerate() {
                let Some(values) = data.time_series_data.gpu_usage.get(name) else {
                    continue;
                };

                let style = if gpu_styles.is_empty() {
                    Style::default()
                } else {
                    gpu_styles[itx % gpu_styles.len()]
                };

                let display_name = if short_gpu_names {
                    if gpu_count == 1 {
                        String::from("GPU")
                    } else {
                        format!("GPU{itx}")
                    }
                } else {
                    name.to_string()
                };

                let label = format!("{display_name}: {current_util:.0}%");

                graph_data.push(
                    GraphData::default()
                        .name(label.into())
                        .time(time)
                        .values(values)
                        .style(style),
                );
            }

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let marker = self.get_marker(app_state.app_config_fields.use_dot);

            gpu_state.graph.draw(
                f,
                draw_loc,
                GraphDrawCtx {
                    title: " GPU ".into(),
                    border_style,
                    title_style: self.styles.widget_title_style,
                    graph_style: self.styles.graph_style,
                    general_widget_style: self.styles.general_widget_style,
                    border_type: self.styles.border_type,
                    marker,
                    hide_x_labels,
                    is_selected: app_state.current_widget.widget_id == widget_id,
                    is_expanded: app_state.is_expanded,
                    legend_position: app_state.app_config_fields.gpu_usage_legend_position,
                    legend_constraints: None,
                },
                graph_data,
            );
        }

        if app_state.should_get_widget_bounds()
            && let Some(widget) = app_state.widget_map.get_mut(&widget_id)
        {
            widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
            widget.bottom_right_corner =
                Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
        }
    }
}
