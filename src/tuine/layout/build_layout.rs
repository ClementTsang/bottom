use tui::layout::Rect;

use crate::tuine::{Bounds, Element, LayoutNode, TmpComponent};

pub fn build_layout_tree<Message>(rect: Rect, root: &Element<Message>) -> LayoutNode {
    let mut root_layout_node = LayoutNode::from_rect(rect);
    let bounds = Bounds {
        min_width: 0,
        min_height: 0,
        max_width: rect.width,
        max_height: rect.height,
    };

    let _ = root.layout(bounds, &mut root_layout_node);

    root_layout_node
}
