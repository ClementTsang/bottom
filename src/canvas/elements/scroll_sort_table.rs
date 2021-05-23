#![allow(dead_code)]

//! Code for a generic table element with scroll and sort support.

use crate::app::{AppScrollWidgetState, CanvasTableWidthState};

use super::element::ElementBounds;

#[derive(Debug, Default)]
pub struct State {
    scroll: AppScrollWidgetState,
    width: CanvasTableWidthState,
}

/// A [`ScrollSortTable`] is a stateful generic table element with scroll and sort support.
pub struct ScrollSortTable {
    state: State,
    bounds: ElementBounds,
}

impl ScrollSortTable {
    /// Function for incrementing the scroll.
    fn increment_scroll(&mut self) {}

    /// Function for decrementing the scroll.
    fn decrement_scroll(&mut self) {}

    pub fn on_down(&mut self) {
        self.increment_scroll();
    }

    pub fn on_up(&mut self) {
        self.decrement_scroll();
    }
}
