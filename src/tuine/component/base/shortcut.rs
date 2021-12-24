use crossterm::event::KeyEvent;
use rustc_hash::FxHashMap;
use tui::{backend::Backend, Frame};

use crate::tuine::{
    Bounds, DrawContext, Element, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

/// A [`Component`] to handle keyboard shortcuts and assign actions to them.
///
/// Inspired by [Flutter's approach](https://docs.flutter.dev/development/ui/advanced/actions_and_shortcuts).
#[derive(Default)]
pub struct Shortcut<'a, Message> {
    child: Option<Box<Element<'a, Message>>>,
    shortcuts: FxHashMap<
        Event,
        Box<
            dyn Fn(
                &mut Element<'a, Message>,
                &mut StateContext<'_>,
                DrawContext<'_>,
                Event,
                &mut Vec<Message>,
            ) -> Status,
        >,
    >,
}

impl<'a, Message> Shortcut<'a, Message> {
    pub fn with_child(child: Element<'a, Message>) -> Self {
        Self {
            child: Some(child.into()),
            shortcuts: Default::default(),
        }
    }
}

impl<'a, Message> TmpComponent<Message> for Shortcut<'a, Message> {
    fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        if let Some(child) = &mut self.child {
            if let Some(child_draw_ctx) = draw_ctx.children().next() {
                child.draw(state_ctx, child_draw_ctx, frame)
            }
        }
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        if let Some(child_draw_ctx) = draw_ctx.children().next() {
            if let Some(child) = &mut self.child {
                match child.on_event(state_ctx, child_draw_ctx, event, messages) {
                    Status::Captured => {
                        return Status::Captured;
                    }
                    Status::Ignored => {
                        if let Some(f) = self.shortcuts.get(&event) {
                            return f(child, state_ctx, child_draw_ctx, event, messages);
                        }
                    }
                }
            }
        }

        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        todo!()
    }
}
