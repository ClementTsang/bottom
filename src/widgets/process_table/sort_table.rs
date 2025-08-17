use std::{borrow::Cow, num::NonZeroU16};

use crate::canvas::components::data_table::{ColumnHeader, DataTableColumn, DataToCell};

pub struct SortTableColumn;

impl ColumnHeader for SortTableColumn {
    fn text(&self) -> Cow<'static, str> {
        "Sort By".into()
    }
}

impl DataToCell<SortTableColumn> for &'static str {
    fn to_cell_text(
        &self, _column: &SortTableColumn, _calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(self))
    }

    fn column_widths<C: DataTableColumn<SortTableColumn>>(data: &[Self], _columns: &[C]) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![data.iter().map(|d| d.len() as u16).max().unwrap_or(0)]
    }
}

impl DataToCell<SortTableColumn> for Cow<'static, str> {
    fn to_cell_text(
        &self, _column: &SortTableColumn, _calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        Some(self.clone())
    }

    fn column_widths<C: DataTableColumn<SortTableColumn>>(data: &[Self], _columns: &[C]) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![data.iter().map(|d| d.len() as u16).max().unwrap_or(0)]
    }
}
