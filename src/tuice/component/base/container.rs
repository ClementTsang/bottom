use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Bounds, Component, DrawContext, Event, Length, Size, Status, LayoutNode};

pub struct Container<'a, Message, B>
where
    B: Backend,
{
    width: Length,
    height: Length,
    child: Box<dyn Component<Message, B> + 'a>,
}

impl<'a, Message, B> Container<'a, Message, B>
where
    B: Backend,
{
    pub fn new(child: Box<dyn Component<Message, B> + 'a>) -> Self {
        Self {
            width: Length::Flex,
            height: Length::Flex,
            child,
        }
    }
}

impl<'a, Message, B> Component<Message, B> for Container<'a, Message, B>
where
    B: Backend,
{
    fn draw(&mut self, area: Rect, _context: &DrawContext, _frame: &mut Frame<'_, B>) {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        todo!()
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        let width = match self.width {
            Length::Flex => {
                todo!()
            }
            Length::FlexRatio(ratio) => {
                todo!()
            }
            Length::Fixed(length) => length.clamp(bounds.min_width, bounds.max_width),
            Length::Child => {
                todo!()
            }
        };

        let height = match self.height {
            Length::Flex => {
                todo!()
            }
            Length::FlexRatio(ratio) => {
                todo!()
            }
            Length::Fixed(length) => length.clamp(bounds.min_height, bounds.max_height),
            Length::Child => {
                todo!()
            }
        };

        Size { height, width }
    }
}
