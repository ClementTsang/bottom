use std::{fmt::Debug, sync::mpsc::Receiver};

use tui::Terminal;

use super::{
    runtime::{self, RuntimeEvent},
    Element, Event,
};

/// An alias to the [`tui::backend::CrosstermBackend`] writing to [`std::io::Stdout`].
pub type CrosstermBackend = tui::backend::CrosstermBackend<std::io::Stdout>;

#[allow(unused_variables)]
pub trait Application: Sized {
    type Message: Debug;

    /// Determines how to handle a given message.
    fn update(&mut self, message: Self::Message);

    /// Returns whether to stop the application. Defaults to
    /// always returning false.
    fn is_terminated(&self) -> bool;

    fn view(&mut self) -> Element<'static, Self::Message>;

    /// To run upon stopping the application.
    fn destroy(&mut self) {}

    /// An optional event handler, intended for use with global shortcuts or events.
    /// This will be run *after* trying to send the events into the user interface, and
    /// *only* if it is not handled at all by it.
    ///
    /// Defaults to not doing anything.
    fn global_event_handler(&mut self, event: Event, messages: &mut Vec<Self::Message>) {}
}

/// Launches some application with tuice. Note this will take over the calling thread.
pub fn launch_with_application<A, B>(
    application: A, receiver: Receiver<RuntimeEvent<A::Message>>, terminal: &mut Terminal<B>,
) -> anyhow::Result<()>
where
    A: Application + 'static,
    B: tui::backend::Backend,
{
    runtime::launch(application, receiver, terminal)
}
