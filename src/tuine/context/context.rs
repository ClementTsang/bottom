use gapbuffer::GapBuffer;

use crate::tuine::{Key, KeyCreator, State};

/// A [`Context`] is used to create a [`Component`](super::Component).
///
/// The internal implementation is based on Jetpack Compose's [Positional Memoization](https://medium.com/androiddevelopers/under-the-hood-of-jetpack-compose-part-2-of-2-37b2c20c6cdd),
/// in addition to [Crochet](https://github.com/raphlinus/crochet/blob/master/src/tree.rs) in its entirety.
pub struct Context {
    component_key_creator: KeyCreator,
    buffer: GapBuffer<Slot>,
}

enum Payload {
    State(Box<dyn State>),
    View,
}

struct Item {
    key: Key,
    payload: Payload,
}

enum Slot {
    Begin(Item),
    End,
}

impl Context {
    pub fn use_state(&self) {}

    pub fn start(&mut self) {}

    pub fn end(&mut self) {}
}
