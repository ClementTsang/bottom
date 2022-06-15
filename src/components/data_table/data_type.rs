use tui::widgets::Row;

pub trait ToDataRow {
    /// Builds a [`Row`] given data.
    fn to_data_row<'a>(&self, widths: &[u16]) -> Row<'a>;

    /// Returns the desired column widths in light of having seen data.
    fn column_widths(data: &[Self]) -> Vec<u16>
    where
        Self: Sized;
}
