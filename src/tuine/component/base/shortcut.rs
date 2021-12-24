use rustc_hash::FxHashMap;
use tui::{backend::Backend, layout::Rect, Frame};

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
                &DrawContext<'_>,
                Event,
                &mut Vec<Message>,
            ) -> Status,
        >,
    >,
}

impl<'a, Message> Shortcut<'a, Message> {
    pub fn with_child<C>(child: C) -> Self
    where
        C: Into<Element<'a, Message>>,
    {
        Self {
            child: Some(Box::new(child.into())),
            shortcuts: Default::default(),
        }
    }

    pub fn child<C>(mut self, child: Option<C>) -> Self
    where
        C: Into<Element<'a, Message>>,
    {
        self.child = child.map(|c| Box::new(c.into()));
        self
    }

    pub fn shortcut(
        mut self, event: Event,
        f: Box<
            dyn Fn(
                &mut Element<'a, Message>,
                &mut StateContext<'_>,
                &DrawContext<'_>,
                Event,
                &mut Vec<Message>,
            ) -> Status,
        >,
    ) -> Self {
        self.shortcuts.insert(event, f);
        self
    }

    pub fn remove_shortcut(mut self, event: &Event) -> Self {
        self.shortcuts.remove(event);
        self
    }
}

impl<'a, Message> TmpComponent<Message> for Shortcut<'a, Message> {
    fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        if let Some(child) = &mut self.child {
            if let Some(child_draw_ctx) = draw_ctx.children().next() {
                child.draw(state_ctx, &child_draw_ctx, frame)
            }
        }
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        if let Some(child_draw_ctx) = draw_ctx.children().next() {
            if let Some(child) = &mut self.child {
                match child.on_event(state_ctx, &child_draw_ctx, event, messages) {
                    Status::Captured => {
                        return Status::Captured;
                    }
                    Status::Ignored => {
                        if let Some(f) = self.shortcuts.get(&event) {
                            return f(child, state_ctx, &child_draw_ctx, event, messages);
                        }
                    }
                }
            }
        }

        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        if let Some(child) = &self.child {
            let mut child_node = LayoutNode::default();
            let child_size = child.layout(bounds, &mut child_node);

            child_node.rect = Rect::new(0, 0, child_size.width, child_size.height);
            node.children = vec![child_node];

            child_size
        } else {
            Size {
                width: 0,
                height: 0,
            }
        }
    }
}
