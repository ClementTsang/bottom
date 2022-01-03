use crate::tuine::{Caller, Key, State, StateMap};

use super::StateContext;

pub struct BuildContext<'a> {
    key_counter: usize,
    state_context: StateContext<'a>,
}

impl<'a> BuildContext<'a> {
    pub fn new(state_map: &'a mut StateMap) -> Self {
        Self {
            key_counter: 0,
            state_context: StateContext::new(state_map),
        }
    }

    pub fn register_component<C: Into<Caller>>(&mut self, caller: C) -> Key {
        self.key_counter += 1;
        Key::new(caller.into(), self.key_counter)
    }

    pub fn state<S: State + Default + 'static>(&mut self, key: Key) -> &S {
        self.state_context.state(key)
    }

    pub fn mut_state<S: State + Default + 'static>(&mut self, key: Key) -> &mut S {
        self.state_context.mut_state(key)
    }

    pub fn register_and_state<C: Into<Caller>, S: State + Default + 'static>(
        &mut self, caller: C,
    ) -> (Key, &S) {
        self.key_counter += 1;
        let key = Key::new(caller.into(), self.key_counter);

        (key, self.state(key))
    }

    pub fn register_and_mut_state<C: Into<Caller>, S: State + Default + 'static>(
        &mut self, caller: C,
    ) -> (Key, &mut S) {
        self.key_counter += 1;
        let key = Key::new(caller.into(), self.key_counter);

        (key, self.mut_state(key))
    }

    pub fn register_and_state_with_default<
        C: Into<Caller>,
        S: State + 'static,
        F: FnOnce() -> S,
    >(
        &mut self, caller: C, default: F,
    ) -> (Key, &S) {
        self.key_counter += 1;
        let key = Key::new(caller.into(), self.key_counter);

        (key, self.state_context.state_with_default(key, default))
    }

    pub fn register_and_mut_state_with_default<
        C: Into<Caller>,
        S: State + 'static,
        F: FnOnce() -> S,
    >(
        &mut self, caller: C, default: F,
    ) -> (Key, &mut S) {
        self.key_counter += 1;
        let key = Key::new(caller.into(), self.key_counter);

        (key, self.state_context.mut_state_with_default(key, default))
    }
}
