use std::ops::{Add, AddAssign};

/// A [`Size`] represents calculated widths and heights for a component.
///
/// A [`Size`] is sent from a child component back up to its parents after
/// first being given a [`Bounds`](super::Bounds) from the parent.
#[derive(Clone, Copy, Default)]
pub struct Size {
    /// The width that the component has determined.
    pub width: u16,

    /// The height that the component has determined.
    pub height: u16,
}

impl Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}
