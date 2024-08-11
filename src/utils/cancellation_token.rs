use std::{
    sync::{Condvar, Mutex},
    time::Duration,
};

/// A cancellation token.
pub(crate) struct CancellationToken {
    // The "check" for the cancellation token. Setting this to true will mark the cancellation token as "cancelled".
    mutex: Mutex<bool>,
    cvar: Condvar,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self {
            mutex: Mutex::new(false),
            cvar: Condvar::new(),
        }
    }
}

impl CancellationToken {
    /// Mark the [`CancellationToken`] as cancelled.
    ///
    /// This is idempotent, and once cancelled, will stay cancelled. Sending it
    /// again will not do anything.
    pub fn cancel(&self) {
        let mut guard = self
            .mutex
            .lock()
            .expect("cancellation token lock should not be poisoned");

        if !*guard {
            *guard = true;
            self.cvar.notify_all();
        }
    }

    /// Try and check the [`CancellationToken`]'s status. Note that
    /// this will not block.
    pub fn try_check(&self) -> Option<bool> {
        self.mutex.try_lock().ok().map(|guard| *guard)
    }

    /// Allows a thread to sleep while still being interruptible with by the token.
    ///
    /// Returns the condition state after either sleeping or being woken up.
    pub fn sleep_with_cancellation(&self, duration: Duration) -> bool {
        let guard = self
            .mutex
            .lock()
            .expect("cancellation token lock should not be poisoned");

        let (result, _) = self
            .cvar
            .wait_timeout(guard, duration)
            .expect("cancellation token lock should not be poisoned");

        *result
    }
}
