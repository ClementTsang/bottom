use std::cmp::{max, min};

use anyhow::{bail, Result};

/// A bound on the width of a column.
#[derive(Clone, Copy, Debug)]
pub enum ColumnWidthBounds {
    /// A width of this type is either as long as `min`, but can otherwise shrink and grow up to a point.
    Soft {
        /// The desired, calculated width. Take this if possible as the base starting width.
        desired: u16,

        /// The max width, as a percentage of the total width available. If [`None`],
        /// then it can grow as desired.
        max_percentage: Option<f32>,
    },

    /// A width of this type is either as long as specified, or does not appear at all.
    Hard(u16),
}

pub trait ColumnDisplay {
    /// The "text" version of the column.
    fn text(&self) -> String;

    /// The actually displayed "header".
    ///
    /// The default implementation just uses [`ColumnDisplay::text`].
    fn header(&self) -> String {
        self.text()
    }
}

impl ColumnDisplay for &str {
    fn text(&self) -> String {
        self.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct ColumnInfo<T: ColumnDisplay> {
    /// The inner column header.
    inner: T,

    /// A restriction on this column's width.
    bounds: ColumnWidthBounds,
}

impl<T: ColumnDisplay> ColumnInfo<T> {
    pub const fn hard(inner: T, width: u16) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Hard(width),
        }
    }

    pub const fn soft(inner: T, max_percentage: Option<f32>) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Soft {
                desired: 0,
                max_percentage,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct SwitcherCol<T: ColumnDisplay> {
    cols: Vec<ColumnInfo<T>>,
    selected: usize,
}

impl<T: ColumnDisplay> SwitcherCol<T> {
    pub fn set_index(&mut self, new_index: usize) -> Result<()> {
        if new_index < self.cols.len() {
            self.selected = new_index;
            Ok(())
        } else {
            bail!("index not in range")
        }
    }
}

#[derive(Clone, Debug)]
pub enum ColumnType<T: ColumnDisplay> {
    Single(ColumnInfo<T>),
    Switcher(SwitcherCol<T>),
}

impl<T: ColumnDisplay> ColumnType<T> {
    pub fn new(col: ColumnInfo<T>) -> Self {
        Self::Single(col)
    }

    pub fn new_switcher(cols: Vec<ColumnInfo<T>>, index: usize) -> Self {
        let selected = index.clamp(0, cols.len().saturating_sub(1));
        Self::Switcher(SwitcherCol { cols, selected })
    }
}

#[derive(Clone, Debug)]
pub struct Column<T: ColumnDisplay> {
    col: ColumnType<T>,

    /// Marks that this column is currently "hidden", and should *always* be skipped.
    pub is_hidden: bool,
}

impl<T: ColumnDisplay> Column<T> {
    pub const fn hard(inner: T, width: u16) -> Self {
        Self {
            col: ColumnType::Single(ColumnInfo::hard(inner, width)),
            is_hidden: false,
        }
    }

    pub const fn soft(inner: T, max_percentage: Option<f32>) -> Self {
        Self {
            col: ColumnType::Single(ColumnInfo::soft(inner, max_percentage)),
            is_hidden: false,
        }
    }

    pub fn bounds(&self) -> &ColumnWidthBounds {
        match &self.col {
            ColumnType::Single(col) => &col.bounds,
            ColumnType::Switcher(SwitcherCol { cols, selected }) => &cols[*selected].bounds,
        }
    }

    pub fn adjust_inner_width(&mut self, width: u16) {
        let col = match &mut self.col {
            ColumnType::Single(col) => col,
            ColumnType::Switcher(SwitcherCol { cols, selected }) => &mut cols[*selected],
        };

        match &mut col.bounds {
            ColumnWidthBounds::Soft { desired, .. } => {
                *desired = max(col.inner.header().len() as u16, width);
            }
            ColumnWidthBounds::Hard(_) => {}
        }
    }

    pub fn inner(&self) -> &T {
        match &self.col {
            ColumnType::Single(col) => &col.inner,
            ColumnType::Switcher(SwitcherCol { cols, selected }) => &cols[*selected].inner,
        }
    }

    pub fn inner_text(&self) -> String {
        self.inner().text()
    }

    pub fn inner_header(&self) -> String {
        self.inner().header()
    }
}

pub trait DrawDataColumn {
    /// Calculates widths for the columns of this table, given the current width when called.
    ///
    /// * `total_width` is the, well, total width available.
    /// * `left_to_right` is a boolean whether to go from left to right if true, or right to left if
    ///   false.
    ///
    /// **NOTE:** Trailing 0's may break tui-rs, remember to filter them out later!
    fn calculate_column_widths(&self, total_width: u16, left_to_right: bool) -> Vec<u16>;
}

impl<T: ColumnDisplay> DrawDataColumn for [Column<T>] {
    fn calculate_column_widths(&self, total_width: u16, left_to_right: bool) -> Vec<u16> {
        use itertools::Either;

        let mut total_width_left = total_width;
        let mut calculated_widths = vec![0; self.len()];
        let columns = if left_to_right {
            Either::Left(self.iter().zip(calculated_widths.iter_mut()))
        } else {
            Either::Right(self.iter().zip(calculated_widths.iter_mut()).rev())
        };

        let mut num_columns = 0;
        for (column, calculated_width) in columns {
            if column.is_hidden {
                continue;
            }

            match column.bounds() {
                ColumnWidthBounds::Soft {
                    desired,
                    max_percentage,
                } => {
                    let min_width = column.inner_header().len() as u16;
                    if min_width > total_width_left {
                        break;
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
                        break;
                    } else if space_taken > 0 {
                        total_width_left = total_width_left.saturating_sub(space_taken + 1);
                        *calculated_width = space_taken;
                        num_columns += 1;
                    }
                }
                ColumnWidthBounds::Hard(width) => {
                    let min_width = *width;

                    if min_width > total_width_left || min_width == 0 {
                        break;
                    } else if min_width > 0 {
                        total_width_left = total_width_left.saturating_sub(min_width + 1);
                        *calculated_width = min_width;
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

            for width in calculated_widths.iter_mut() {
                if num_dist == 0 {
                    break;
                }

                if *width > 0 {
                    if total_width_left > 0 {
                        *width += amount_per_slot + 1;
                        total_width_left -= 1;
                    } else {
                        *width += amount_per_slot;
                    }

                    num_dist -= 1;
                }
            }
        }

        calculated_widths
    }
}
