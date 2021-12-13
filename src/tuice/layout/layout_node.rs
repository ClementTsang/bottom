use tui::layout::Rect;

#[derive(Clone, Default)]
pub struct LayoutNode {
    pub rect: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn from_rect(rect: Rect) -> Self {
        Self {
            rect,
            children: vec![],
        }
    }
}
