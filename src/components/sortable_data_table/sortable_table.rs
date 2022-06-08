use crate::components::data_table::ToDataRow;

use super::{sortable_column::SortColumn, sorts_row::SortsRow};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub struct SortableDataTable<Row, ColumnType>
where
    Row: ToDataRow,
    ColumnType: SortsRow<Row>,
{
    /// The backing table itself.
    // pub table: DataTable<Row>,

    /// The current column index we are sorting with.
    current_sort_index: usize,

    /// The current sorting order.
    current_sort_direction: SortOrder,

    /// Additional data used for sorting, click handling, and shortcuts.
    sort_columns: Vec<SortColumn<Row, ColumnType>>,

    /// The "y location" of the header row. Since all headers share the same
    /// y-location we just set it once here.
    y_loc: u16,
}
