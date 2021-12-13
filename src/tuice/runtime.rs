use std::sync::mpsc::Receiver;

use tui::layout::Rect;

use crate::tuice::Status;

use super::{Application, Event, TmpComponent};

#[derive(Clone, Copy, Debug)]
pub enum RuntimeEvent<Message> {
    UserInterface(Event),
    Resize { width: u16, height: u16 },
    Custom(Message),
}

pub(crate) fn launch<A: Application + 'static>(
    mut application: A, receiver: Receiver<RuntimeEvent<A::Message>>,
) {
    let mut user_interface = application.view();

    while !application.is_terminated() {
        if let Ok(event) = receiver.recv() {
            match event {
                RuntimeEvent::UserInterface(event) => {
                    let mut messages = vec![];

                    let rect = Rect::default(); // FIXME: TEMP
                    match user_interface.on_event(rect, event, &mut messages) {
                        Status::Captured => {}
                        Status::Ignored => {
                            application.global_event_handler(event, &mut messages);
                        }
                    }

                    for msg in messages {
                        debug!("Message: {:?}", msg); // FIXME: Remove this debug line!
                        application.update(msg);
                    }

                    user_interface = application.view();
                    // FIXME: Draw!
                }
                RuntimeEvent::Custom(message) => {
                    application.update(message);
                }
                RuntimeEvent::Resize {
                    width: _,
                    height: _,
                } => {
                    user_interface = application.view();
                }
            }
        } else {
            break;
        }
    }

    application.destroy();
}
