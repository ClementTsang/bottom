use tui::layout::Rect;

/// A node for the layout tree.
pub enum LayoutNode {
    Leaf {
        area: Rect,
    },
    Branch {
        area: Rect,
        children: Vec<LayoutNode>,
    },
}
