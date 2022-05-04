use tui::{backend::Backend, layout::Rect, terminal::Frame};

use crate::{
    app,
    canvas::{
        components::{TextTable, TextTableTitle},
        Painter,
    },
};

impl Painter {
    pub fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(temp_widget_state) = app_state.temp_state.widget_states.get_mut(&widget_id) {
            let is_on_widget = app_state.current_widget.widget_id == widget_id;

            let (border_style, highlighted_text_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };
            TextTable {
                table_gap: app_state.app_config_fields.table_gap,
                is_force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                header_style: self.colours.table_header_style,
                border_style,
                highlighted_text_style,
                title: Some(TextTableTitle {
                    title: " Temperatures ".into(),
                    is_expanded: app_state.is_expanded,
                }),
                is_on_widget,
                draw_border,
                show_table_scroll_position: app_state.app_config_fields.show_table_scroll_position,
                title_style: self.colours.widget_title_style,
                text_style: self.colours.text_style,
                left_to_right: false,
            }
            .draw_text_table(
                f,
                draw_loc,
                &mut temp_widget_state.table_state,
                &app_state.canvas_data.temp_sensor_data,
            );

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
}
