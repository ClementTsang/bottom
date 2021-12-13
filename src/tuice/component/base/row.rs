use itertools::izip;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{
    Bounds, DrawContext, Element, Event, FlexElement, LayoutNode, Size, Status, TmpComponent,
};

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

    pub fn with_child<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(FlexElement::with_no_flex(child.into()));
        self
    }

    pub fn with_flex_child<E>(mut self, child: E, flex: u16) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children
            .push(FlexElement::with_flex(child.into(), flex));
        self
    }
}

impl<'a, Message> TmpComponent<Message> for Row<'a, Message> {
    fn draw<B>(&mut self, context: DrawContext<'_>, frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        self.children
            .iter_mut()
            .zip(context.children())
            .for_each(|(child, child_node)| {
                child.draw(child_node, frame);
            });
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        let mut remaining_bounds = bounds;
        let mut children = vec![LayoutNode::default(); self.children.len()];
        let mut inflexible_children_indexes = vec![];
        let mut offsets = vec![];
        let mut current_x = 0;
        let mut current_y = 0;
        let mut sizes = Vec::with_capacity(self.children.len());
        let mut current_size = Size::default();

        let mut get_child_size = |child: &FlexElement<'_, Message>,
                                  child_node: &mut LayoutNode,
                                  remaining_bounds: &mut Bounds| {
            let size = child.layout(*remaining_bounds, child_node);
            current_size += size;
            remaining_bounds.shrink_size(size);
            offsets.push((current_x, current_y));
            current_x += size.width;
            current_y += size.height;

            size
        };

        // We handle inflexible children first, then distribute all remaining
        // space to flexible children.
        self.children
            .iter()
            .zip(children.iter_mut())
            .enumerate()
            .for_each(|(index, (child, child_node))| {
                if child.flex == 0 && remaining_bounds.has_space() {
                    let size = get_child_size(child, child_node, &mut remaining_bounds);
                    sizes.push(size);
                } else {
                    inflexible_children_indexes.push(index);
                    sizes.push(Size::default());
                }
            });

        inflexible_children_indexes.into_iter().for_each(|index| {
            // The index accesses are safe by above definitions, so we can use unsafe operations.
            // If you EVER make changes to above, ensure this invariant still holds!
            let child = unsafe { self.children.get_unchecked(index) };
            let child_node = unsafe { children.get_unchecked_mut(index) };
            let size = unsafe { sizes.get_unchecked_mut(index) };

            *size = get_child_size(child, child_node, &mut remaining_bounds);
        });

        // If there is still remaining space after, distribute the rest if
        // appropriate (e.x. current_size is too small for the bounds).
        if current_size.width < bounds.min_width {
            // For now, we'll cheat and just set it to be equal.
            current_size.width = bounds.min_width;
        }
        if current_size.height < bounds.min_height {
            // For now, we'll cheat and just set it to be equal.
            current_size.height = bounds.min_height;
        }

        // Now that we're done determining sizes, convert all children into the appropriate
        // layout nodes.  Remember - parents determine children, and so, we determine
        // children here!
        izip!(sizes, offsets, children.iter_mut()).for_each(
            |(size, offset, child): (Size, (u16, u16), &mut LayoutNode)| {
                let rect = Rect::new(offset.0, offset.1, size.width, size.height);
                child.rect = rect;
            },
        );
        node.children = children;

        current_size
    }
}
