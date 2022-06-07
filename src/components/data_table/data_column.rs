use std::borrow::Cow;

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

    /// Always uses the width of the [`CellContent`].
    CellWidth,
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

pub struct DataColumn {
    /// The header string value of the column.
    pub header: Cow<'static, str>,

    /// A restriction on this column's width.
    pub width_bounds: ColumnWidthBounds,

    /// The calculated width of the column.
    pub calculated_width: u16,

    /// Marks that this column is currently "hidden", and should *always* be skipped.
    pub is_hidden: bool,
}
