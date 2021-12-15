//! Code here is based on crochet's implementation - see
//! [here](https://github.com/raphlinus/crochet/blob/master/src/key.rs) for more details!

use std::hash::Hash;
use std::panic::Location;

/// A newtype around [`Location`].
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Caller(&'static Location<'static>);

/// A unique key built around using the [`Location`] given by
/// `#[track_caller]` and a sequence index.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub(crate) caller: Caller,
    pub(crate) index: usize,
}

impl Key {
    pub fn new(caller: impl Into<Caller>, index: usize) -> Self {
        Self {
            caller: caller.into(),
            index,
        }
    }
}
