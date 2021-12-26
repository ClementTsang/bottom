use rustc_hash::FxHashMap;

use crate::tuine::{Key, State};

#[derive(Default)]
pub struct StateMap(FxHashMap<Key, Box<dyn State>>);

impl StateMap {
    pub fn state<S: State + Default + 'static>(&mut self, key: Key) -> &S {
        let state = self.0.entry(key).or_insert_with(|| Box::new(S::default()));

        state
            .as_any()
            .downcast_ref()
            .expect("Successful downcast of state.")
    }

    pub fn mut_state<S: State + Default + 'static>(&mut self, key: Key) -> &mut S {
        let state = self.0.entry(key).or_insert_with(|| Box::new(S::default()));

        state
            .as_mut_any()
            .downcast_mut()
            .expect("Successful downcast of state.")
    }

    pub fn state_with_default<S: State + 'static, F: FnOnce() -> S>(
        &mut self, key: Key, default: F,
    ) -> &S {
        let state = self.0.entry(key).or_insert_with(|| Box::new(default()));

        state
            .as_any()
            .downcast_ref()
            .expect("Successful downcast of state.")
    }

    pub fn mut_state_with_default<S: State + 'static, F: FnOnce() -> S>(
        &mut self, key: Key, default: F,
    ) -> &mut S {
        let state = self.0.entry(key).or_insert_with(|| Box::new(default()));

        state
            .as_mut_any()
            .downcast_mut()
            .expect("Successful downcast of state.")
    }
}
