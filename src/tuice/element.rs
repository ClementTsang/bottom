use enum_dispatch::enum_dispatch;
use tui::{layout::Rect, Frame};

use super::{
    Block, Bounds, Carousel, Column, Container, Event, LayoutNode, Row, Shortcut, Size, Status,
    TextTable, TmpComponent,
};

/// An [`Element`] is an instantiated [`Component`].
#[enum_dispatch(TmpComponent<Message>)]
pub enum Element<'a, Message> {
    Block,
    Carousel,
    Column,
    Container(Container<'a, Message>),
    Row(Row<'a, Message>),
    Shortcut,
    TextTable(TextTable<'a, Message>),
}
