use std::sync::mpsc::Receiver;

use rustc_hash::FxHashMap;
use tui::{backend::Backend, layout::Rect, Terminal};

use crate::tuine::Status;

use super::{
    build_layout_tree, Application, Element, Event, Key, State, StateMap, TmpComponent, ViewContext,
};

#[derive(Clone, Copy, Debug)]
pub enum RuntimeEvent<Message> {
    UserInterface(Event),
    Resize { width: u16, height: u16 },
    Custom(Message),
}

#[derive(Default)]
struct AppData {
    state_map: StateMap,
}

pub(crate) fn launch<A, B>(
    mut application: A, receiver: Receiver<RuntimeEvent<A::Message>>, terminal: &mut Terminal<B>,
) -> anyhow::Result<()>
where
    A: Application + 'static,
    B: Backend,
{
    let mut app_data = AppData::default();
    let mut user_interface = {
        let mut ctx = ViewContext::new(&mut app_data.state_map);
        let mut ui = application.view(&mut ctx);
        draw(&mut ui, terminal)?;
        ui
    };

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

                    let mut ctx = ViewContext::new(&mut app_data.state_map);
                    user_interface = application.view(&mut ctx);
                    draw(&mut user_interface, terminal)?;
                }
                RuntimeEvent::Custom(message) => {
                    application.update(message);
                }
                RuntimeEvent::Resize {
                    width: _,
                    height: _,
                } => {
                    let mut ctx = ViewContext::new(&mut app_data.state_map);
                    user_interface = application.view(&mut ctx);
                    // FIXME: Also nuke any cache and the like...
                    draw(&mut user_interface, terminal)?;
                }
            }
        } else {
            break;
        }
    }

    application.destroy();

    Ok(())
}

fn draw<M, B>(user_interface: &mut Element<'_, M>, terminal: &mut Terminal<B>) -> anyhow::Result<()>
where
    B: Backend,
{
    terminal.draw(|frame| {
        let rect = frame.size();
        let layout = build_layout_tree(rect, &user_interface);
        let context = super::DrawContext::root(&layout);

        user_interface.draw(context, frame);
    })?;

    Ok(())
}
