use std::{borrow::Cow, num::NonZeroU16};

use tui::widgets::Row;

use super::{ColumnHeader, DataTableColumn};
use crate::canvas::Painter;

/// Something that represents a table displaying data.
pub trait DataTable<DataType> {
    /// The type of column header that this table uses.
    type HeaderType: ColumnHeader;

    /// Given data, a column, and its corresponding width, return the string in
    /// the cell that will be displayed in the [`super::DataTable`].
    fn to_cell_text(
        &self, data: &DataType, column: &Self::HeaderType, calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>>;

    /// Given a column, how to style a cell if one needs to override the default styling.
    ///
    /// By default this just returns [`None`], deferring to the row or table styling.
    #[expect(
        unused_variables,
        reason = "The default implementation just returns `None`."
    )]
    fn style_cell(
        &self, data: &DataType, column: &Self::HeaderType, painter: &Painter,
    ) -> Option<tui::style::Style> {
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
    fn style_row<'a>(&self, data: &DataType, row: Row<'a>, painter: &Painter) -> Row<'a> {
        row
    }

    /// Returns the desired column widths in light of having seen data.
    fn column_widths<C: DataTableColumn<Self::HeaderType>>(
        &self, data: &[DataType], columns: &[C],
    ) -> Vec<u16>
    where
        Self: Sized;
}

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
