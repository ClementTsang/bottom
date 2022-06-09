use tui::{backend::Backend, layout::Rect, terminal::Frame};

use crate::{
    app,
    canvas::Painter,
    components::data_table::{DrawInfo, SelectionState, Styling},
};

impl Painter {
    pub fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(temp_widget_state) = app_state.temp_state.widget_states.get_mut(&widget_id) {
            let is_on_widget = app_state.current_widget.widget_id == widget_id;

            // FIXME: This should be moved elsewhere.
            let styling = Styling {
                header_style: self.colours.table_header_style,
                border_style: self.colours.border_style,
                highlighted_border_style: self.colours.highlighted_border_style,
                text_style: self.colours.text_style,
                highlighted_text_style: self.colours.currently_selected_text_style,
                title_style: self.colours.widget_title_style,
            };
            let draw_info = DrawInfo {
                styling,
                loc: draw_loc,
                force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                selection_state: if app_state.is_expanded {
                    SelectionState::Expanded
                } else if is_on_widget {
                    SelectionState::Selected
                } else {
                    SelectionState::NotSelected
                },
            };

            temp_widget_state.table.draw(
                f,
                &draw_info,
                &app_state.converted_data.temp_data,
                app_state.widget_map.get_mut(&widget_id),
            );
        }
    }
}
