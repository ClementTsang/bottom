use tui::{backend::Backend, layout::Rect};

use crate::tuice::{Bounds, Element, LayoutNode};

pub fn build_layout_tree<Message, B: Backend>(
    rect: Rect, root: &Element<'_, Message, B>,
) -> LayoutNode {
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
