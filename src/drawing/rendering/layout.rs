use tui::layout::Rect;

/// A [`Node`] corresponds to a [`Widget`] in the hierarchy.
pub struct Node {
    bounds: Rect,
    children: Vec<Node>,
}

impl Node {
    pub fn new(bounds: Rect, children: Vec<Node>) -> Self {
        Self { bounds, children }
    }

    /// Returns the children of a [`Node`].
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Returns the bound of the [`Node`]
    pub fn bounds(&self) -> Rect {
        self.bounds
    }
}
