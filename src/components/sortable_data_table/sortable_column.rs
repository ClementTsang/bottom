use std::{marker::PhantomData, ops::Range};

use super::Shortcut;

pub struct SortColumn<Row, ColumnType> {
    /// The "x locations" of the column.
    pub range: Range<u16>,

    /// Any shortcuts, if available.
    pub shortcut: Option<Shortcut>,

    /// The actual type of the column.
    pub column_type: ColumnType,

    /// Necessary due to Rust's limitations.
    _pd: PhantomData<Row>,
}
