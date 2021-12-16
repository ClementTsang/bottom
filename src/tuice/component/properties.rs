use enum_dispatch::enum_dispatch;

use crate::tuice::*;

/// A trait that the properties of a [`Component`](super::Component)
/// should implement.
#[enum_dispatch]
pub trait Properties: PartialEq + Clone {}

#[derive(PartialEq, Clone, Debug)]
pub struct DefaultProp;

impl Properties for DefaultProp {}

#[enum_dispatch(Properties)]
#[derive(PartialEq, Clone)]
pub enum Props {
    DefaultProp,
    TextTableProps,
}
