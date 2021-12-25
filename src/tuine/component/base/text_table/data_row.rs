use tui::{style::Style, widgets::Row};

use super::DataCell;

#[derive(Default, Clone)]
pub struct DataRow {
    cells: Vec<DataCell>,
    style: Option<Style>,
}

impl DataRow {
    pub fn new_with_vec<D: Into<DataCell>>(cells: Vec<D>) -> Self {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            style: None,
        }
    }

    pub fn cell<D: Into<DataCell>>(mut self, cell: D) -> Self {
        self.cells.push(cell.into());
        self
    }

    pub fn style(mut self, style: Option<Style>) -> Self {
        self.style = style;
        self
    }

    pub fn get(&self, index: usize) -> Option<&DataCell> {
        self.cells.get(index)
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
