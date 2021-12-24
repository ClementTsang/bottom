use tui::{backend::Backend, layout::Rect, Frame};

pub mod flex_element;
pub use flex_element::FlexElement;

use crate::tuine::{
    Bounds, DrawContext, Element, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    /// Represents the x-axis.
    Horizontal,

    /// Represents the y-axis.
    Vertical,
}

pub struct Flex<'a, Message> {
    children: Vec<FlexElement<'a, Message>>,
    alignment: Axis,
}

impl<'a, Message> Flex<'a, Message> {
    pub fn new(alignment: Axis) -> Self {
        Self {
            children: vec![],
            alignment,
        }
    }

    /// Creates a new [`Flex`] with a horizontal alignment.
    pub fn row() -> Self {
        Self {
            children: vec![],
            alignment: Axis::Horizontal,
        }
    }

    /// Creates a new [`Flex`] with a horizontal alignment with the given children.
    pub fn row_with_children<C>(children: Vec<C>) -> Self
    where
        C: Into<FlexElement<'a, Message>>,
    {
        Self {
            children: children.into_iter().map(Into::into).collect(),
            alignment: Axis::Horizontal,
        }
    }

    /// Creates a new [`Flex`] with a vertical alignment.
    pub fn column() -> Self {
        Self {
            children: vec![],
            alignment: Axis::Vertical,
        }
    }

    /// Creates a new [`Flex`] with a vertical alignment with the given children.
    pub fn column_with_children<C>(children: Vec<C>) -> Self
    where
        C: Into<FlexElement<'a, Message>>,
    {
        Self {
            children: children.into_iter().map(Into::into).collect(),
            alignment: Axis::Vertical,
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

impl<'a, Message> TmpComponent<Message> for Flex<'a, Message> {
    fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        self.children
            .iter_mut()
            .zip(draw_ctx.children())
            .for_each(|(child, child_draw_ctx)| {
                if child_draw_ctx.should_draw() {
                    child.draw(state_ctx, &child_draw_ctx, frame);
                }
            });
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        for (child, child_draw_ctx) in self.children.iter_mut().zip(draw_ctx.children()) {
            match child.on_event(state_ctx, &child_draw_ctx, event, messages) {
                Status::Captured => {
                    return Status::Captured;
                }
                Status::Ignored => {}
            }
        }

        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        let mut remaining_bounds = bounds;
        let mut children = vec![LayoutNode::default(); self.children.len()];
        let mut flexible_children_indexes = vec![];
        let mut current_x_offset = 0;
        let mut current_y_offset = 0;
        let mut sizes = Vec::with_capacity(self.children.len());
        let mut current_size = Size::default();
        let mut total_flex = 0;

        // Our general approach is to first handle inflexible children first,
        // then distribute all remaining space to flexible children.
        self.children
            .iter()
            .zip(children.iter_mut())
            .enumerate()
            .for_each(|(index, (child, child_node))| {
                if child.flex == 0 {
                    let size = if remaining_bounds.has_space() {
                        let size = child.child_layout(remaining_bounds, child_node);
                        current_size += size;
                        remaining_bounds.shrink_size(size);

                        size
                    } else {
                        Size::default()
                    };

                    sizes.push(size);
                } else {
                    total_flex += child.flex;
                    flexible_children_indexes.push(index);
                    sizes.push(Size::default());
                }
            });

        flexible_children_indexes.into_iter().for_each(|index| {
            // The index accesses are assumed to be safe by above definitions.
            // This means that we can use the unsafe operations below.
            //
            // NB: If you **EVER** make changes in this function, ensure these assumptions
            // still hold!
            let child = unsafe { self.children.get_unchecked(index) };
            let child_node = unsafe { children.get_unchecked_mut(index) };
            let size = unsafe { sizes.get_unchecked_mut(index) };

            let new_size =
                child.ratio_layout(remaining_bounds, total_flex, child_node, self.alignment);
            current_size += new_size;
            *size = new_size;
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
        sizes
            .iter()
            .zip(children.iter_mut())
            .for_each(|(size, child)| {
                child.rect = Rect::new(current_x_offset, current_y_offset, size.width, size.height);

                match self.alignment {
                    Axis::Horizontal => {
                        current_x_offset += size.width;
                    }
                    Axis::Vertical => {
                        current_y_offset += size.height;
                    }
                }
            });
        node.children = children;

        current_size
    }
}
