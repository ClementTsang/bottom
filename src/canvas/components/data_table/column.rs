use std::{
    borrow::Cow,
    cmp::{max, min},
    num::NonZeroU16,
};

/// A bound on the width of a column.
#[derive(Clone, Copy, Debug)]
pub enum ColumnWidthBounds {
    /// A width of this type is as long as `desired`, but can otherwise shrink
    /// and grow up to a point.
    Soft {
        /// The desired, calculated width. Take this if possible as the base
        /// starting width.
        desired: u16,

        /// The max width, as a percentage of the total width available. If
        /// [`None`], then it can grow as desired.
        max_percentage: Option<f32>,
    },

    /// A width of this type is either as long as specified, or does not appear
    /// at all.
    Hard(u16),

    /// A width of this type always resizes to the column header's text width.
    FollowHeader,
}

pub trait ColumnHeader {
    /// The "text" version of the column header.
    fn text(&self) -> Cow<'static, str>;

    /// The version displayed when drawing the table. Defaults to
    /// [`ColumnHeader::text`].
    #[inline(always)]
    fn header(&self) -> Cow<'static, str> {
        self.text()
    }
}

impl ColumnHeader for &'static str {
    fn text(&self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

impl ColumnHeader for String {
    fn text(&self) -> Cow<'static, str> {
        Cow::Owned(self.clone())
    }
}

pub trait DataTableColumn<H: ColumnHeader> {
    fn inner(&self) -> &H;

    fn inner_mut(&mut self) -> &mut H;

    fn bounds(&self) -> ColumnWidthBounds;

    fn bounds_mut(&mut self) -> &mut ColumnWidthBounds;

    fn is_hidden(&self) -> bool;

    /// The actually displayed "header".
    fn header(&self) -> Cow<'static, str>;

    /// The header length, along with any required additional lengths for things
    /// like arrows. Defaults to getting the length of
    /// [`DataTableColumn::header`].
    fn header_len(&self) -> usize {
        self.header().len()
    }
}

#[derive(Clone, Debug)]
pub struct Column<H> {
    /// The inner column header.
    inner: H,

    /// A restriction on this column's width.
    bounds: ColumnWidthBounds,

    /// Marks that this column is currently "hidden", and should *always* be
    /// skipped.
    is_hidden: bool,
}

impl<H: ColumnHeader> DataTableColumn<H> for Column<H> {
    #[inline]
    fn inner(&self) -> &H {
        &self.inner
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut H {
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
        self.inner.text()
    }
}

impl<H: ColumnHeader> Column<H> {
    pub const fn hard(inner: H, width: u16) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Hard(width),
            is_hidden: false,
        }
    }

    pub const fn soft(inner: H, max_percentage: Option<f32>) -> Self {
        Self {
            inner,
            bounds: ColumnWidthBounds::Soft {
                desired: 0,
                max_percentage,
            },
            is_hidden: false,
        }
    }
}

pub trait CalculateColumnWidths<H> {
    /// Calculates widths for the columns of this table, given the current width
    /// when called.
    ///
    /// * `total_width` is the total width on the canvas that the columns can
    ///   try and work with.
    /// * `left_to_right` is whether to size from left-to-right (`true`) or
    ///   right-to-left (`false`).
    fn calculate_column_widths(&self, total_width: u16, left_to_right: bool) -> Vec<NonZeroU16>;
}

impl<H, C> CalculateColumnWidths<H> for [C]
where
    H: ColumnHeader,
    C: DataTableColumn<H>,
{
    fn calculate_column_widths(&self, total_width: u16, left_to_right: bool) -> Vec<NonZeroU16> {
        use itertools::Either;

        const COLUMN_SPACING: u16 = 1;

        #[inline]
        fn stop_allocating_space(desired: u16, available: u16) -> bool {
            desired > available || desired == 0
        }

        let mut total_width_left = total_width;
        let mut calculated_widths = vec![];
        let columns = if left_to_right {
            Either::Left(self.iter())
        } else {
            Either::Right(self.iter().rev())
        };

        for column in columns {
            if column.is_hidden() {
                continue;
            }

            match &column.bounds() {
                ColumnWidthBounds::Soft {
                    desired,
                    max_percentage,
                } => {
                    let min_width = column.header_len() as u16;
                    if min_width > total_width_left {
                        break;
                    }

                    let soft_limit = max(
                        if let Some(max_percentage) = max_percentage {
                            ((*max_percentage * f32::from(total_width)).ceil()) as u16
                        } else {
                            *desired
                        },
                        min_width,
                    );
                    let space_taken = min(min(soft_limit, *desired), total_width_left);

                    if stop_allocating_space(space_taken, total_width_left) {
                        break;
                    } else {
                        total_width_left =
                            total_width_left.saturating_sub(space_taken + COLUMN_SPACING);

                        // SAFETY: This is safe as we call `stop_allocating_space` which checks that
                        // the value pushed is greater than zero.
                        unsafe {
                            calculated_widths.push(NonZeroU16::new_unchecked(space_taken));
                        }
                    }
                }
                ColumnWidthBounds::Hard(width) => {
                    let min_width = *width;
                    if stop_allocating_space(min_width, total_width_left) {
                        break;
                    } else {
                        total_width_left =
                            total_width_left.saturating_sub(min_width + COLUMN_SPACING);

                        // SAFETY: This is safe as we call `stop_allocating_space` which checks that
                        // the value pushed is greater than zero.
                        unsafe {
                            calculated_widths.push(NonZeroU16::new_unchecked(min_width));
                        }
                    }
                }
                ColumnWidthBounds::FollowHeader => {
                    let min_width = column.header_len() as u16;
                    if stop_allocating_space(min_width, total_width_left) {
                        break;
                    } else {
                        total_width_left =
                            total_width_left.saturating_sub(min_width + COLUMN_SPACING);

                        // SAFETY: This is safe as we call `stop_allocating_space` which checks that
                        // the value pushed is greater than zero.
                        unsafe {
                            calculated_widths.push(NonZeroU16::new_unchecked(min_width));
                        }
                    }
                }
            }
        }

        if !calculated_widths.is_empty() {
            if !left_to_right {
                calculated_widths.reverse();
            }

            // Redistribute remaining space.
            let mut num_dist = calculated_widths.len() as u16;
            let amount_per_slot = total_width_left / num_dist; // Safe from DBZ by above empty check.
            total_width_left %= num_dist;

            for width in calculated_widths.iter_mut() {
                if num_dist == 0 {
                    break;
                }

                if total_width_left > 0 {
                    *width = width.saturating_add(amount_per_slot + 1);
                    total_width_left -= 1;
                } else {
                    *width = width.saturating_add(amount_per_slot);
                }

                num_dist -= 1;
            }
        }

        calculated_widths
    }
}
