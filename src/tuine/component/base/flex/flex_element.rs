use tui::{backend::Backend, Frame};

use crate::tuine::{
    Bounds, DrawContext, Element, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

use super::Axis;

pub struct FlexElement<'a, Message> {
    /// Represents a ratio with other [`FlexElement`]s on how far to expand.
    pub flex: u16,
    element: Element<'a, Message>,
}

impl<'a, Message> FlexElement<'a, Message> {
    pub fn new<I: Into<Element<'a, Message>>>(element: I) -> Self {
        Self {
            flex: 1,
            element: element.into(),
        }
    }

    pub fn with_flex<I: Into<Element<'a, Message>>>(element: I, flex: u16) -> Self {
        Self {
            flex,
            element: element.into(),
        }
    }

    pub fn with_no_flex<I: Into<Element<'a, Message>>>(element: I) -> Self {
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
    /// its sizing.
    ///
    /// **Note it does NOT check for div by zero!** Please check this yourself.
    pub(crate) fn ratio_layout(
        &self, bounds: Bounds, total_flex: u16, node: &mut LayoutNode, parent_alignment: Axis,
    ) -> Size {
        let (width, height) = match parent_alignment {
            Axis::Horizontal => (bounds.max_width * self.flex / total_flex, bounds.max_height),
            Axis::Vertical => (bounds.max_width, bounds.max_height * self.flex / total_flex),
        };

        self.element.layout(
            Bounds {
                min_width: width,
                min_height: height,
                max_width: width,
                max_height: height,
            },
            node,
        );

        Size { width, height }
    }
}

impl<'a, Message> From<Element<'a, Message>> for FlexElement<'a, Message> {
    fn from(element: Element<'a, Message>) -> Self {
        Self { flex: 0, element }
    }
}
