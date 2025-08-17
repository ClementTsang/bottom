use std::{borrow::Cow, num::NonZeroU16};

use tui::widgets::Row;

use super::{ColumnHeader, DataTableColumn};
use crate::canvas::Painter;

pub trait DataToCell<H>
where
    H: ColumnHeader,
{
    /// Given data, a column, and its corresponding width, return the string in
    /// the cell that will be displayed in the [`super::DataTable`].
    fn to_cell_text(&self, column: &H, calculated_width: NonZeroU16) -> Option<Cow<'static, str>>;

    /// Given a column, how to style a cell if one needs to override the default styling.
    ///
    /// By default this just returns [`None`], deferring to the row or table styling.
    #[expect(
        unused_variables,
        reason = "The default implementation just returns `None`."
    )]
    fn style_cell(&self, column: &H, painter: &Painter) -> Option<tui::style::Style> {
        None
    }

    /// Apply styling to the generated [`Row`] of cells.
    ///
    /// The default implementation just returns the `row` that is passed in.
    #[inline(always)]
    #[expect(
        unused_variables,
        reason = "The default implementation just returns an unstyled row."
    )]
    fn style_row<'a>(&self, row: Row<'a>, painter: &Painter) -> Row<'a> {
        row
    }

    /// Returns the desired column widths in light of having seen data.
    fn column_widths<C: DataTableColumn<H>>(data: &[Self], columns: &[C]) -> Vec<u16>
    where
        Self: Sized;
}
