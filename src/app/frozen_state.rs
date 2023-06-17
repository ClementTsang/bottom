use super::DataCollection;

/// The [`FrozenState`] indicates whether the application state should be frozen. It is either not frozen or
/// frozen and containing a copy of the state at the time.
pub enum FrozenState {
    NotFrozen,
    Frozen(Box<DataCollection>),
}

impl Default for FrozenState {
    fn default() -> Self {
        Self::NotFrozen
    }
}

pub type IsFrozen = bool;

impl FrozenState {
    /// Checks whether the [`FrozenState`] is currently frozen.
    pub fn is_frozen(&self) -> IsFrozen {
        matches!(self, FrozenState::Frozen(_))
    }

    /// Freezes the [`FrozenState`].
    pub fn freeze(&mut self, data: Box<DataCollection>) {
        *self = FrozenState::Frozen(data);
    }

    /// Unfreezes the [`FrozenState`].
    pub fn thaw(&mut self) {
        *self = FrozenState::NotFrozen;
    }

    /// Toggles the [`FrozenState`] and returns whether it is now frozen.
    pub fn toggle(&mut self, data: &DataCollection) -> IsFrozen {
        if self.is_frozen() {
            self.thaw();
            false
        } else {
            // Could we use an Arc instead? Is it worth it?
            self.freeze(Box::new(data.clone()));
            true
        }
    }
}
