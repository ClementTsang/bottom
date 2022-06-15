use std::{marker::PhantomData, ops::Range};

use crossterm::event::KeyCode;

use crate::components::data_table::DataTableColumn;

use super::SortOrder;

pub struct SortColumn<RowData, ColumnType>
where
    ColumnType: SortsRow<RowData>,
{
    pub info: SortColumnInfo<RowData, ColumnType>,
    pub data_table_col: DataTableColumn,
}

pub trait SortsRow<DataType> {
    /// Sorts data.
    fn sort_data(&self, data: &mut [DataType], ascending: bool);
}

pub struct SortColumnInfo<DataType, ColumnType>
where
    ColumnType: SortsRow<DataType>,
{
    /// The "x locations" of the column.
    pub range: Range<u16>,

    /// A shortcut, if set.
    pub shortcut: Option<KeyCode>,

    /// The actual type of the column.
    pub column_type: ColumnType,

    /// The default sort ordering.
    pub default_order: SortOrder,

    /// Necessary due to Rust's limitations.
    _pd: PhantomData<DataType>,
}

impl<DataType, ColumnType> SortColumnInfo<DataType, ColumnType>
where
    ColumnType: SortsRow<DataType>,
{
    pub fn new(
        shortcut: Option<KeyCode>, column_type: ColumnType, default_order: SortOrder,
    ) -> Self {
        Self {
            range: Range::default(),
            shortcut,
            column_type,
            default_order,
            _pd: PhantomData,
        }
    }
}
