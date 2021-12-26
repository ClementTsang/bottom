use tui::layout::Rect;

use crate::tuine::LayoutNode;

pub struct DrawContext<'a> {
    current_node: &'a LayoutNode,
    current_offset: (u16, u16),
}

impl<'a> DrawContext<'a> {
    /// Creates a new [`DrawContext`], with the offset set to `(0, 0)`.
    pub(crate) fn root(root: &'a LayoutNode) -> DrawContext<'a> {
        DrawContext {
            current_node: root,
            current_offset: (0, 0),
        }
    }

    pub(crate) fn global_rect(&self) -> Rect {
        let mut rect = self.current_node.rect;
        rect.x += self.current_offset.0;
        rect.y += self.current_offset.1;

        rect
    }

    pub(crate) fn should_draw(&self) -> bool {
        self.current_node.rect.area() != 0
    }

    pub(crate) fn children(&'a self) -> impl Iterator<Item = DrawContext<'_>> {
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
