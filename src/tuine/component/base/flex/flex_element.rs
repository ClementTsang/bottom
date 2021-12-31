use tui::{backend::Backend, Frame};

use crate::tuine::{
    Bounds, DrawContext, Element, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

use super::Axis;

pub struct FlexElement<Message> {
    /// Represents a ratio with other [`FlexElement`]s on how far to expand.
    pub flex: u16,
    element: Element<Message>,
}

impl<Message> FlexElement<Message> {
    /// Creates a new [`FlexElement`] with a flex of 1.
    pub fn new<E: Into<Element<Message>>>(element: E) -> Self {
        Self {
            flex: 1,
            element: element.into(),
        }
    }

    pub fn with_flex<E: Into<Element<Message>>>(element: E, flex: u16) -> Self {
        Self {
            flex,
            element: element.into(),
        }
    }

    pub fn with_no_flex<E: Into<Element<Message>>>(element: E) -> Self {
        Self {
            flex: 0,
            element: element.into(),
        }
    }

    pub fn flex(mut self, flex: u16) -> Self {
        self.flex = flex;
        self
    }

    pub(crate) fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        self.element.draw(state_ctx, draw_ctx, frame)
    }

    pub(crate) fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        self.element.on_event(state_ctx, draw_ctx, event, messages)
    }

    /// Assumes the flex is 0. Just calls layout on its child.
    pub(crate) fn child_layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        self.element.layout(bounds, node)
    }

    /// Assumes the flex is NOT 0. Will call layout on its children, but will ignore
    /// its sizing on the given axis.
    ///
    /// **Note it does NOT check for div by zero!** Please check this yourself.
    pub(crate) fn ratio_layout(
        &self, bounds: Bounds, total_flex: u16, node: &mut LayoutNode, parent_alignment: Axis,
    ) -> Size {
        let (min_width, min_height, max_width, max_height) = match parent_alignment {
            Axis::Horizontal => {
                let w = bounds.max_width * self.flex / total_flex;
                (w, 0, w, bounds.max_height)
            }
            Axis::Vertical => {
                let h = bounds.max_height * self.flex / total_flex;
                (0, h, bounds.max_width, h)
            }
        };

        let ratio_res = self.element.layout(
            Bounds {
                min_width,
                min_height,
                max_width,
                max_height,
            },
            node,
        );

        ratio_res
    }
}

impl<Message> From<Element<Message>> for FlexElement<Message> {
    fn from(element: Element<Message>) -> Self {
        Self { flex: 0, element }
    }
}
