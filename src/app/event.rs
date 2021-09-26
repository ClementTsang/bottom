pub mod multi_key;
pub use multi_key::*;

/// These are "signals" that are sent along with an [`WidgetEventResult`] to signify a potential additional action
/// that the caller must do, along with the "core" result of either drawing or redrawing.
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub enum ComponentEventResult {
    /// The event isn't handled by the widget, and should be propagated to the parent.
    Unhandled,
    /// Trigger a redraw.
    Redraw,
    /// Don't trigger a redraw.
    NoRedraw,
    /// Return a signal.
    Signal(ReturnSignal),
}

/// How a widget should handle a widget selection request.
pub enum SelectionAction {
    /// This occurs if the widget internally handled the selection action. A redraw is required.
    Handled,
    /// This occurs if the widget did not handle the selection action; the caller must handle it.
    NotHandled,
}
