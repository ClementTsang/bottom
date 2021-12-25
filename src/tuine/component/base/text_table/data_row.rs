use tui::style::Style;

use super::DataCell;

pub struct DataRow {
    cells: Vec<DataCell>,
    style: Option<Style>,
}

impl DataRow {
    pub fn style(mut self, style: Option<Style>) -> Self {
        self.style = style;
        self
    }
}
