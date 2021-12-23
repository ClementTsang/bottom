use std::{panic::Location, rc::Rc};

use rustc_hash::FxHashMap;
use tui::layout::Rect;

use super::{Key, LayoutNode, State};

#[derive(Default)]
pub struct StateMap(FxHashMap<Key, (Rc<Box<dyn State>>, bool)>);

impl StateMap {
    pub fn state<S: State + Default + 'static>(&mut self, key: Key) -> Rc<Box<dyn State>> {
        let state = self
            .0
            .entry(key)
            .or_insert_with(|| (Rc::new(Box::new(S::default())), true));

        state.1 = true;

        state.0.clone()
    }
}

pub struct ViewContext<'a> {
    key_counter: usize,
    state_map: &'a mut StateMap,
}

impl<'a> ViewContext<'a> {
    pub fn new(state_map: &'a mut StateMap) -> Self {
        Self {
            key_counter: 0,
            state_map,
        }
    }

    pub fn state<S: State + Default + 'static>(
        &mut self, location: &'static Location<'static>,
    ) -> Rc<Box<dyn State>> {
        let key = Key::new(location, self.key_counter);
        self.key_counter += 1;
        self.state_map.state::<S>(key)
    }
}

pub struct DrawContext<'a> {
    current_node: &'a LayoutNode,
    current_offset: (u16, u16),
}

impl<'a> DrawContext<'_> {
    /// Creates a new [`DrawContext`], with the offset set to `(0, 0)`.
    pub(crate) fn root(root: &'a LayoutNode) -> DrawContext<'a> {
        DrawContext {
            current_node: root,
            current_offset: (0, 0),
        }
    }

    pub(crate) fn rect(&self) -> Rect {
        let mut rect = self.current_node.rect;
        rect.x += self.current_offset.0;
        rect.y += self.current_offset.1;

        rect
    }

    pub(crate) fn should_draw(&self) -> bool {
        self.current_node.rect.area() != 0
    }

    pub(crate) fn children(&self) -> impl Iterator<Item = DrawContext<'_>> {
        let new_offset = (
            self.current_offset.0 + self.current_node.rect.x,
            self.current_offset.1 + self.current_node.rect.y,
        );

        self.current_node
            .children
            .iter()
            .map(move |layout_node| DrawContext {
                current_node: layout_node,
                current_offset: new_offset,
            })
    }
}
