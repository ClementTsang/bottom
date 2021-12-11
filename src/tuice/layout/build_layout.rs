use tui::layout::Rect;

use crate::tuice::{Bounds, Component, LayoutNode};

pub fn build_layout_tree<Message, Backend>(
    area: Rect, root: &Box<dyn Component<Message, Backend>>,
) -> LayoutNode
where
    Backend: tui::backend::Backend,
{
    let mut root_layout_node = LayoutNode::from_area(area);
    let bounds = Bounds {
        min_width: 0,
        min_height: 0,
        max_width: area.width,
        max_height: area.height,
    };

    root.layout(bounds, &mut root_layout_node);

    root_layout_node
}
