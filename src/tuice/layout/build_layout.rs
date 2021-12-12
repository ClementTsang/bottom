use tui::layout::Rect;

use crate::tuice::{Bounds, Element, LayoutNode, TmpComponent};

pub fn build_layout_tree<Message>(area: Rect, root: &Element<'_, Message>) -> LayoutNode {
    let mut root_layout_node = LayoutNode::from_area(area);
    let bounds = Bounds {
        min_width: 0,
        min_height: 0,
        max_width: area.width,
        max_height: area.height,
    };

    let _ = root.layout(bounds, &mut root_layout_node);

    root_layout_node
}
