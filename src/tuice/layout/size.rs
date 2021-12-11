/// A [`Size`] represents calculated widths and heights for a component.
///
/// A [`Size`] is sent from a child component back up to its parents after
/// first being given a [`Bounds`](super::Bounds) from the parent.
pub struct Size {
    /// The given width.
    pub width: u16,

    /// The given height.
    pub height: u16,
}
