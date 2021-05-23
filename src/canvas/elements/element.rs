#![allow(dead_code)]

use tui::{backend::Backend, layout::Rect, Frame};

use crate::{app::AppState, canvas::canvas_colours::CanvasColours};

/// A single point.
#[derive(Copy, Clone)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

/// The top-left and bottom-right corners of a [`Element`].
#[derive(Copy, Clone)]
pub enum ElementBounds {
    Unset,
    Points {
        top_left_corner: Point,
        bottom_right_corner: Point,
    },
}

/// A basic [`Element`] trait, all drawn components must implement this.
pub trait Element {
    /// The type of data that is expected for the [`Element`].

    /// The main drawing function.
    fn draw<B: Backend>(
        &mut self, f: &mut Frame<'_, B>, app_state: &AppState, draw_loc: Rect,
        style: &CanvasColours,
    ) -> anyhow::Result<()>;

    /// Recalculates the click bounds.
    fn recalculate_click_bounds(&mut self);

    /// A function to determine the main widget click bounds.
    fn click_bounds(&self) -> ElementBounds;

    /// Returns whether am [`Element`] is selected.
    fn is_selected(&self) -> bool;

    /// Marks an [`Element`] as selected.
    fn select(&mut self);

    /// Marks an [`Element`] as unselected.
    fn unselect(&mut self);
}
