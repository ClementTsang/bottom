use crate::tuine::{Key, State, StateMap};

pub struct StateContext<'a> {
    state_map: &'a mut StateMap,
}

impl<'a> StateContext<'a> {
    pub fn new(state_map: &'a mut StateMap) -> Self {
        Self { state_map }
    }

    pub fn state<S: State + Default + 'static>(&mut self, key: Key) -> &S {
        self.state_map.state::<S>(key)
    }

    pub fn mut_state<S: State + Default + 'static>(&mut self, key: Key) -> &mut S {
        self.state_map.mut_state::<S>(key)
    }

    pub fn state_with_default<S: State + 'static, F: FnOnce() -> S>(
        &mut self, key: Key, default: F,
    ) -> &S {
        self.state_map.state_with_default::<S, F>(key, default)
    }

    pub fn mut_state_with_default<S: State + 'static, F: FnOnce() -> S>(
        &mut self, key: Key, default: F,
    ) -> &mut S {
        self.state_map.mut_state_with_default::<S, F>(key, default)
    }
}
