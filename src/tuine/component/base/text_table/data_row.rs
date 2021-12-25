use tui::{style::Style, widgets::Row};

use super::DataCell;

#[derive(Clone)]
pub struct DataRow {
    pub cells: Vec<DataCell>,
    pub style: Option<Style>,
}

impl DataRow {
    pub fn new(cells: Vec<DataCell>) -> Self {
        Self { cells, style: None }
    }

    pub fn style(mut self, style: Option<Style>) -> Self {
        self.style = style;
        self
    }
}

impl From<DataRow> for Row<'_> {
    fn from(row: DataRow) -> Self {
        if let Some(style) = row.style {
            Row::new(row.cells).style(style)
        } else {
            Row::new(row.cells)
        }
    }
}
