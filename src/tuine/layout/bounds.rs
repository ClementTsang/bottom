use crate::tuine::Size;

/// [`Bounds`] represent minimal and maximal widths/height constraints while laying things out.
///
/// These are sent from a parent component to a child to determine the [`Size`](super::Size)
/// of a child, which is passed back up to the parent.
#[derive(Clone, Copy, Default)]
pub struct Bounds {
    /// The minimal width available.
    pub min_width: u16,

    /// The minimal height available.
    pub min_height: u16,

    /// The maximal width available.
    pub max_width: u16,

    /// The maximal height available.
    pub max_height: u16,
}

impl Bounds {
    /// Shrinks the current bounds by some amount.
    pub fn shrink(&mut self, width: u16, height: u16) {
        self.max_width = self.max_width.saturating_sub(width);
        self.max_height = self.max_height.saturating_sub(height);
    }

    /// Shrinks by a given [`Size`].
    pub fn shrink_size(&mut self, size: Size) {
        self.max_width = self.max_width.saturating_sub(size.width);
        self.max_height = self.max_height.saturating_sub(size.height);
    }

    /// Returns whether there is any space left in this bound for laying out things.
    pub fn has_space(&self) -> bool {
        self.min_width > self.max_width
            || self.min_height > self.max_height
            || self.max_width == 0
            || self.max_height == 0
    }
}
