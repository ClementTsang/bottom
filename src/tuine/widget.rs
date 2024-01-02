use tui::{layout::Rect, Frame};

/// A [`Widget`] converts raw data into something that a user can see and interact with.
pub trait Widget<Data> {
    /// How to actually draw the widget to the terminal.
    fn draw(&self, f: &mut Frame<'_>, draw_location: Rect, widget_id: u64);
}
