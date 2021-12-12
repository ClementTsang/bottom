use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Bounds, Element, Event, LayoutNode, Size, Status, TmpComponent};

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

    pub(crate) fn draw<B>(&mut self, area: Rect, frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        self.element.draw(area, frame)
    }

    pub(crate) fn on_event(
        &mut self, area: Rect, event: Event, messages: &mut Vec<Message>,
    ) -> Status {
        self.element.on_event(area, event, messages)
    }

    pub(crate) fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        todo!()
    }
}

impl<'a, Message> From<Element<'a, Message>> for FlexElement<'a, Message> {
    fn from(element: Element<'a, Message>) -> Self {
        Self { flex: 0, element }
    }
}
