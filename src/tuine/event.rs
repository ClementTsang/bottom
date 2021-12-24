use crossterm::event::{KeyEvent, MouseEvent};
use tui::layout::Rect;

/// The status of an event after it has been handled.
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Captured,
    Ignored,
}

/// An [`Event`] represents some sort of user interface event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// A keyboard event
    Keyboard(KeyEvent),

    /// A mouse event
    Mouse(MouseEvent),
}

pub trait MouseBoundIntersect {
    /// Returns whether an event intersects some bounds.
    fn does_mouse_intersect_bounds(&self, bounds: Rect) -> bool;

    fn does_intersect(bounds: Rect, x: u16, y: u16) -> bool {
        x >= bounds.left() && x < bounds.right() && y >= bounds.top() && y < bounds.bottom()
    }
}

impl MouseBoundIntersect for MouseEvent {
    fn does_mouse_intersect_bounds(&self, bounds: Rect) -> bool {
        let x = self.column;
        let y = self.row;

        Self::does_intersect(bounds, x, y)
    }
}
