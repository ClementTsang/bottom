use std::borrow::Cow;

use crate::{
    app::App,
    canvas::{
        components::{GraphData, TimeGraph},
        drawing_utils::should_hide_x_label,
        Painter,
    },
};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
};

impl Painter {
    pub fn draw_memory_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        const Y_BOUNDS: [f64; 2] = [0.0, 100.5];
        const Y_LABELS: [Cow<'static, str>; 2] = [Cow::Borrowed("  0%"), Cow::Borrowed("100%")];

        if let Some(mem_widget_state) = app_state.mem_state.widget_states.get_mut(&widget_id) {
            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let x_bounds = [0, mem_widget_state.current_display_time];
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut mem_widget_state.autohide_timer,
                draw_loc,
            );
            let points = {
                let mut points = Vec::with_capacity(2);
                if let Some((label_percent, label_frac)) = &app_state.converted_data.mem_labels {
                    let mem_label = format!("RAM:{}{}", label_percent, label_frac);
                    points.push(GraphData {
                        points: &app_state.converted_data.mem_data,
                        style: self.colours.ram_style,
                        name: Some(mem_label.into()),
                    });
                }
                if let Some((label_percent, label_frac)) = &app_state.converted_data.swap_labels {
                    let swap_label = format!("SWP:{}{}", label_percent, label_frac);
                    points.push(GraphData {
                        points: &app_state.converted_data.swap_data,
                        style: self.colours.swap_style,
                        name: Some(swap_label.into()),
                    });
                }

                points
            };

            TimeGraph {
                use_dot: app_state.app_config_fields.use_dot,
                x_bounds,
                hide_x_labels,
                y_bounds: Y_BOUNDS,
                y_labels: &Y_LABELS,
                graph_style: self.colours.graph_style,
                border_style,
                title: " Memory ".into(),
                is_expanded: app_state.is_expanded,
                title_style: self.colours.widget_title_style,
                legend_constraints: Some((Constraint::Ratio(3, 4), Constraint::Ratio(3, 4))),
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
