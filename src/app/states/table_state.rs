use std::{borrow::Cow, convert::TryInto};

use tui::widgets::TableState;

use super::ScrollDirection;

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
#[derive(Clone)]
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
}

pub trait TableComponentHeader {
    fn header_text(&self) -> &CellContent;
}

impl TableComponentHeader for CellContent {
    fn header_text(&self) -> &CellContent {
        self
    }
}

impl From<&'static str> for CellContent {
    fn from(s: &'static str) -> Self {
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
    pub fn new(header: H, width_bounds: WidthBounds) -> Self {
        Self {
            header,
            width_bounds,
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub fn default_hard(header: H) -> Self {
        let width = header.header_text().len() as u16;
        Self {
            header,
            width_bounds: WidthBounds::Hard(width),
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub fn default_soft(header: H, max_percentage: Option<f32>) -> Self {
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

pub enum SortOrder {
    Ascending,
    Descending,
}

/// Represents the current table's sorting state.
pub enum SortState {
    Unsortable,
    Sortable { index: usize, order: SortOrder },
}

impl Default for SortState {
    fn default() -> Self {
        SortState::Unsortable
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
            sort_state: Default::default(),
        }
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

        for column in self.columns.iter_mut() {
            column.calculated_width = 0;
        }

        let columns = if left_to_right {
            Either::Left(self.columns.iter_mut())
        } else {
            Either::Right(self.columns.iter_mut().rev())
        };

        let arrow_offset = match self.sort_state {
            SortState::Unsortable => 0,
            SortState::Sortable { index: _, order: _ } => 1,
        };
        let mut num_columns = 0;
        for column in columns {
            if column.is_hidden {
                continue;
            }

            match &column.width_bounds {
                WidthBounds::Soft {
                    min_width,
                    desired,
                    max_percentage,
                } => {
                    let offset_min = *min_width + arrow_offset;
                    let soft_limit = max(
                        if let Some(max_percentage) = max_percentage {
                            // Rust doesn't have an `into()` or `try_into()` for floats to integers???
                            ((*max_percentage * f32::from(total_width)).ceil()) as u16
                        } else {
                            *desired
                        },
                        offset_min,
                    );
                    let space_taken = min(min(soft_limit, *desired), total_width_left);

                    if offset_min > space_taken {
                        break;
                    } else if space_taken > 0 {
                        total_width_left = total_width_left.saturating_sub(space_taken + 1);
                        column.calculated_width = space_taken;
                        num_columns += 1;
                    }
                }
                WidthBounds::Hard(width) => {
                    let min_width = *width + arrow_offset;
                    let space_taken = min(min_width, total_width_left);

                    if min_width > space_taken {
                        break;
                    } else if space_taken > 0 {
                        total_width_left = total_width_left.saturating_sub(space_taken + 1);
                        column.calculated_width = space_taken;
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
            sort_state: Default::default(),
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
            TableComponentColumn::default_hard(CellContent::from("a")),
            TableComponentColumn::new(
                "a".into(),
                WidthBounds::Soft {
                    min_width: 1,
                    desired: 10,
                    max_percentage: Some(0.125),
                },
            ),
            TableComponentColumn::new(
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

        state.sort_state = SortState::Sortable {
            index: 1,
            order: SortOrder::Ascending,
        };

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
}
