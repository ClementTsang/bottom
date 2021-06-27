use crossterm::event::{KeyEvent, MouseEvent};

/// Represents whether the event has been handled by a [`Widget`] or not.
#[derive(Debug)]
pub enum EventStatus {
    Handled,
    Ignored,
}

/// An [`Event`] represents an event.  TODO: Move this out.
#[derive(Debug, Clone)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyEvent),
}
