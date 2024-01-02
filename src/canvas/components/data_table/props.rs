use std::borrow::Cow;

pub struct DataTableProps {
    /// An optional title for the table.
    pub title: Option<Cow<'static, str>>,

    /// The size of the gap between the header and rows.
    pub table_gap: u16,

    /// Whether this table determines column widths from left to right.
    pub left_to_right: bool,

    /// Whether this table is a basic table. This affects the borders.
    pub is_basic: bool,

    /// Whether to show the table scroll position.
    pub show_table_scroll_position: bool,

    /// Whether to show the current entry as highlighted when not focused.
    pub show_current_entry_when_unfocused: bool,
}
