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
    pub fn with_two_bounds(width: u16, height: u16) -> Self {
        Self {
            min_width: width,
            min_height: height,
            max_width: width,
            max_height: height,
        }
    }
}
