use rustc_hash::FxHashMap;

use crate::tuine::{Key, State};

#[derive(Default)]
pub struct StateMap(FxHashMap<Key, (Box<dyn State>, bool)>);

impl StateMap {
    pub fn state<S: State + Default + 'static>(&mut self, key: Key) -> &S {
        let state = self
            .0
            .entry(key)
            .or_insert_with(|| (Box::new(S::default()), true));

        state.1 = true;

        state
            .0
            .as_any()
            .downcast_ref()
            .expect("Successful downcast of state.")
    }

    pub fn mut_state<S: State + Default + 'static>(&mut self, key: Key) -> &mut S {
        let state = self
            .0
            .entry(key)
            .or_insert_with(|| (Box::new(S::default()), true));

        state.1 = true;

        state
            .0
            .as_mut_any()
            .downcast_mut()
            .expect("Successful downcast of state.")
    }
}
