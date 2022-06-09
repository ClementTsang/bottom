use tui::widgets::Row;

use super::data_column::DataColumn;

pub trait ToDataRow {
    fn to_data_row<'a>(&'a self, columns: &[DataColumn]) -> Row<'a>;

    fn column_widths(data: &[Self]) -> Vec<u16>
    where
        Self: Sized;
}
