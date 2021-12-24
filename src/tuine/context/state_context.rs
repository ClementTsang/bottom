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
}
