use indextree::NodeId;
use tui::layout::Rect;

use crate::app::Component;

/// A container that "holds"" multiple [`BottomWidget`]s through their [`NodeId`]s.
pub struct Carousel {
    index: usize,
    children: Vec<NodeId>,
    bounds: Rect,
}

impl Carousel {
    /// Creates a new [`Carousel`] with the specified children.
    pub fn new(children: Vec<NodeId>) -> Self {
        Self {
            index: 0,
            children,
            bounds: Rect::default(),
        }
    }

    /// Adds a new child to a [`Carousel`].
    pub fn add_child(&mut self, child: NodeId) {
        self.children.push(child);
    }

    /// Returns the currently selected [`NodeId`] if possible.
    pub fn get_currently_selected(&self) -> Option<&NodeId> {
        self.children.get(self.index)
    }
}

impl Component for Carousel {
    fn bounds(&self) -> tui::layout::Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: tui::layout::Rect) {
        self.bounds = new_bounds;
    }
}
