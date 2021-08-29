use std::time::{Duration, Instant};

const MAX_TIMEOUT: Duration = Duration::from_millis(400);

/// These are "signals" that are sent along with an [`WidgetEventResult`] to signify a potential additional action
/// that the caller must do, along with the "core" result of either drawing or redrawing.
pub enum ReturnSignal {
    /// A signal returned when some process widget was told to try to kill a process (or group of processes).
    ///
    /// This return signal should trigger a redraw when handled.
    KillProcess,

    /// A signal returned when a widget needs the app state to re-trigger its update call. Usually needed for
    /// widgets where the displayed contents are built only on update.
    ///
    /// This return signal should trigger a redraw when handled.
    Update,
}

/// The results of handling an event by the [`AppState`].
pub enum EventResult {
    /// Kill the program.
    Quit,
    /// Trigger a redraw.
    Redraw,
    /// Don't trigger a redraw.
    NoRedraw,
}

/// The results of a widget handling some event, like a mouse or key event,
/// signifying what the program should then do next.
pub enum WidgetEventResult {
    /// Kill the program.
    Quit,
    /// Trigger a redraw.
    Redraw,
    /// Don't trigger a redraw.
    NoRedraw,
    /// Return a signal.
    Signal(ReturnSignal),
}

/// How a widget should handle a widget selection request.
pub enum SelectionAction {
    /// This event occurs if the widget internally handled the selection action.
    Handled,
    /// This event occurs if the widget did not handle the selection action; the caller must handle it.
    NotHandled,
}

/// The states a [`MultiKey`] can be in.
enum MultiKeyState {
    /// Currently not waiting on any next input.
    Idle,
    /// Waiting for the next input, with a given trigger [`Instant`].
    Waiting {
        /// When it was triggered.
        trigger_instant: Instant,

        /// What part of the pattern it is at.
        checked_index: usize,
    },
}

/// The possible outcomes of calling [`MultiKey::input`].
pub enum MultiKeyResult {
    /// Returned when a character was *accepted*, but has not completed the sequence required.
    Accepted,
    /// Returned when a character is accepted and completes the sequence.
    Completed,
    /// Returned if a character breaks the sequence or if it has already timed out.
    Rejected,
}

/// A struct useful for managing multi-key keybinds.
pub struct MultiKey {
    state: MultiKeyState,
    pattern: Vec<char>,
}

impl MultiKey {
    /// Creates a new [`MultiKey`] with a given pattern and timeout.
    pub fn register(pattern: Vec<char>) -> Self {
        Self {
            state: MultiKeyState::Idle,
            pattern,
        }
    }

    /// Resets to an idle state.
    fn reset(&mut self) {
        self.state = MultiKeyState::Idle;
    }

    /// Handles a char input and returns the current status of the [`MultiKey`] after, which is one of:
    /// - Accepting the char and moving to the next state
    /// - Completing the multi-key pattern
    /// - Rejecting it
    ///
    /// Note that if a [`MultiKey`] only "times out" upon calling this - if it has timed out, it will first reset
    /// before trying to check the char.
    pub fn input(&mut self, c: char) -> MultiKeyResult {
        match &mut self.state {
            MultiKeyState::Idle => {
                if let Some(first) = self.pattern.first() {
                    if *first == c {
                        self.state = MultiKeyState::Waiting {
                            trigger_instant: Instant::now(),
                            checked_index: 0,
                        };

                        return MultiKeyResult::Accepted;
                    }
                }

                MultiKeyResult::Rejected
            }
            MultiKeyState::Waiting {
                trigger_instant,
                checked_index,
            } => {
                if trigger_instant.elapsed() > MAX_TIMEOUT {
                    // Just reset and recursively call (putting it into Idle).
                    self.reset();
                    self.input(c)
                } else if let Some(next) = self.pattern.get(*checked_index + 1) {
                    if *next == c {
                        *checked_index += 1;

                        if *checked_index == self.pattern.len() - 1 {
                            self.reset();
                            MultiKeyResult::Completed
                        } else {
                            MultiKeyResult::Accepted
                        }
                    } else {
                        self.reset();
                        MultiKeyResult::Rejected
                    }
                } else {
                    self.reset();
                    MultiKeyResult::Rejected
                }
            }
        }
    }
}
