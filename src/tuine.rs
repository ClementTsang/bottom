//! tuine helps "tie" together ratatui/tui-rs and some layout/event logic to abstract away a bunch of the logic.

mod constraints;
mod container;
mod element;
mod widget;

pub use container::*;
pub use element::*;
pub use widget::*;

use crate::app::layout_manager::BottomLayout;

/// The overall widget tree.
///
/// TODO: The current implementation is a bit WIP while I transition things over.
pub struct WidgetTree {
    root: Element,
}

impl WidgetTree {
    /// Create a [`WidgetTree`].
    ///
    /// TODO: The current implementation is a bit WIP while I transition things over.
    pub fn new(layout: BottomLayout) -> Self {

        
        Self { root: todo!() }
    }

    /// Draw the widget tree.
    pub fn draw(&mut self) {}
}
