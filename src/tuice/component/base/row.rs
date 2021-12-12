use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Bounds, Event, FlexElement, LayoutNode, Size, Status, TmpComponent};

#[derive(Default)]
pub struct Row<'a, Message> {
    children: Vec<FlexElement<'a, Message>>,
}

impl<'a, Message> Row<'a, Message> {
    /// Creates a new [`Row`] with the given children.
    pub fn with_children<C>(children: Vec<C>) -> Self
    where
        C: Into<FlexElement<'a, Message>>,
    {
        Self {
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    pub fn with_child(mut self) -> Self {
        self
    }

    pub fn with_flex_child(mut self) -> Self {
        self
    }
}

impl<'a, Message> TmpComponent<Message> for Row<'a, Message> {
    fn draw<B>(&mut self, area: Rect, frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        self.children.iter_mut().for_each(|child| {
            child.draw(area, frame);
        })
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        let mut remaining_bounds = bounds;

        let child_nodes: Vec<LayoutNode> = self
            .children
            .iter()
            .map(|child| {
                let mut child_node = LayoutNode::default();
                let size = child.layout(remaining_bounds, &mut child_node);
                child_node
            })
            .collect();

        todo!()
    }
}
