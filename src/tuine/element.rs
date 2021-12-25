use enum_dispatch::enum_dispatch;
use tui::Frame;

use super::{
    Block, Bounds, Carousel, Container, DrawContext, Empty, Event, Flex, LayoutNode, Shortcut,
    Size, StateContext, Status, TempTable, TextTable, TmpComponent,
};

/// An [`Element`] is an instantiated [`Component`].
#[enum_dispatch(TmpComponent<Message>)]
pub enum Element<Message, C = Empty>
where
    C: TmpComponent<Message>,
{
    Block,
    Carousel,
    Container(Container<Message>),
    Flex(Flex<Message>),
    Shortcut(Shortcut<Message, C>),
    TextTable(TextTable<Message>),
    Empty,
    TempTable(TempTable<Message>),
}
