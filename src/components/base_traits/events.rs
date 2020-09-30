use crossterm::event::KeyEvent;

use super::Coordinate;

pub trait HandleKeyInputs {
    /// How to handle a key input
    fn on_char(&mut self, key_input: KeyEvent);
}

pub trait HandleScroll {
    fn on_scroll_up(&mut self);

    fn on_scroll_down(&mut self);
}

pub enum MouseButton {
    Left,
    Middle,
    Right,
}

pub trait HandleClick {
    /// How to handle a click.
    fn on_click(&mut self, button: MouseButton, click_coord: Coordinate);
}
