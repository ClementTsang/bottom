// FIXME: Delete this!

use concat_string::concat_string;
use itertools::Itertools;
use tui::{layout::Rect, widgets::Row};

use crate::{
    components::data_table::{DataTable, DataTableColumn, DataTableInner, DataTableProps},
    utils::gen_util::truncate_text,
};

use super::sortable_column::{SortColumn, SortColumnInfo, SortsRow};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub struct SortableDataTableState<DataType, ColumnType>
where
    ColumnType: SortsRow<DataType>,
    T: DataTableInner<DataType>,
{
    /// The current column index we are sorting with.
    sort_index: usize,

    /// The current sorting order.
    order: SortOrder,

    /// Additional data used for sorting, click handling, and shortcuts.
    sort_columns: Vec<SortColumnInfo<DataType, ColumnType>>,

    /// The "y location" of the header row. Since all headers share the same
    /// y-location we just set it once here.
    y_loc: u16,

    /// The inner table representation.
    table: DataTable<DataType>,
}

impl<DataType, ColumnType, T> SortableDataTableState<DataType, ColumnType, T>
where
    ColumnType: SortsRow<DataType>,
    T: DataTableInner<DataType>,
{
    /// Creates a new [`SortableDataTableState`].
    pub fn new(
        default_sort_index: usize, columns: Vec<SortColumn<DataType, ColumnType>>,
        props: DataTableProps, inner: T,
    ) -> anyhow::Result<Self> {
        let order = columns
            .get(default_sort_index)
            .map(|col| col.info.default_order)
            .ok_or(anyhow::anyhow!(
                "Default sort index matches a column that does not exist."
            ))?;

        let mut sort_columns = Vec::with_capacity(columns.len());
        let mut dt_columns = Vec::with_capacity(columns.len());

        for column in columns {
            sort_columns.push(column.info);
            dt_columns.push(column.data_table_col);
        }

        Ok(Self {
            sort_index: default_sort_index,
            order,
            sort_columns,
            y_loc: 0,
            table: DataTable::new(dt_columns, props, inner),
        })
    }

    /// Sets the new draw locations for the table headers.
    ///
    /// **Note:** The function assumes the ranges will create a *sorted* list with the length
    /// equal to the number of columns - in debug mode, the program will assert all this, but
    /// it will **not** do so in release mode!
    pub fn update_header_draw_location(&mut self, draw_loc: Rect, row_widths: &[u16]) {
        let mut start = draw_loc.x;

        debug_assert_eq!(
            row_widths.len(),
            self.sort_columns.len(),
            "row width and sort column length should be equal"
        );

        row_widths
            .iter()
            .zip(self.sort_columns.iter_mut())
            .for_each(|(width, column)| {
                let range_start = start;
                let range_end = start + width + 1; // +1 for the gap between cols
                start = range_end;

                column.range = range_start..range_end;
            });

        debug_assert!(
            self.sort_columns
                .iter()
                .all(|a| { a.range.start <= a.range.end }),
            "all sort column ranges should have a start <= end"
        );

        debug_assert!(
            self.sort_columns
                .iter()
                .tuple_windows()
                .all(|(a, b)| { b.range.start >= a.range.end }),
            "sort column ranges should be sorted"
        );

        self.y_loc = draw_loc.y;
    }

    /// Given some `x` and `y`, if possible, select the corresponding column or toggle the column if already selected,
    /// and otherwise do nothing.
    ///
    /// If there was some update, the corresponding column type will be returned. If nothing happens, [`None`] is
    /// returned.
    pub fn try_select_location(&mut self, x: u16, y: u16) -> Option<usize> {
        if self.y_loc == y {
            if let Some(index) = self.get_range(x) {
                self.update_sort_index(index);
                Some(self.sort_index)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Updates the sort index, and sets the sort order as appropriate.
    ///
    /// If the index is different from the previous one, it will move to the new index and set the sort order
    /// to the prescribed default sort order.
    ///
    /// If the index is the same as the previous one, it will simply toggle the current sort order.
    pub fn update_sort_index(&mut self, index: usize) {
        if self.sort_index == index {
            self.toggle_order();
        } else if let Some(col) = self.sort_columns.get(index) {
            self.sort_index = index;
            self.order = col.default_order;
        }
    }

    /// Toggles the current sort order.
    pub fn toggle_order(&mut self) {
        self.order = match self.order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    /// Given a `needle` coordinate, select the corresponding index and value.
    fn get_range(&self, needle: u16) -> Option<usize> {
        match self
            .sort_columns
            .binary_search_by_key(&needle, |col| col.range.start)
        {
            Ok(index) => Some(index),
            Err(index) => index.checked_sub(1),
        }
        .and_then(|index| {
            if needle < self.sort_columns[index].range.end {
                Some(index)
            } else {
                None
            }
        })
    }
}

impl<DataType, ColumnType, T> DataTableInner<DataType>
    for SortableDataTableState<DataType, ColumnType, T>
where
    ColumnType: SortsRow<DataType>,
    T: DataTableInner<DataType>,
{
    fn to_data_row<'a>(&self, data: &'a DataType, columns: &[DataTableColumn]) -> Row<'a> {
        self.table.inner.to_data_row(data, columns)
    }

    fn column_widths(&self, data: &[DataType]) -> Vec<u16> {
        self.table.inner.column_widths(data)
    }

    fn build_header(&self, columns: &[DataTableColumn]) -> Row<'_> {
        const UP_ARROW: &str = "▲";
        const DOWN_ARROW: &str = "▼";

        let current_index = self.sort_index;
        let arrow = match self.order {
            SortOrder::Ascending => UP_ARROW,
            SortOrder::Descending => DOWN_ARROW,
        };

        Row::new(columns.iter().enumerate().filter_map(|(index, c)| {
            if c.calculated_width == 0 {
                None
            } else if index == current_index {
                Some(truncate_text(
                    concat_string!(c.header, arrow).into(),
                    c.calculated_width.into(),
                ))
            } else {
                Some(truncate_text(c.header.clone(), c.calculated_width.into()))
            }
        }))
    }
}
