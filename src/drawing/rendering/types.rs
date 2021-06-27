//! Some common types.

use tui::widgets::Borders;

/// An [`Axis`] represents either a horizontal (left and right) or vertical (up and down) direction.
#[derive(Debug, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// A [`Border`] represents either a disabled border or an enabled border, in which case a bitflag is expected
/// to determine which borders to show.
#[derive(Debug, Hash)]
pub enum Border {
    Disabled,
    Enabled(Borders),
}

/// [`VerticalScrollDirection`] represents either up or down.
#[derive(Debug, Hash)]
pub enum VerticalScrollDirection {
    /// Scrolling [`Up`] usually *decrements* the index.
    Up,
    /// Scrolling [`Down`] usually *increments* the index.
    Down,
}

/// [`Padding`] represents padding.
#[derive(Debug, Hash)]
pub enum Padding {
    Disabled,
    Left(u16),
    Right(u16),
    Up(u16),
    Down(u16),
    Horizontal(u16),
    Vertical(u16),
    All(u16),
}

/// A point in a graph.
pub type Point = (f64, f64);
