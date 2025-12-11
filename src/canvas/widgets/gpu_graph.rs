use tui::{Frame, layout::Rect};

use crate::{
    app::{App, GraphStyle},
    canvas::Painter,
    canvas::components::time_graph::GraphData,
    canvas::components::time_graph::variants::percent::PercentTimeGraph,
};

impl Painter {
    pub fn draw_gpu_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(gpu_widget_state) = app_state.states.gpu_state.widget_states.get(&widget_id) {
            let data = app_state.data_store.get_data();
            let time = &data.timeseries_data.time;

            // Collect datasets
            let mut graph_data: Vec<GraphData<'_>> = Vec::new();

            let filled = matches!(app_state.app_config_fields.graph_style, GraphStyle::Filled);

            // Map each GPU to a dataset (sorted by name for consistent colors)
            let mut gpu_names: Vec<_> = data.timeseries_data.agnostic_gpu.keys().collect();
            gpu_names.sort();

            for (i, name) in gpu_names.iter().enumerate() {
                if let Some(values) = data.timeseries_data.agnostic_gpu.get(*name) {
                    let style =
                        self.styles.cpu_colour_styles[i % self.styles.cpu_colour_styles.len()];

                    graph_data.push(
                        GraphData::default()
                            .style(style)
                            .time(time)
                            .values(values)
                            .filled(filled),
                    );
                }
            }
            // Create title with GPU info
            let title = if let Some(gpu_data) = data.agnostic_gpu_harvest.first() {
                let mem_used_mb = gpu_data.memory_used / 1024 / 1024;
                let mem_total_mb = gpu_data.memory_total / 1024 / 1024;
                format!(
                    " GPU: {} [{:.0}%] {}MB/{}MB ",
                    gpu_data.name, gpu_data.load_percent, mem_used_mb, mem_total_mb
                )
                .into()
            } else {
                " GPU ".into()
            };

            PercentTimeGraph {
                display_range: gpu_widget_state.current_display_time,
                hide_x_labels: false, // For now
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
}
