use crate::tuine::{Caller, Key, State, StateMap};

use super::StateContext;

pub struct ViewContext<'a> {
    key_counter: usize,
    state_context: StateContext<'a>,
}

impl<'a> ViewContext<'a> {
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
}
