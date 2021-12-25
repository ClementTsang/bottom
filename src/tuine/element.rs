use enum_dispatch::enum_dispatch;
use tui::Frame;

use super::{
    Block, Bounds, Carousel, Container, DrawContext, Empty, Event, Flex, LayoutNode, Shortcut,
    Size, StateContext, Status, TempTable, TextTable, TmpComponent,
};

/// An [`Element`] is an instantiated [`Component`].
#[enum_dispatch(TmpComponent<Message>)]
pub enum Element<'a, Message, C = Empty>
where
    C: TmpComponent<Message>,
{
    Block,
    Carousel,
    Container(Container<'a, Message>),
    Flex(Flex<'a, Message>),
    Shortcut(Shortcut<Message, C>),
    TextTable(TextTable<'a, Message>),
    Empty,
    TempTable(TempTable<'a, Message>),
}
