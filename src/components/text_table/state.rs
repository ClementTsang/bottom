use std::{borrow::Cow, convert::TryInto, ops::Range};

use itertools::Itertools;
use tui::{layout::Rect, widgets::TableState};

use crate::app::ScrollDirection;

/// A bound on the width of a column.
#[derive(Clone, Copy, Debug)]
pub enum WidthBounds {
    /// A width of this type is either as long as `min`, but can otherwise shrink and grow up to a point.
    Soft {
        /// The minimum amount before giving up and hiding.
        min_width: u16,

        /// The desired, calculated width. Take this if possible as the base starting width.
        desired: u16,

        /// The max width, as a percentage of the total width available. If [`None`],
        /// then it can grow as desired.
        max_percentage: Option<f32>,
    },

    /// A width of this type is either as long as specified, or does not appear at all.
    Hard(u16),

    /// Always uses the width of the [`CellContent`].
    CellWidth,
}

impl WidthBounds {
    pub const fn soft_from_str(name: &'static str, max_percentage: Option<f32>) -> WidthBounds {
        let len = name.len() as u16;
        WidthBounds::Soft {
            min_width: len,
            desired: len,
            max_percentage,
        }
    }

    pub const fn soft_from_str_with_alt(
        name: &'static str, alt: &'static str, max_percentage: Option<f32>,
    ) -> WidthBounds {
        WidthBounds::Soft {
            min_width: alt.len() as u16,
            desired: name.len() as u16,
            max_percentage,
        }
    }
}

/// A [`CellContent`] contains text information for display in a table.
#[derive(Clone, Debug)]
pub enum CellContent {
    Simple(Cow<'static, str>),
    HasAlt {
        alt: Cow<'static, str>,
        main: Cow<'static, str>,
    },
}

impl CellContent {
    /// Creates a new [`CellContent`].
    pub fn new<I>(name: I, alt: Option<I>) -> Self
    where
        I: Into<Cow<'static, str>>,
    {
        if let Some(alt) = alt {
            CellContent::HasAlt {
                alt: alt.into(),
                main: name.into(),
            }
        } else {
            CellContent::Simple(name.into())
        }
    }

    /// Returns the length of the [`CellContent`]. Note that for a [`CellContent::HasAlt`], it will return
    /// the length of the "main" field.
    pub fn len(&self) -> usize {
        match self {
            CellContent::Simple(s) => s.len(),
            CellContent::HasAlt { alt: _, main: long } => long.len(),
        }
    }

    /// Whether the [`CellContent`]'s text is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn main_text(&self) -> &Cow<'static, str> {
        match self {
            CellContent::Simple(main) => main,
            CellContent::HasAlt { alt: _, main } => main,
        }
    }
}

pub trait TableComponentHeader {
    fn header_text(&self) -> &CellContent;
}

impl TableComponentHeader for CellContent {
    fn header_text(&self) -> &CellContent {
        self
    }
}
impl From<Cow<'static, str>> for CellContent {
    fn from(c: Cow<'static, str>) -> Self {
        CellContent::Simple(c)
    }
}

impl From<&'static str> for CellContent {
    fn from(s: &'static str) -> Self {
        CellContent::Simple(s.into())
    }
}

impl From<String> for CellContent {
    fn from(s: String) -> Self {
        CellContent::Simple(s.into())
    }
}

pub struct TableComponentColumn<H: TableComponentHeader> {
    /// The header of the column.
    pub header: H,

    /// A restriction on this column's width, if desired.
    pub width_bounds: WidthBounds,

    /// The calculated width of the column.
    pub calculated_width: u16,

    /// Marks that this column is currently "hidden", and should *always* be skipped.
    pub is_hidden: bool,
}

impl<H: TableComponentHeader> TableComponentColumn<H> {
    pub fn new_custom(header: H, width_bounds: WidthBounds) -> Self {
        Self {
            header,
            width_bounds,
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub fn new(header: H) -> Self {
        Self {
            header,
            width_bounds: WidthBounds::CellWidth,
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub fn new_hard(header: H, width: u16) -> Self {
        Self {
            header,
            width_bounds: WidthBounds::Hard(width),
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub fn new_soft(header: H, max_percentage: Option<f32>) -> Self {
        let min_width = header.header_text().len() as u16;
        Self {
            header,
            width_bounds: WidthBounds::Soft {
                min_width,
                desired: min_width,
                max_percentage,
            },
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub fn is_zero_width(&self) -> bool {
        self.calculated_width == 0
    }

    pub fn is_skipped(&self) -> bool {
        self.is_zero_width() || self.is_hidden
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn is_descending(&self) -> bool {
        matches!(self, SortOrder::Descending)
    }
}

/// Represents the current table's sorting state.
#[derive(Debug)]
pub enum SortState {
    Unsortable,
    Sortable(SortableState),
}

#[derive(Debug)]
pub struct SortableState {
    /// The "x locations" of the headers.
    visual_mappings: Vec<Range<u16>>,

    /// The "y location" of the header row. Since all headers share the same y-location we just set it once here.
    y_loc: u16,

    /// This is a bit of a lazy hack to handle this for now - ideally the entire [`SortableState`]
    /// is instead handled by a separate table struct that also can access the columns and their default sort orderings.
    default_sort_orderings: Vec<SortOrder>,

    /// The currently selected sort index.
    pub current_index: usize,

    /// The current sorting order.
    pub order: SortOrder,
}

impl SortableState {
    /// Creates a new [`SortableState`].
    pub fn new(
        default_index: usize, default_order: SortOrder, default_sort_orderings: Vec<SortOrder>,
    ) -> Self {
        Self {
            visual_mappings: Default::default(),
            y_loc: 0,
            default_sort_orderings,
            current_index: default_index,
            order: default_order,
        }
    }

    /// Toggles the current sort order.
    pub fn toggle_order(&mut self) {
        self.order = match self.order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    /// Updates the visual index.
    ///
    /// This function will create a *sorted* range list - in debug mode,
    /// the program will assert this, but it will not do so in release mode!
    pub fn update_visual_index(&mut self, draw_loc: Rect, row_widths: &[u16]) {
        let mut start = draw_loc.x;
        let visual_index = row_widths
            .iter()
            .map(|width| {
                let range_start = start;
                let range_end = start + width + 1; // +1 for the gap b/w cols.
                start = range_end;
                range_start..range_end
            })
            .collect_vec();

        debug_assert!(visual_index.iter().all(|a| { a.start <= a.end }));

        debug_assert!(visual_index
            .iter()
            .tuple_windows()
            .all(|(a, b)| { b.start >= a.end }));

        self.visual_mappings = visual_index;
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
                Some(self.current_index)
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
        if self.current_index == index {
            self.toggle_order();
        } else {
            self.current_index = index;
            self.order = self.default_sort_orderings[index];
        }
    }

    /// Given a `needle` coordinate, select the corresponding index and value.
    fn get_range(&self, needle: u16) -> Option<usize> {
        match self
            .visual_mappings
            .binary_search_by_key(&needle, |range| range.start)
        {
            Ok(index) => Some(index),
            Err(index) => index.checked_sub(1),
        }
        .and_then(|index| {
            if needle < self.visual_mappings[index].end {
                Some(index)
            } else {
                None
            }
        })
    }
}

/// [`TableComponentState`] deals with fields for a scrollable's current state.
pub struct TableComponentState<H: TableComponentHeader = CellContent> {
    pub current_scroll_position: usize,
    pub scroll_bar: usize,
    pub scroll_direction: ScrollDirection,
    pub table_state: TableState,
    pub columns: Vec<TableComponentColumn<H>>,
    pub sort_state: SortState,
}

impl<H: TableComponentHeader> TableComponentState<H> {
    pub fn new(columns: Vec<TableComponentColumn<H>>) -> Self {
        Self {
            current_scroll_position: 0,
            scroll_bar: 0,
            scroll_direction: ScrollDirection::Down,
            table_state: Default::default(),
            columns,
            sort_state: SortState::Unsortable,
        }
    }

    pub fn sort_state(mut self, sort_state: SortState) -> Self {
        self.sort_state = sort_state;
        self
    }

    /// Calculates widths for the columns for this table.
    ///
    /// * `total_width` is the, well, total width available.
    /// * `left_to_right` is a boolean whether to go from left to right if true, or right to left if
    ///   false.
    ///
    /// **NOTE:** Trailing 0's may break tui-rs, remember to filter them out later!
    pub fn calculate_column_widths(&mut self, total_width: u16, left_to_right: bool) {
        use itertools::Either;
        use std::cmp::{max, min};

        let mut total_width_left = total_width;

        let columns = if left_to_right {
            Either::Left(self.columns.iter_mut())
        } else {
            Either::Right(self.columns.iter_mut().rev())
        };

        let arrow_offset = match self.sort_state {
            SortState::Unsortable => 0,
            SortState::Sortable { .. } => 1,
        };

        let mut num_columns = 0;
        let mut skip_iter = false;
        for column in columns {
            column.calculated_width = 0;

            if column.is_hidden || skip_iter {
                continue;
            }

            match &column.width_bounds {
                WidthBounds::Soft {
                    min_width,
                    desired,
                    max_percentage,
                } => {
                    let min_width = *min_width + arrow_offset;
                    if min_width > total_width_left {
                        skip_iter = true;
                        continue;
                    }

                    let soft_limit = max(
                        if let Some(max_percentage) = max_percentage {
                            // TODO: Rust doesn't have an `into()` or `try_into()` for floats to integers.
                            ((*max_percentage * f32::from(total_width)).ceil()) as u16
                        } else {
                            *desired
                        },
                        min_width,
                    );
                    let space_taken = min(min(soft_limit, *desired), total_width_left);

                    if min_width > space_taken || min_width == 0 {
                        skip_iter = true;
                    } else if space_taken > 0 {
                        total_width_left = total_width_left.saturating_sub(space_taken + 1);
                        column.calculated_width = space_taken;
                        num_columns += 1;
                    }
                }
                WidthBounds::CellWidth => {
                    let width = column.header.header_text().len() as u16;
                    let min_width = width + arrow_offset;

                    if min_width > total_width_left || min_width == 0 {
                        skip_iter = true;
                    } else if min_width > 0 {
                        total_width_left = total_width_left.saturating_sub(min_width + 1);
                        column.calculated_width = min_width;
                        num_columns += 1;
                    }
                }
                WidthBounds::Hard(width) => {
                    let min_width = *width + arrow_offset;

                    if min_width > total_width_left || min_width == 0 {
                        skip_iter = true;
                    } else if min_width > 0 {
                        total_width_left = total_width_left.saturating_sub(min_width + 1);
                        column.calculated_width = min_width;
                        num_columns += 1;
                    }
                }
            }
        }

        if num_columns > 0 {
            // Redistribute remaining.
            let mut num_dist = num_columns;
            let amount_per_slot = total_width_left / num_dist;
            total_width_left %= num_dist;

            for column in self.columns.iter_mut() {
                if num_dist == 0 {
                    break;
                }

                if column.calculated_width > 0 {
                    if total_width_left > 0 {
                        column.calculated_width += amount_per_slot + 1;
                        total_width_left -= 1;
                    } else {
                        column.calculated_width += amount_per_slot;
                    }

                    num_dist -= 1;
                }
            }
        }
    }

    /// Updates the position if possible, and if there is a valid change, returns the new position.
    pub fn update_position(&mut self, change: i64, num_entries: usize) -> Option<usize> {
        if change == 0 {
            return None;
        }

        let csp: Result<i64, _> = self.current_scroll_position.try_into();
        if let Ok(csp) = csp {
            let proposed: Result<usize, _> = (csp + change).try_into();
            if let Ok(proposed) = proposed {
                if proposed < num_entries {
                    self.current_scroll_position = proposed;
                    if change < 0 {
                        self.scroll_direction = ScrollDirection::Up;
                    } else {
                        self.scroll_direction = ScrollDirection::Down;
                    }

                    return Some(self.current_scroll_position);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scroll_update_position() {
        #[track_caller]
        fn check_scroll_update(
            scroll: &mut TableComponentState, change: i64, max: usize, ret: Option<usize>,
            new_position: usize,
        ) {
            assert_eq!(scroll.update_position(change, max), ret);
            assert_eq!(scroll.current_scroll_position, new_position);
        }

        let mut scroll = TableComponentState {
            current_scroll_position: 5,
            scroll_bar: 0,
            scroll_direction: ScrollDirection::Down,
            table_state: Default::default(),
            columns: vec![],
            sort_state: SortState::Unsortable,
        };
        let s = &mut scroll;

        // Update by 0. Should not change.
        check_scroll_update(s, 0, 15, None, 5);

        // Update by 5. Should increment to index 10.
        check_scroll_update(s, 5, 15, Some(10), 10);

        // Update by 5. Should not change.
        check_scroll_update(s, 5, 15, None, 10);

        // Update by 4. Should increment to index 14 (supposed max).
        check_scroll_update(s, 4, 15, Some(14), 14);

        // Update by 1. Should do nothing.
        check_scroll_update(s, 1, 15, None, 14);

        // Update by -15. Should do nothing.
        check_scroll_update(s, -15, 15, None, 14);

        // Update by -14. Should land on position 0.
        check_scroll_update(s, -14, 15, Some(0), 0);

        // Update by -1. Should do nothing.
        check_scroll_update(s, -15, 15, None, 0);

        // Update by 0. Should do nothing.
        check_scroll_update(s, 0, 15, None, 0);

        // Update by 15. Should do nothing.
        check_scroll_update(s, 15, 15, None, 0);

        // Update by 15 but with a larger bound. Should increment to 15.
        check_scroll_update(s, 15, 16, Some(15), 15);
    }

    #[test]
    fn test_table_width_calculation() {
        #[track_caller]
        fn test_calculation(state: &mut TableComponentState, width: u16, expected: Vec<u16>) {
            state.calculate_column_widths(width, true);
            assert_eq!(
                state
                    .columns
                    .iter()
                    .filter_map(|c| if c.calculated_width == 0 {
                        None
                    } else {
                        Some(c.calculated_width)
                    })
                    .collect::<Vec<_>>(),
                expected
            )
        }

        let mut state = TableComponentState::new(vec![
            TableComponentColumn::new(CellContent::from("a")),
            TableComponentColumn::new_custom(
                "a".into(),
                WidthBounds::Soft {
                    min_width: 1,
                    desired: 10,
                    max_percentage: Some(0.125),
                },
            ),
            TableComponentColumn::new_custom(
                "a".into(),
                WidthBounds::Soft {
                    min_width: 2,
                    desired: 10,
                    max_percentage: Some(0.5),
                },
            ),
        ]);

        test_calculation(&mut state, 0, vec![]);
        test_calculation(&mut state, 1, vec![1]);
        test_calculation(&mut state, 2, vec![1]);
        test_calculation(&mut state, 3, vec![1, 1]);
        test_calculation(&mut state, 4, vec![1, 1]);
        test_calculation(&mut state, 5, vec![2, 1]);
        test_calculation(&mut state, 6, vec![1, 1, 2]);
        test_calculation(&mut state, 7, vec![1, 1, 3]);
        test_calculation(&mut state, 8, vec![1, 1, 4]);
        test_calculation(&mut state, 14, vec![2, 2, 7]);
        test_calculation(&mut state, 20, vec![2, 4, 11]);
        test_calculation(&mut state, 100, vec![27, 35, 35]);

        state.sort_state = SortState::Sortable(SortableState::new(1, SortOrder::Ascending, vec![]));

        test_calculation(&mut state, 0, vec![]);
        test_calculation(&mut state, 1, vec![]);
        test_calculation(&mut state, 2, vec![2]);
        test_calculation(&mut state, 3, vec![2]);
        test_calculation(&mut state, 4, vec![3]);
        test_calculation(&mut state, 5, vec![2, 2]);
        test_calculation(&mut state, 6, vec![2, 2]);
        test_calculation(&mut state, 7, vec![3, 2]);
        test_calculation(&mut state, 8, vec![3, 3]);
        test_calculation(&mut state, 14, vec![2, 2, 7]);
        test_calculation(&mut state, 20, vec![3, 4, 10]);
        test_calculation(&mut state, 100, vec![27, 35, 35]);
    }

    #[test]
    fn test_visual_index_selection() {
        let mut state = SortableState::new(
            0,
            SortOrder::Ascending,
            vec![SortOrder::Ascending, SortOrder::Descending],
        );

        const X_OFFSET: u16 = 10;
        const Y_OFFSET: u16 = 15;
        state.update_visual_index(Rect::new(X_OFFSET, Y_OFFSET, 20, 15), &[4, 14]);

        #[track_caller]
        fn test_selection(
            state: &mut SortableState, from_x_offset: u16, from_y_offset: u16,
            result: (Option<usize>, SortOrder),
        ) {
            assert_eq!(
                state.try_select_location(X_OFFSET + from_x_offset, Y_OFFSET + from_y_offset),
                result.0
            );
            assert_eq!(state.order, result.1);
        }

        use SortOrder::*;

        // Clicking on these don't do anything, so don't show any change.
        test_selection(&mut state, 5, 1, (None, Ascending));
        test_selection(&mut state, 21, 0, (None, Ascending));

        // Clicking on the first column should toggle it as it is already selected.
        test_selection(&mut state, 3, 0, (Some(0), Descending));

        // Clicking on the first column should toggle it again as it is already selected.
        test_selection(&mut state, 4, 0, (Some(0), Ascending));

        // Clicking on second column should select and switch to the descending ordering as that is its default.
        test_selection(&mut state, 5, 0, (Some(1), Descending));

        // Clicking on second column should toggle it.
        test_selection(&mut state, 19, 0, (Some(1), Ascending));

        // Overshoot, should not do anything.
        test_selection(&mut state, 20, 0, (None, Ascending));

        // Further overshoot, should not do anything.
        test_selection(&mut state, 25, 0, (None, Ascending));

        // Go back to first column, should be ascending to match default for index 0.
        test_selection(&mut state, 3, 0, (Some(0), Ascending));

        // Click on first column should then go to descending as it is already selected and ascending.
        test_selection(&mut state, 3, 0, (Some(0), Descending));
    }
}
