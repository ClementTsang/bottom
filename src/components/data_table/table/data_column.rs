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
pub struct DataColumn {
    /// The header value of the column.
    pub header: Cow<'static, str>, // FIXME: May want to make this customizable

    /// A restriction on this column's width.
    pub width_bounds: ColumnWidthBounds,

    /// The calculated width of the column.
    pub calculated_width: u16,

    /// Marks that this column is currently "hidden", and should *always* be skipped.
    pub is_hidden: bool,
}

impl DataColumn {
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
