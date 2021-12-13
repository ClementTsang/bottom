use enum_dispatch::enum_dispatch;
use tui::{layout::Rect, Frame};

use super::{
    Block, Bounds, Carousel, Container, DrawContext, Event, Flex, LayoutNode, Shortcut, Size,
    Status, TextTable, TmpComponent,
};

/// An [`Element`] is an instantiated [`Component`].
#[enum_dispatch(TmpComponent<Message>)]
pub enum Element<'a, Message> {
    Block,
    Carousel,
    Container(Container<'a, Message>),
    Flex(Flex<'a, Message>),
    Shortcut,
    TextTable(TextTable<'a, Message>),
}
