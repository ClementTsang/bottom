use std::{borrow::Cow, num::NonZeroU16};

use tui::widgets::Row;

use super::{ColumnHeader, DataTableColumn};
use crate::canvas::Painter;

pub trait DataToCell<H>
where
    H: ColumnHeader,
{
    /// Given data, a column, and its corresponding width, return the string in the cell that will
    /// be displayed in the [`DataTable`](super::DataTable).
    fn to_cell(&self, column: &H, calculated_width: NonZeroU16) -> Option<Cow<'static, str>>;

    /// Apply styling to the generated [`Row`] of cells.
    ///
    /// The default implementation just returns the `row` that is passed in.
    #[inline(always)]
    fn style_row<'a>(&self, row: Row<'a>, _painter: &Painter) -> Row<'a> {
        row
    }

    /// Returns the desired column widths in light of having seen data.
    fn column_widths<C: DataTableColumn<H>>(data: &[Self], columns: &[C]) -> Vec<u16>
    where
        Self: Sized;
}
