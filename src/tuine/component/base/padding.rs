use std::marker::PhantomData;

use crate::tuine::TmpComponent;

/// A [`Padding`] surrounds a child widget with spacing.
pub struct Padding<Child, Message>
where
    Child: TmpComponent<Message>,
{
    _pd: PhantomData<Message>,
    padding_left: u16,
    padding_right: u16,
    padding_up: u16,
    padding_down: u16,
    child: Option<Child>,
}
