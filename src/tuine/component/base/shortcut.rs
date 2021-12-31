use rustc_hash::FxHashMap;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuine::{
    Bounds, DrawContext, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

enum MultiShortcutStep<Message, Child>
where
    Child: TmpComponent<Message>,
{
    NextStep(Event),
    Action(
        Box<
            dyn Fn(
                &mut Child,
                &mut StateContext<'_>,
                &DrawContext<'_>,
                Event,
                &mut Vec<Message>,
            ) -> Status,
        >,
    ),
}

/// A [`Component`] to handle keyboard shortcuts and assign actions to them.
///
/// Inspired by [Flutter's approach](https://docs.flutter.dev/development/ui/advanced/actions_and_shortcuts).
#[derive(Default)]
pub struct Shortcut<Message, Child>
where
    Child: TmpComponent<Message>,
{
    child: Option<Child>,
    shortcuts: FxHashMap<
        Event,
        Box<
            dyn Fn(
                &mut Child,
                &mut StateContext<'_>,
                &DrawContext<'_>,
                Event,
                &mut Vec<Message>,
            ) -> Status,
        >,
    >,
    multi_shortcuts: FxHashMap<Event, MultiShortcutStep<Message, Child>>,
    enabled_multi_shortcuts: FxHashMap<Event, MultiShortcutStep<Message, Child>>,
}

impl<Message, Child> Shortcut<Message, Child>
where
    Child: TmpComponent<Message>,
{
    pub fn with_child(child: Child) -> Self {
        Self {
            child: Some(child),
            shortcuts: Default::default(),
            multi_shortcuts: Default::default(),
            enabled_multi_shortcuts: Default::default(),
        }
    }

    pub fn child(mut self, child: Option<Child>) -> Self {
        self.child = child;
        self
    }

    pub fn shortcut(
        mut self, event: Event,
        f: Box<
            dyn Fn(
                &mut Child,
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

impl<'a, Message, Child> TmpComponent<Message> for Shortcut<Message, Child>
where
    Child: TmpComponent<Message>,
{
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
