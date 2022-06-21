use std::borrow::Cow;

use tui::widgets::Row;

use crate::utils::gen_util::truncate_text;

pub trait ToDataRow {
    /// Builds a [`Row`] given data.
    fn to_data_row<'a>(&self, widths: &[u16]) -> Row<'a>;

    /// Returns the desired column widths in light of having seen data.
    fn column_widths(data: &[Self]) -> Vec<u16>
    where
        Self: Sized;
}

impl ToDataRow for Cow<'static, str> {
    fn to_data_row<'a>(&self, widths: &[u16]) -> Row<'a> {
        Row::new(vec![truncate_text(self.clone(), widths[0].into())])
    }

    fn column_widths(_data: &[Self]) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![]
    }
}
