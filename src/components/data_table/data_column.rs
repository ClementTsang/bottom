use std::{
    borrow::Cow,
    cmp::{max, min},
};

/// A bound on the width of a column.
#[derive(Clone, Copy, Debug)]
pub enum ColumnWidthBounds {
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

    /// Always uses the width of the header.
    HeaderWidth,
}

impl ColumnWidthBounds {
    pub const fn soft(name: &'static str, max_percentage: Option<f32>) -> ColumnWidthBounds {
        let len = name.len() as u16;
        ColumnWidthBounds::Soft {
            min_width: len,
            desired: len,
            max_percentage,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataTableColumn {
    /// The header value of the column.
    pub header: Cow<'static, str>, // FIXME: May want to make this customizable

    /// A restriction on this column's width.
    pub width_bounds: ColumnWidthBounds,

    /// The calculated width of the column.
    pub calculated_width: u16,

    /// Marks that this column is currently "hidden", and should *always* be skipped.
    pub is_hidden: bool,
}

impl DataTableColumn {
    pub const fn hard(name: &'static str, width: u16) -> Self {
        Self {
            header: Cow::Borrowed(name),
            width_bounds: ColumnWidthBounds::Hard(width),
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub const fn soft(name: &'static str, max_percentage: Option<f32>) -> Self {
        Self {
            header: Cow::Borrowed(name),
            width_bounds: ColumnWidthBounds::soft(name, max_percentage),
            calculated_width: 0,
            is_hidden: false,
        }
    }

    pub const fn header(name: &'static str) -> Self {
        Self {
            header: Cow::Borrowed(name),
            width_bounds: ColumnWidthBounds::HeaderWidth,
            calculated_width: 0,
            is_hidden: false,
        }
    }
}

pub trait CalculateColumnWidth {
    /// Calculates widths for the columns of this table, given the current width when called.
    ///
    /// * `total_width` is the, well, total width available.
    /// * `left_to_right` is a boolean whether to go from left to right if true, or right to left if
    ///   false.
    ///
    /// **NOTE:** Trailing 0's may break tui-rs, remember to filter them out later!
    fn calculate_column_widths(&mut self, total_width: u16, left_to_right: bool);
}

impl CalculateColumnWidth for [DataTableColumn] {
    fn calculate_column_widths(&mut self, total_width: u16, left_to_right: bool) {
        use itertools::Either;

        let mut total_width_left = total_width;

        let columns = if left_to_right {
            Either::Left(self.iter_mut())
        } else {
            Either::Right(self.iter_mut().rev())
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
                ColumnWidthBounds::HeaderWidth => {
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

            for column in self.iter_mut() {
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
}
