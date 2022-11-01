//! fxhash but with a random start state. See the following links for the rationale on why this is needed:
//! - https://morestina.net/blog/1843/the-stable-hashmap-trap (changes are based on the provided workaround)
//! - https://accidentallyquadratic.tumblr.com/post/153545455987/rust-hash-iteration-reinsertion
//! - https://github.com/cbreeden/fxhash/issues/15

use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hasher},
};

use fxhash::FxHasher;
use rand::Rng;

pub type RfxHashMap<K, V> = HashMap<K, V, FxRandomState>;
pub type RfxHashSet<V> = HashSet<V, FxRandomState>;

#[derive(Copy, Clone, Debug)]
pub struct FxRandomState(usize);

impl BuildHasher for FxRandomState {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> FxHasher {
        let mut hasher = FxHasher::default();
        hasher.write_usize(self.0);
        hasher
    }
}

impl Default for FxRandomState {
    fn default() -> Self {
        thread_local! {
            static SEED: Cell<usize> = Cell::new(rand::thread_rng().gen())
        }
        let seed = SEED.with(|seed| {
            let n = seed.get();
            seed.set(n.wrapping_add(1));
            n
        });
        FxRandomState(seed)
    }
}
