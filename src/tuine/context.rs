use std::panic::Location;

use rustc_hash::FxHashMap;
use tui::layout::Rect;

use super::{Key, LayoutNode, State};

#[derive(Default)]
pub struct ComponentContext {
    key_counter: usize,
    state_map: FxHashMap<Key, Box<dyn State>>,
    stale_map: FxHashMap<Key, bool>,
}

impl ComponentContext {
    pub fn access_or_new<S: State + Default + 'static>(
        &mut self, location: &'static Location<'static>,
    ) -> &mut Box<dyn State> {
        let key = Key::new(location, self.key_counter);
        self.key_counter += 1;

        *(self.stale_map.entry(key).or_insert(true)) = true;
        self.state_map
            .entry(key)
            .or_insert_with(|| Box::new(S::default()))
    }

    pub fn cycle(&mut self) {}
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
