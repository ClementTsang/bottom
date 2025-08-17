use std::{borrow::Cow, marker::PhantomData, num::NonZeroU16};

use concat_string::concat_string;
use itertools::Itertools;
use tui::widgets::Row;

use super::{
    ColumnHeader, ColumnWidthBounds, DataTable, DataTableColumn, DataTableProps, DataTableState,
    DataTableStyling, DataToCell,
};
use crate::utils::strings::truncate_to_text;

/// Denotes the sort order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    /// Returns the reverse [`SortOrder`].
    pub fn rev(&self) -> SortOrder {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    /// A hack to get a const default.
    pub const fn const_default() -> Self {
        Self::Ascending
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::const_default()
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
/// to denote whether the table is meant to display/store sorted or unsorted
/// data.
///
/// Note that the trait is [sealed](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed),
/// and therefore only [`Unsortable`] and [`Sortable`] can implement it.
pub trait SortType: private::Sealed {
    /// Constructs the table header.
    fn build_header<H, C>(&self, columns: &[C], widths: &[NonZeroU16]) -> Row<'_>
    where
        H: ColumnHeader,
        C: DataTableColumn<H>,
    {
        Row::new(
            columns
                .iter()
                .zip(widths)
                .map(|(c, &width)| truncate_to_text(&c.header(), width.get())),
        )
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
    fn build_header<H, C>(&self, columns: &[C], widths: &[NonZeroU16]) -> Row<'_>
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
                .map(|(index, (c, &width))| {
                    if index == self.sort_index {
                        let arrow = match self.order {
                            SortOrder::Ascending => UP_ARROW,
                            SortOrder::Descending => DOWN_ARROW,
                        };
                        // TODO: I think I can get away with removing the truncate_to_text call
                        // since I almost always bind to at least the header
                        // size... TODO: Or should we instead truncate but
                        // ALWAYS leave the arrow at the end?
                        truncate_to_text(&concat_string!(c.header(), arrow), width.get())
                    } else {
                        truncate_to_text(&c.header(), width.get())
                    }
                }),
        )
    }
}

pub trait SortsRow {
    type DataType;

    /// Sorts data.
    fn sort_data(&self, data: &mut [Self::DataType], descending: bool);
}

#[derive(Debug, Clone)]
pub struct SortColumn<T> {
    /// The inner column header.
    inner: T,

    /// The default sort order.
    pub default_order: SortOrder,

    /// A restriction on this column's width.
    pub bounds: ColumnWidthBounds,

    /// Marks that this column is currently "hidden", and should *always* be
    /// skipped.
    pub is_hidden: bool,
}

impl<D, T> DataTableColumn<T> for SortColumn<T>
where
    T: ColumnHeader + SortsRow<DataType = D>,
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

    fn header(&self) -> Cow<'static, str> {
        self.inner.header()
    }

    fn header_len(&self) -> usize {
        self.header().len() + 1
    }
}

impl<D, T> SortColumn<T>
where
    T: ColumnHeader + SortsRow<DataType = D>,
{
    /// Creates a new [`SortColumn`] with a width that follows the header width,
    /// which has no shortcut and sorts by default in ascending order
    /// ([`SortOrder::Ascending`]).
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::FollowHeader,
            is_hidden: false,
            default_order: SortOrder::default(),
        }
    }

    /// Creates a new [`SortColumn`] with a hard width, which has no shortcut
    /// and sorts by default in ascending order ([`SortOrder::Ascending`]).
    pub const fn hard(inner: T, width: u16) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Hard(width),
            is_hidden: false,
            default_order: SortOrder::const_default(),
        }
    }

    /// Creates a new [`SortColumn`] with a soft width, which has no shortcut
    /// and sorts by default in ascending order ([`SortOrder::Ascending`]).
    pub const fn soft(inner: T, max_percentage: Option<f32>) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Soft {
                desired: 0,
                max_percentage,
            },
            is_hidden: false,
            default_order: SortOrder::const_default(),
        }
    }

    /// Sets the default sort order to [`SortOrder::Descending`].
    pub const fn default_descending(mut self) -> Self {
        self.default_order = SortOrder::Descending;
        self
    }

    /// Given a [`SortColumn`] and the sort order, sort a mutable slice of
    /// associated data.
    pub fn sort_by(&self, data: &mut [D], order: SortOrder) {
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
pub type SortDataTable<DataType, H> = DataTable<DataType, H, Sortable, SortColumn<H>>;

impl<D, H> SortDataTable<D, H>
where
    D: DataToCell<H>,
    H: ColumnHeader + SortsRow<DataType = D>,
{
    pub fn new_sortable<C: Into<Vec<SortColumn<H>>>>(
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
            first_index: None,
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

    /// Given some `x` and `y`, if possible, select the corresponding column or
    /// toggle the column if already selected, and otherwise do nothing.
    ///
    /// If there was some update, the corresponding column type will be
    /// returned. If nothing happens, [`None`] is returned.
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
    /// If the index is different from the previous one, it will move to the new
    /// index and set the sort order to the prescribed default sort order.
    ///
    /// If the index is the same as the previous one, it will simply toggle the
    /// current sort order.
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
                start += width.get() + 1; // +1 for the gap b/w cols.

                entry_start
            })
            .collect_vec();

        match range.binary_search(&needle) {
            Ok(index) => Some(index),
            Err(index) => index.checked_sub(1),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct TestType {
        index: usize,
        data: u64,
    }

    enum ColumnType {
        Index,
        Data,
    }

    impl DataToCell<ColumnType> for TestType {
        fn to_cell_text(
            &self, _column: &ColumnType, _calculated_width: NonZeroU16,
        ) -> Option<Cow<'static, str>> {
            None
        }

        fn column_widths<C: DataTableColumn<ColumnType>>(_data: &[Self], _columns: &[C]) -> Vec<u16>
        where
            Self: Sized,
        {
            vec![]
        }
    }

    impl ColumnHeader for ColumnType {
        fn text(&self) -> Cow<'static, str> {
            match self {
                ColumnType::Index => "Index".into(),
                ColumnType::Data => "Data".into(),
            }
        }
    }

    impl SortsRow for ColumnType {
        type DataType = TestType;

        fn sort_data(&self, data: &mut [TestType], descending: bool) {
            match self {
                ColumnType::Index => data.sort_by_key(|t| t.index),
                ColumnType::Data => data.sort_by_key(|t| t.data),
            }

            if descending {
                data.reverse();
            }
        }
    }

    #[test]
    fn test_sorting() {
        let columns = [
            SortColumn::new(ColumnType::Index),
            SortColumn::new(ColumnType::Data),
        ];
        let props = {
            let inner = DataTableProps {
                title: Some("test".into()),
                table_gap: 1,
                left_to_right: false,
                is_basic: false,
                show_table_scroll_position: true,
                show_current_entry_when_unfocused: false,
            };

            SortDataTableProps {
                inner,
                sort_index: 0,
                order: SortOrder::Descending,
            }
        };

        let styling = DataTableStyling::default();

        let mut table = DataTable::new_sortable(columns, props, styling);
        let mut data = vec![
            TestType {
                index: 4,
                data: 100,
            },
            TestType {
                index: 1,
                data: 200,
            },
            TestType {
                index: 0,
                data: 300,
            },
            TestType {
                index: 3,
                data: 400,
            },
            TestType {
                index: 2,
                data: 500,
            },
        ];

        table
            .columns
            .get(table.sort_type.sort_index)
            .unwrap()
            .sort_by(&mut data, SortOrder::Ascending);
        assert_eq!(
            data,
            vec![
                TestType {
                    index: 0,
                    data: 300,
                },
                TestType {
                    index: 1,
                    data: 200,
                },
                TestType {
                    index: 2,
                    data: 500,
                },
                TestType {
                    index: 3,
                    data: 400,
                },
                TestType {
                    index: 4,
                    data: 100,
                },
            ]
        );

        table
            .columns
            .get(table.sort_type.sort_index)
            .unwrap()
            .sort_by(&mut data, SortOrder::Descending);
        assert_eq!(
            data,
            vec![
                TestType {
                    index: 4,
                    data: 100,
                },
                TestType {
                    index: 3,
                    data: 400,
                },
                TestType {
                    index: 2,
                    data: 500,
                },
                TestType {
                    index: 1,
                    data: 200,
                },
                TestType {
                    index: 0,
                    data: 300,
                },
            ]
        );

        table.set_sort_index(1);
        table
            .columns
            .get(table.sort_type.sort_index)
            .unwrap()
            .sort_by(&mut data, SortOrder::Ascending);
        assert_eq!(
            data,
            vec![
                TestType {
                    index: 4,
                    data: 100,
                },
                TestType {
                    index: 1,
                    data: 200,
                },
                TestType {
                    index: 0,
                    data: 300,
                },
                TestType {
                    index: 3,
                    data: 400,
                },
                TestType {
                    index: 2,
                    data: 500,
                },
            ]
        );
    }
}
