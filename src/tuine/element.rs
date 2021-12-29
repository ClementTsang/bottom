use enum_dispatch::enum_dispatch;
use tui::Frame;

use super::*;

/// An [`Element`] is an instantiated [`Component`].
#[enum_dispatch(TmpComponent<Message>)]
pub enum Element<Message, C = Empty>
where
    C: TmpComponent<Message>,
{
    Block(Block<Message, C>),
    Carousel,
    Container(Container<Message>),
    Flex(Flex<Message>),
    Shortcut(Shortcut<Message, C>),
    TextTable(TextTable<Message>),
    Empty,
    BatteryTable(BatteryTable),
    CpuGraph(CpuGraph),
    CpuSimple(CpuSimple),
    DiskTable(DiskTable),
    MemGraph(MemGraph),
    MemSimple(MemSimple),
    NetGraph(NetGraph),
    NetSimple(NetSimple),
    ProcessTable(ProcessTable),
    SimpleTable(SimpleTable<Message>),
    TempTable(TempTable<Message>),
}
