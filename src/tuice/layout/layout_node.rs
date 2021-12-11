use tui::layout::Rect;

#[derive(Default)]
pub struct LayoutNode {
    pub area: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn from_area(area: Rect) -> Self {
        Self {
            area,
            children: vec![],
        }
    }
}
