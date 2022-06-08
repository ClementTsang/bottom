use std::{convert::TryInto, marker::PhantomData};

pub mod data_row;
pub use data_row::*;

pub mod data_column;
pub use data_column::*;

pub mod styling;
pub use styling::*;

pub mod props;
pub use props::DataTableProps;

pub mod state;
pub use state::{DataTableState, ScrollDirection};

/// A [`DataTable`] is a component that displays data in a tabular form.
///
/// Note that the data is not guaranteed to be sorted, or managed in any way. If a
/// sortable variant is needed, use a [`SortableDataTable`](crate::components::data_table::SortableDataTable)
/// instead.
pub struct DataTable<RowType: ToDataRow> {
    /// The columns of the [`DataTable`].
    pub columns: Vec<DataColumn>,

    /// Styling for the [`DataTable`].
    pub styling: Styling,

    /// Internal state of the [`DataTable`].
    pub state: DataTableState,

    /// Internal properties of a [`DataTable`].
    pub props: DataTableProps,

    _pd: PhantomData<RowType>,
}

impl<RowType: ToDataRow> DataTable<RowType> {
    pub fn new() -> Self {
        todo!()
    }

    /// Calculates widths for the columns of this table, given the current width when called.
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

        let mut num_columns = 0;
        let mut skip_iter = false;
        for column in columns {
            column.calculated_width = 0;

            if column.is_hidden || skip_iter {
                continue;
            }

            match &column.width_bounds {
                ColumnWidthBounds::Soft {
                    min_width,
                    desired,
                    max_percentage,
                } => {
                    let min_width = *min_width;
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
                ColumnWidthBounds::CellWidth => {
                    let width = column.header.len() as u16;
                    let min_width = width;

                    if min_width > total_width_left || min_width == 0 {
                        skip_iter = true;
                    } else if min_width > 0 {
                        total_width_left = total_width_left.saturating_sub(min_width + 1);
                        column.calculated_width = min_width;
                        num_columns += 1;
                    }
                }
                ColumnWidthBounds::Hard(width) => {
                    let min_width = *width;

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

    /// Updates the scroll position if possible by a positive/negative offset. If there is a
    /// valid change, this function will also return the new position wrapped in an [`Option`].
    pub fn update_scroll_position(&mut self, change: i64, num_entries: usize) -> Option<usize> {
        if change == 0 {
            return None;
        }

        let csp: Result<i64, _> = self.state.current_scroll_position.try_into();
        if let Ok(csp) = csp {
            let proposed: Result<usize, _> = (csp + change).try_into();
            if let Ok(proposed) = proposed {
                if proposed < num_entries {
                    self.state.current_scroll_position = proposed;
                    if change < 0 {
                        self.state.scroll_direction = ScrollDirection::Up;
                    } else {
                        self.state.scroll_direction = ScrollDirection::Down;
                    }

                    return Some(self.state.current_scroll_position);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {}
