use tui::style::Style;

use crate::options::config::style::Styles;

pub(crate) mod auto_y_axis;
pub(crate) mod percent;

fn get_border_style(styles: &Styles, widget_id: u64, selected_widget_id: u64) -> Style {
    let is_on_widget = widget_id == selected_widget_id;
    if is_on_widget {
        styles.highlighted_border_style
    } else {
        styles.border_style
    }
}
