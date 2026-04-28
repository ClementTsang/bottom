use tui::{Frame, layout::Rect};

use crate::{app::App, canvas::Painter};

impl Painter {
    pub fn draw_temperature_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(widget_state) = app_state
            .states
            .temp_graph_state
            .widget_states
            .get_mut(&widget_id)
        {}

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
