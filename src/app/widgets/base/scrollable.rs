use crossterm::event::{KeyEvent, KeyModifiers, MouseButton, MouseEvent};
use tui::{layout::Rect, widgets::TableState};

use crate::app::{
    event::{EventResult, MultiKey, MultiKeyResult},
    Component,
};

pub enum ScrollDirection {
    Up,
    Down,
}

/// A "scrollable" [`Widget`] component.  Intended for use as part of another [`Widget`] - as such, it does
/// not have any bounds or the like.
pub struct Scrollable {
    current_index: usize,
    previous_index: usize,
    scroll_direction: ScrollDirection,
    num_items: usize,

    tui_state: TableState,
    gg_manager: MultiKey,

    bounds: Rect,
}

impl Scrollable {
    /// Creates a new [`Scrollable`].
    pub fn new(num_items: usize) -> Self {
        Self {
            current_index: 0,
            previous_index: 0,
            scroll_direction: ScrollDirection::Down,
            num_items,
            tui_state: TableState::default(),
            gg_manager: MultiKey::register(vec!['g', 'g']), // TODO: Use a static arrayvec
            bounds: Rect::default(),
        }
    }

    /// Creates a new [`Scrollable`].  Note this will set the associated [`TableState`] to select the first entry.
    pub fn new_selected(num_items: usize) -> Self {
        let mut scrollable = Scrollable::new(num_items);
        scrollable.tui_state.select(Some(0));

        scrollable
    }

    pub fn index(&self) -> usize {
        self.current_index
    }

    /// Update the index with this!  This will automatically update the previous index and scroll direction!
    fn update_index(&mut self, new_index: usize) {
        use std::cmp::Ordering;

        match new_index.cmp(&self.current_index) {
            Ordering::Greater => {
                self.previous_index = self.current_index;
                self.current_index = new_index;
                self.scroll_direction = ScrollDirection::Down;
            }
            Ordering::Less => {
                self.previous_index = self.current_index;
                self.current_index = new_index;
                self.scroll_direction = ScrollDirection::Up;
            }

            Ordering::Equal => {}
        }
    }

    fn skip_to_first(&mut self) -> EventResult {
        if self.current_index != 0 {
            self.update_index(0);

            EventResult::Redraw
        } else {
            EventResult::NoRedraw
        }
    }

    fn skip_to_last(&mut self) -> EventResult {
        let last_index = self.num_items - 1;
        if self.current_index != last_index {
            self.update_index(last_index);

            EventResult::Redraw
        } else {
            EventResult::NoRedraw
        }
    }

    /// Moves *downward* by *incrementing* the current index.
    fn move_down(&mut self, change_by: usize) -> EventResult {
        let new_index = self.current_index + change_by;
        if new_index >= self.num_items {
            let last_index = self.num_items - 1;
            if self.current_index != last_index {
                self.update_index(last_index);

                EventResult::Redraw
            } else {
                EventResult::NoRedraw
            }
        } else {
            self.update_index(new_index);
            EventResult::Redraw
        }
    }

    /// Moves *upward* by *decrementing* the current index.
    fn move_up(&mut self, change_by: usize) -> EventResult {
        let new_index = self.current_index.saturating_sub(change_by);
        if new_index == 0 {
            if self.current_index != 0 {
                self.update_index(0);

                EventResult::Redraw
            } else {
                EventResult::NoRedraw
            }
        } else {
            self.update_index(new_index);
            EventResult::Redraw
        }
    }

    pub fn update_num_items(&mut self, num_items: usize) {
        self.num_items = num_items;

        if num_items <= self.current_index {
            self.current_index = num_items - 1;
        }

        if num_items <= self.previous_index {
            self.previous_index = num_items - 1;
        }
    }
}

impl Component for Scrollable {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        use crossterm::event::KeyCode::{Char, Down, Up};

        if event.modifiers == KeyModifiers::NONE || event.modifiers == KeyModifiers::SHIFT {
            match event.code {
                Down if event.modifiers == KeyModifiers::NONE => self.move_down(1),
                Up if event.modifiers == KeyModifiers::NONE => self.move_up(1),
                Char('j') => self.move_down(1),
                Char('k') => self.move_up(1),
                Char('g') => match self.gg_manager.input('g') {
                    MultiKeyResult::Completed => self.skip_to_first(),
                    MultiKeyResult::Accepted => EventResult::NoRedraw,
                    MultiKeyResult::Rejected => EventResult::NoRedraw,
                },
                Char('G') => self.skip_to_last(),
                _ => EventResult::NoRedraw,
            }
        } else {
            EventResult::NoRedraw
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        match event.kind {
            crossterm::event::MouseEventKind::Down(MouseButton::Left) => {
                // This requires a bit of fancy calculation.  The main trick is remembering that
                // we are using a *visual* index here - not what is the actual index!  Luckily, we keep track of that
                // inside our linked copy of TableState!

                // Note that y is assumed to be *relative*;
                // we assume that y starts at where the list starts (and there are no gaps or whatever).
                let y = usize::from(event.row - self.bounds.top());

                if let Some(selected) = self.tui_state.selected() {
                    if y > selected {
                        let offset = y - selected;
                        return self.move_down(offset);
                    } else {
                        let offset = selected - y;
                        return self.move_up(offset);
                    }
                }

                EventResult::NoRedraw
            }
            crossterm::event::MouseEventKind::ScrollDown => self.move_down(1),
            crossterm::event::MouseEventKind::ScrollUp => self.move_up(1),
            _ => EventResult::NoRedraw,
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}
