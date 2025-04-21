use tui::{style::Style, widgets::BorderType};

use crate::options::config::style::Styles;

#[derive(Default)]
pub struct DataTableStyling {
    pub header_style: Style,
    pub border_style: Style,
    pub border_type: BorderType,
    pub highlighted_border_style: Style,
    pub text_style: Style,
    pub highlighted_text_style: Style,
    pub title_style: Style,
}

impl DataTableStyling {
    pub fn from_palette(styles: &Styles) -> Self {
        Self {
            header_style: styles.table_header_style,
            border_style: styles.border_style,
            border_type: styles.border_type,
            highlighted_border_style: styles.highlighted_border_style,
            text_style: styles.text_style,
            highlighted_text_style: styles.selected_text_style,
            title_style: styles.widget_title_style,
        }
    }
}
