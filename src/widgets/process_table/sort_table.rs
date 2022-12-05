use std::borrow::Cow;

use tui::text::Text;

use crate::{
    components::data_table::{ColumnHeader, DataTableColumn, DataToCell},
    utils::gen_util::truncate_to_text,
};

pub struct SortTableColumn;

impl ColumnHeader for SortTableColumn {
    fn text(&self) -> std::borrow::Cow<'static, str> {
        "Sort By".into()
    }
}

impl DataToCell<SortTableColumn> for &'static str {
    fn to_cell<'a>(&'a self, _column: &SortTableColumn, calculated_width: u16) -> Option<Text<'a>> {
        Some(truncate_to_text(self, calculated_width))
    }

    fn column_widths<C: DataTableColumn<SortTableColumn>>(data: &[Self], _columns: &[C]) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![data.iter().map(|d| d.len() as u16).max().unwrap_or(0)]
    }
}

impl DataToCell<SortTableColumn> for Cow<'static, str> {
    fn to_cell<'a>(&'a self, _column: &SortTableColumn, calculated_width: u16) -> Option<Text<'a>> {
        Some(truncate_to_text(self, calculated_width))
    }

    fn column_widths<C: DataTableColumn<SortTableColumn>>(data: &[Self], _columns: &[C]) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![data.iter().map(|d| d.len() as u16).max().unwrap_or(0)]
    }
}
