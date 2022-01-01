use std::{
    collections::hash_map::Entry,
    panic::Location,
    time::{Duration, Instant},
};

use rustc_hash::FxHashMap;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuine::{
    Bounds, DrawContext, Event, Key, LayoutNode, Size, StateContext, StatefulComponent, Status,
    TmpComponent,
};

const MAX_TIMEOUT: Duration = Duration::from_millis(400);

#[derive(Debug, PartialEq)]
enum ShortcutTriggerState {
    /// Currently not waiting on any next input.
    Idle,
    /// Waiting for the next input, initially triggered at [`Instant`].
    Waiting {
        /// When it was initially triggered.
        trigger_instant: Instant,

        /// The currently built-up list of events.
        current_events: Vec<Event>,
    },
}

impl Default for ShortcutTriggerState {
    fn default() -> Self {
        ShortcutTriggerState::Idle
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct ShortcutState {
    trigger_state: ShortcutTriggerState,
    forest: FxHashMap<Vec<Event>, bool>,
}

/// Properties for a [`Shortcut`].
#[derive(Default)]
pub struct ShortcutProps<Message, Child>
where
    Child: TmpComponent<Message>,
{
    child: Option<Child>,
    shortcuts: FxHashMap<
        Vec<Event>,
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
}

impl<Message, Child> ShortcutProps<Message, Child>
where
    Child: TmpComponent<Message>,
{
    /// Creates a new [`ShortcutProps`] with a child.
    pub fn with_child(child: Child) -> Self {
        Self {
            child: Some(child),
            shortcuts: Default::default(),
        }
    }

    /// Sets the child of the [`ShortcutProps`].
    pub fn child(mut self, child: Option<Child>) -> Self {
        self.child = child;
        self
    }

    /// Inserts a shortcut that only needs a single [`Event`].
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
        self.shortcuts.insert(vec![event], f);
        self
    }

    /// Inserts a shortcut that can take one or more [`Event`]s.
    pub fn multi_shortcut(
        mut self, events: Vec<Event>,
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
        self.shortcuts.insert(events, f);
        self
    }
}

/// A [`Component`] to handle keyboard shortcuts and assign actions to them.
///
/// Inspired by [Flutter's approach](https://docs.flutter.dev/development/ui/advanced/actions_and_shortcuts).
pub struct Shortcut<Message, Child>
where
    Child: TmpComponent<Message>,
{
    key: Key,
    child: Option<Child>,
    shortcuts: FxHashMap<
        Vec<Event>,
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
}

impl<Message, Child> StatefulComponent<Message> for Shortcut<Message, Child>
where
    Child: TmpComponent<Message>,
{
    type Properties = ShortcutProps<Message, Child>;

    type ComponentState = ShortcutState;

    fn build(ctx: &mut crate::tuine::ViewContext<'_>, props: Self::Properties) -> Self {
        let (key, state) =
            ctx.register_and_mut_state::<_, Self::ComponentState>(Location::caller());
        let mut forest: FxHashMap<Vec<Event>, bool> = FxHashMap::default();

        props.shortcuts.iter().for_each(|(events, _action)| {
            if !events.is_empty() {
                let mut visited = vec![];
                let last = events.len() - 1;
                for (itx, event) in events.iter().enumerate() {
                    visited.push(*event);
                    match forest.entry(visited.clone()) {
                        Entry::Occupied(mut occupied) => {
                            *occupied.get_mut() = *occupied.get() || itx == last;
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(itx == last);
                        }
                    }
                }
            }
        });

        if forest != state.forest {
            // Invalidate state.
            *state = ShortcutState {
                trigger_state: ShortcutTriggerState::Idle,
                forest,
            };
        } else if let ShortcutTriggerState::Waiting {
            trigger_instant,
            current_events: _,
        } = state.trigger_state
        {
            if Instant::now().duration_since(trigger_instant) > MAX_TIMEOUT {
                // Invalidate state.
                *state = ShortcutState {
                    trigger_state: ShortcutTriggerState::Idle,
                    forest,
                };
            }
        }

        Shortcut {
            key,
            child: props.child,
            shortcuts: props.shortcuts,
        }
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
                        let state = state_ctx.mut_state::<ShortcutState>(self.key);
                        match &state.trigger_state {
                            ShortcutTriggerState::Idle => {
                                let current_events = vec![event];
                                if let Some(&should_fire) = state.forest.get(&current_events) {
                                    state.trigger_state = ShortcutTriggerState::Waiting {
                                        trigger_instant: Instant::now(),
                                        current_events: current_events.clone(),
                                    };

                                    if should_fire {
                                        if let Some(f) = self.shortcuts.get(&current_events) {
                                            return f(
                                                child,
                                                state_ctx,
                                                &child_draw_ctx,
                                                event,
                                                messages,
                                            );
                                        }
                                    }
                                }
                            }
                            ShortcutTriggerState::Waiting {
                                trigger_instant,
                                current_events,
                            } => {
                                if Instant::now().duration_since(*trigger_instant) > MAX_TIMEOUT {
                                    state.trigger_state = ShortcutTriggerState::Idle;
                                    return self.on_event(state_ctx, draw_ctx, event, messages);
                                } else {
                                    let mut current_events = current_events.clone();
                                    current_events.push(event);

                                    if let Some(&should_fire) = state.forest.get(&current_events) {
                                        state.trigger_state = ShortcutTriggerState::Waiting {
                                            trigger_instant: Instant::now(),
                                            current_events: current_events.clone(),
                                        };

                                        if should_fire {
                                            if let Some(f) = self.shortcuts.get(&current_events) {
                                                return f(
                                                    child,
                                                    state_ctx,
                                                    &child_draw_ctx,
                                                    event,
                                                    messages,
                                                );
                                            }
                                        }
                                    } else {
                                        state.trigger_state = ShortcutTriggerState::Idle;
                                        return self.on_event(state_ctx, draw_ctx, event, messages);
                                    }
                                }
                            }
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
