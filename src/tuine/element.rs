use enum_dispatch::enum_dispatch;
use tui::Frame;

use super::{
    Block, Bounds, Carousel, Container, DrawContext, Event, Flex, LayoutNode, Shortcut, Size,
    StateContext, Status, TextTable, TmpComponent,
};

/// An [`Element`] is an instantiated [`Component`].
#[enum_dispatch(TmpComponent<Message>)]
pub enum Element<'a, Message> {
    Block,
    Carousel,
    Container(Container<'a, Message>),
    Flex(Flex<'a, Message>),
    Shortcut(Shortcut<'a, Message>),
    TextTable(TextTable<'a, Message>),
}
