use std::sync::mpsc::Receiver;

use tui::{backend::Backend, Terminal};

use crate::tuine::Status;

use super::{
    build_layout_tree, Application, DrawContext, Element, Event, LayoutNode, StateContext,
    StateMap, TmpComponent, ViewContext,
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
    let mut app_data = AppData::default(); // FIXME: This needs to be cleared periodically, DO!
    let mut layout: LayoutNode = LayoutNode::default();

    let mut user_interface = {
        let mut ui = new_user_interface(&mut application, &mut app_data);
        draw(&mut ui, terminal, &mut app_data, &mut layout)?;
        ui
    };

    while !application.is_terminated() {
        if let Ok(event) = receiver.recv() {
            match event {
                RuntimeEvent::UserInterface(event) => {
                    if on_event(
                        &mut application,
                        &mut user_interface,
                        &mut app_data,
                        &mut layout,
                        event,
                    ) {
                        user_interface = new_user_interface(&mut application, &mut app_data);
                        draw(&mut user_interface, terminal, &mut app_data, &mut layout)?;
                    }
                }
                RuntimeEvent::Custom(message) => {
                    if application.update(message) {
                        user_interface = new_user_interface(&mut application, &mut app_data);
                        draw(&mut user_interface, terminal, &mut app_data, &mut layout)?;
                    }
                }
                RuntimeEvent::Resize {
                    width: _,
                    height: _,
                } => {
                    user_interface = new_user_interface(&mut application, &mut app_data);
                    // FIXME: Also nuke any cache and the like...
                    draw(&mut user_interface, terminal, &mut app_data, &mut layout)?;
                }
            }
        } else {
            break;
        }
    }

    application.destructor();

    Ok(())
}

/// Handles a [`Event`].
fn on_event<A>(
    application: &mut A, user_interface: &mut Element<A::Message>, app_data: &mut AppData,
    layout: &mut LayoutNode, event: Event,
) -> bool
where
    A: Application + 'static,
{
    let mut messages = vec![];
    let mut state_ctx = StateContext::new(&mut app_data.state_map);
    let draw_ctx = DrawContext::root(&layout);

    let event_handled =
        match user_interface.on_event(&mut state_ctx, &draw_ctx, event, &mut messages) {
            Status::Captured => {
                // TODO: What to do on capture?
                Status::Captured
            }
            Status::Ignored => application.global_event_handler(event, &mut messages),
        };

    let mut should_redraw = match event_handled {
        Status::Captured => true,
        Status::Ignored => false,
    };

    for msg in messages {
        debug!("Message: {:?}", msg); // FIXME: Remove this debug line!
        let msg_result = application.update(msg);
        should_redraw = should_redraw || msg_result;
    }

    should_redraw
}

/// Creates a new [`Element`] representing the root of the user interface.
fn new_user_interface<A>(application: &mut A, app_data: &mut AppData) -> Element<A::Message>
where
    A: Application + 'static,
{
    let mut ctx = ViewContext::new(&mut app_data.state_map);
    application.view(&mut ctx)
}

/// Updates the layout, and draws the given user interface.
fn draw<M, B>(
    user_interface: &mut Element<M>, terminal: &mut Terminal<B>, app_data: &mut AppData,
    layout: &mut LayoutNode,
) -> anyhow::Result<()>
where
    B: Backend,
{
    terminal.draw(|frame| {
        let rect = frame.size();
        *layout = build_layout_tree(rect, &user_interface);
        let mut state_ctx = StateContext::new(&mut app_data.state_map);
        let draw_ctx = DrawContext::root(&layout);

        user_interface.draw(&mut state_ctx, &draw_ctx, frame);
    })?;

    Ok(())
}
