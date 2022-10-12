use std::{borrow::Cow, marker::PhantomData};

use concat_string::concat_string;
use itertools::Itertools;
use tui::widgets::Row;

use crate::utils::gen_util::truncate_text;

use super::{
    ColumnHeader, ColumnWidthBounds, DataTable, DataTableColumn, DataTableProps, DataTableState,
    DataTableStyling, DataToCell,
};

/// Denotes the sort order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

/// Denotes the [`DataTable`] is unsorted.
pub struct Unsortable;

/// Denotes the [`DataTable`] is sorted.
pub struct Sortable {
    /// The currently selected sort index.
    pub sort_index: usize,

    /// The current sorting order.
    pub order: SortOrder,
}

/// The [`SortType`] trait is meant to be used in the typing of a [`DataTable`]
/// to denote whether the table is meant to display/store sorted or unsorted data.
///
/// Note that the trait is [sealed](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed),
/// and therefore only [`Unsortable`] and [`Sortable`] can implement it.
pub trait SortType: private::Sealed {
    /// Constructs the table header.
    fn build_header<H, C>(&self, columns: &[C], widths: &[u16]) -> Row<'_>
    where
        H: ColumnHeader,
        C: DataTableColumn<H>,
    {
        Row::new(columns.iter().zip(widths).filter_map(|(c, &width)| {
            if width == 0 {
                None
            } else {
                Some(truncate_text(&c.header(), width))
            }
        }))
    }
}

mod private {
    use super::{Sortable, Unsortable};

    pub trait Sealed {}

    impl Sealed for Unsortable {}
    impl Sealed for Sortable {}
}

impl SortType for Unsortable {}

impl SortType for Sortable {
    fn build_header<H, C>(&self, columns: &[C], widths: &[u16]) -> Row<'_>
    where
        H: ColumnHeader,
        C: DataTableColumn<H>,
    {
        const UP_ARROW: &str = "▲";
        const DOWN_ARROW: &str = "▼";

        Row::new(
            columns
                .iter()
                .zip(widths)
                .enumerate()
                .filter_map(|(index, (c, &width))| {
                    if width == 0 {
                        None
                    } else if index == self.sort_index {
                        let arrow = match self.order {
                            SortOrder::Ascending => UP_ARROW,
                            SortOrder::Descending => DOWN_ARROW,
                        };
                        Some(truncate_text(&concat_string!(c.header(), arrow), width))
                    } else {
                        Some(truncate_text(&c.header(), width))
                    }
                }),
        )
    }
}

pub trait SortsRow<DataType> {
    /// Sorts data.
    fn sort_data(&self, data: &mut [DataType], descending: bool);
}

#[derive(Debug, Clone)]
pub struct SortColumn<DataType, T> {
    /// The inner column header.
    inner: T,

    /// The default sort order.
    pub default_order: SortOrder,

    /// A restriction on this column's width.
    pub bounds: ColumnWidthBounds,

    /// Marks that this column is currently "hidden", and should *always* be skipped.
    pub is_hidden: bool,

    _pd: PhantomData<DataType>,
}

impl<DataType, T> DataTableColumn<T> for SortColumn<DataType, T>
where
    T: ColumnHeader + SortsRow<DataType>,
{
    #[inline]
    fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    #[inline]
    fn bounds(&self) -> ColumnWidthBounds {
        self.bounds
    }

    #[inline]
    fn bounds_mut(&mut self) -> &mut ColumnWidthBounds {
        &mut self.bounds
    }

    #[inline]
    fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    #[inline]
    fn set_is_hidden(&mut self, is_hidden: bool) {
        self.is_hidden = is_hidden;
    }

    fn header(&self) -> Cow<'static, str> {
        self.inner.header()
    }

    fn header_len(&self) -> usize {
        self.header().len() + 1
    }
}

impl<DataType, T> SortColumn<DataType, T>
where
    T: ColumnHeader + SortsRow<DataType>,
{
    /// Creates a new [`SortColumn`] with a width that follows the header width, which has no shortcut and sorts by
    /// default in ascending order ([`SortOrder::Ascending`]).
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::FollowHeader,
            is_hidden: false,
            default_order: SortOrder::default(),
            _pd: Default::default(),
        }
    }

    /// Creates a new [`SortColumn`] with a hard width, which has no shortcut and sorts by default in
    /// ascending order ([`SortOrder::Ascending`]).
    pub fn hard(inner: T, width: u16) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Hard(width),
            is_hidden: false,
            default_order: SortOrder::default(),
            _pd: Default::default(),
        }
    }

    /// Creates a new [`SortColumn`] with a soft width, which has no shortcut and sorts by default in
    /// ascending order ([`SortOrder::Ascending`]).
    pub fn soft(inner: T, max_percentage: Option<f32>) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Soft {
                desired: 0,
                max_percentage,
            },
            is_hidden: false,
            default_order: SortOrder::default(),
            _pd: Default::default(),
        }
    }

    /// Sets the default sort order to [`SortOrder::Ascending`].
    pub fn default_ascending(mut self) -> Self {
        self.default_order = SortOrder::Ascending;
        self
    }

    /// Sets the default sort order to [`SortOrder::Descending`].
    pub fn default_descending(mut self) -> Self {
        self.default_order = SortOrder::Descending;
        self
    }

    /// Given a [`SortColumn`] and the sort order, sort a mutable slice of associated data.
    pub fn sort_by(&self, data: &mut [DataType], order: SortOrder) {
        let descending = matches!(order, SortOrder::Descending);
        self.inner.sort_data(data, descending);
    }
}

pub struct SortDataTableProps {
    pub inner: DataTableProps,
    pub sort_index: usize,
    pub order: SortOrder,
}

/// A type alias for a sortable [`DataTable`].
pub type SortDataTable<DataType, H> = DataTable<DataType, H, Sortable, SortColumn<DataType, H>>;

impl<DataType, H> SortDataTable<DataType, H>
where
    DataType: DataToCell<H>,
    H: ColumnHeader + SortsRow<DataType>,
{
    pub fn new_sortable<C: Into<Vec<SortColumn<DataType, H>>>>(
        columns: C, props: SortDataTableProps, styling: DataTableStyling,
    ) -> Self {
        Self {
            columns: columns.into(),
            state: DataTableState::default(),
            props: props.inner,
            styling,
            sort_type: Sortable {
                sort_index: props.sort_index,
                order: props.order,
            },
            first_draw: true,
            data: vec![],
            _pd: PhantomData,
        }
    }

    /// Sets the current sort order.
    pub fn set_order(&mut self, order: SortOrder) {
        self.sort_type.order = order;
    }

    /// Gets the current sort order.
    pub fn order(&self) -> SortOrder {
        self.sort_type.order
    }

    /// Toggles the current sort order.
    pub fn toggle_order(&mut self) {
        self.sort_type.order = match self.sort_type.order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    /// Given some `x` and `y`, if possible, select the corresponding column or toggle the column if already selected,
    /// and otherwise do nothing.
    ///
    /// If there was some update, the corresponding column type will be returned. If nothing happens, [`None`] is
    /// returned.
    pub fn try_select_location(&mut self, x: u16, y: u16) -> Option<usize> {
        if self.state.inner_rect.height > 1 && self.state.inner_rect.y == y {
            if let Some(index) = self.get_range(x) {
                self.set_sort_index(index);
                Some(self.sort_type.sort_index)
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
    pub fn set_sort_index(&mut self, index: usize) {
        if self.sort_type.sort_index == index {
            self.toggle_order();
        } else if let Some(col) = self.columns.get(index) {
            self.sort_type.sort_index = index;
            self.sort_type.order = col.default_order;
        }
    }

    /// Returns the current sort index.
    pub fn sort_index(&self) -> usize {
        self.sort_type.sort_index
    }

    /// Given a `needle` coordinate, select the corresponding index and value.
    fn get_range(&self, needle: u16) -> Option<usize> {
        let mut start = self.state.inner_rect.x;
        let range = self
            .state
            .calculated_widths
            .iter()
            .map(|width| {
                let entry_start = start;
                start += width + 1; // +1 for the gap b/w cols.

                entry_start
            })
            .collect_vec();

        match range.binary_search(&needle) {
            Ok(index) => Some(index),
            Err(index) => index.checked_sub(1),
        }
    }
}
