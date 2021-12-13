use tui::layout::Rect;

use super::LayoutNode;

pub struct DrawContext<'a> {
    current_node: &'a LayoutNode,
    current_offset: (u16, u16),
}

impl<'a> DrawContext<'_> {
    pub(crate) fn new() {}

    pub(crate) fn rect(&self) -> Rect {
        self.current_node.rect
    }

    pub(crate) fn children(&self) -> impl Iterator<Item = DrawContext<'_>> {
        let new_offset = (
            self.current_offset.0 + self.current_node.rect.x,
            self.current_offset.1 + self.current_node.rect.y,
        );

        self.current_node
            .children
            .iter()
            .map(move |layout_node| DrawContext {
                current_node: layout_node,
                current_offset: new_offset,
            })
    }
}
