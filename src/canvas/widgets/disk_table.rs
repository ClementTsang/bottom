use tui::{layout::Rect, Frame};

use crate::{
    app,
    canvas::{
        components::data_table::{DrawInfo, SelectionState},
        Painter,
    },
};

impl Painter {
    pub fn draw_disk_table(
        &self, f: &mut Frame<'_>, app_state: &mut app::App, draw_loc: Rect, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(disk_widget_state) = app_state
            .states
            .disk_state
            .widget_states
            .get_mut(&widget_id)
        {
            let is_on_widget = app_state.current_widget.widget_id == widget_id;

            let draw_info = DrawInfo {
                loc: draw_loc,
                force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                selection_state: SelectionState::new(app_state.is_expanded, is_on_widget),
            };

            disk_widget_state.table.draw(
                f,
                &draw_info,
                app_state.widget_map.get_mut(&widget_id),
                self,
            );
        }
    }
}
