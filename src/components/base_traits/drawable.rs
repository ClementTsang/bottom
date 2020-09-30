use tui::{backend::Backend, layout::Rect, Frame};

use crate::app::App;

pub trait Drawable {
    fn draw<B: Backend>(
        &mut self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, is_force_redraw: bool,
    );
}
