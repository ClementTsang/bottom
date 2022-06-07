use std::{marker::PhantomData, ops::Range};

use crate::components::data_table::ToDataRow;

use super::{sorts_row::SortsRow, Shortcut};

pub struct SortColumn<Row, ColumnType>
where
    Row: ToDataRow,
    ColumnType: SortsRow<Row>,
{
    /// The "x locations" of the column.
    pub range: Range<u16>,

    /// Any shortcuts, if available.
    pub shortcut: Option<Shortcut>,

    /// The actual type of the column.
    pub column_type: ColumnType,

    /// Necessary due to Rust's limitations.
    _pd: PhantomData<Row>,
}
