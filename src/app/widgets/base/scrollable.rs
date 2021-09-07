use std::cmp::{min, Ordering};

use crossterm::event::{KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tui::{layout::Rect, widgets::TableState};

use crate::app::{
    event::{MultiKey, MultiKeyResult, WidgetEventResult},
    Component,
};

#[derive(Debug)]
pub enum ScrollDirection {
    Up,
    Down,
}

/// We save the previous window index for future reference, but we must invalidate if the area changes.
#[derive(Default)]
struct WindowIndex {
    index: usize,
    cached_area: Rect,
}

/// A "scrollable" [`Component`].  Intended for use as part of another [`Component`] to help manage scrolled state.
pub struct Scrollable {
    /// The currently selected index. Do *NOT* directly update this, use the helper functions!
    current_index: usize,

    /// The "window index" is the "start" of the displayed data range, used for drawing purposes. See
    /// [`Scrollable::get_list_start`] for more details.
    window_index: WindowIndex,

    /// The direction we're scrolling in.
    scroll_direction: ScrollDirection,

    /// How many items to keep track of.
    num_items: usize,

    /// tui-rs' internal table state; used to keep track of the *visually* selected index.
    tui_state: TableState,

    /// Manages the `gg` double-tap shortcut.
    gg_manager: MultiKey,

    /// The bounds of the [`Scrollable`] component.
    bounds: Rect,
}

impl Scrollable {
    /// Creates a new [`Scrollable`].
    pub fn new(num_items: usize) -> Self {
        let mut tui_state = TableState::default();
        tui_state.select(Some(0));
        Self {
            current_index: 0,
            window_index: WindowIndex::default(),
            scroll_direction: ScrollDirection::Down,
            num_items,
            tui_state,
            gg_manager: MultiKey::register(vec!['g', 'g']), // TODO: Use a static arrayvec
            bounds: Rect::default(),
        }
    }

    /// Returns the currently selected index of the [`Scrollable`].
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Returns the start of the [`Scrollable`] when displayed.
    pub fn get_list_start(&mut self, num_visible_rows: usize) -> usize {
        // So it's probably confusing - what is the "window index"?
        // The idea is that we display a "window" of data in tables that *contains* the currently selected index.
        if self.window_index.cached_area != self.bounds {
            self.window_index.index = 0;
            self.window_index.cached_area = self.bounds;
        }

        match self.scroll_direction {
            ScrollDirection::Down => {
                if self.current_index < self.window_index.index + num_visible_rows {
                    // If, using the current window index, we can see the element
                    // (so within that and + num_visible_rows) just reuse the current previously scrolled position
                } else if self.current_index >= num_visible_rows {
                    // Else if the current position is past the last element visible in the list, omit
                    // until we can see that element. The +1 is because of how indexes start at 0.
                    self.window_index.index = self.current_index - num_visible_rows + 1;
                } else {
                    // Else, if it is not past the last element visible, do not omit anything
                    self.window_index.index = 0;
                }
            }
            ScrollDirection::Up => {
                if self.current_index <= self.window_index.index {
                    // If it's past the first element, then show from that element downwards
                    self.window_index.index = self.current_index;
                } else if self.current_index >= self.window_index.index + num_visible_rows {
                    // Else, if the current index is off screen (sometimes caused by a sudden size change),
                    // just put it so that the selected index is the last entry,
                    self.window_index.index = self.current_index - num_visible_rows + 1;
                }
            }
        }

        // Ensure we don't select a non-existent index.
        self.window_index.index = min(self.num_items.saturating_sub(1), self.window_index.index);

        self.tui_state.select(Some(
            self.current_index.saturating_sub(self.window_index.index),
        ));

        self.window_index.index
    }

    /// Update the index with this!  This will automatically update the scroll direction as well!
    pub fn set_index(&mut self, new_index: usize) {
        match new_index.cmp(&self.current_index) {
            Ordering::Greater => {
                self.current_index = new_index;
                self.scroll_direction = ScrollDirection::Down;
            }
            Ordering::Less => {
                self.current_index = new_index;
                self.scroll_direction = ScrollDirection::Up;
            }
            Ordering::Equal => {
                // Do nothing.
            }
        }
    }

    fn skip_to_first(&mut self) -> WidgetEventResult {
        if self.current_index != 0 {
            self.set_index(0);

            WidgetEventResult::Redraw
        } else {
            WidgetEventResult::NoRedraw
        }
    }

    fn skip_to_last(&mut self) -> WidgetEventResult {
        let last_index = self.num_items - 1;
        if self.current_index != last_index {
            self.set_index(last_index);

            WidgetEventResult::Redraw
        } else {
            WidgetEventResult::NoRedraw
        }
    }

    /// Moves *downward* by *incrementing* the current index.
    fn move_down(&mut self, change_by: usize) -> WidgetEventResult {
        if self.num_items == 0 {
            return WidgetEventResult::NoRedraw;
        }

        let new_index = self.current_index + change_by;
        if new_index >= self.num_items || self.current_index == new_index {
            WidgetEventResult::NoRedraw
        } else {
            self.set_index(new_index);
            WidgetEventResult::Redraw
        }
    }

    /// Moves *upward* by *decrementing* the current index.
    fn move_up(&mut self, change_by: usize) -> WidgetEventResult {
        if self.num_items == 0 {
            return WidgetEventResult::NoRedraw;
        }

        let new_index = self.current_index.saturating_sub(change_by);
        if self.current_index == new_index {
            WidgetEventResult::NoRedraw
        } else {
            self.set_index(new_index);
            WidgetEventResult::Redraw
        }
    }

    pub fn set_num_items(&mut self, num_items: usize) {
        self.num_items = num_items;

        if num_items <= self.current_index {
            self.current_index = num_items.saturating_sub(1);
        }
    }

    pub fn num_items(&self) -> usize {
        self.num_items
    }

    pub fn tui_state(&mut self) -> &mut TableState {
        &mut self.tui_state
    }
}

impl Component for Scrollable {
    fn handle_key_event(&mut self, event: KeyEvent) -> WidgetEventResult {
        use crossterm::event::KeyCode::{Char, Down, Up};

        if event.modifiers == KeyModifiers::NONE || event.modifiers == KeyModifiers::SHIFT {
            match event.code {
                Down if event.modifiers == KeyModifiers::NONE => self.move_down(1),
                Up if event.modifiers == KeyModifiers::NONE => self.move_up(1),
                Char('j') => self.move_down(1),
                Char('k') => self.move_up(1),
                Char('g') => match self.gg_manager.input('g') {
                    MultiKeyResult::Completed => self.skip_to_first(),
                    MultiKeyResult::Accepted => WidgetEventResult::NoRedraw,
                    MultiKeyResult::Rejected => WidgetEventResult::NoRedraw,
                },
                Char('G') => self.skip_to_last(),
                _ => WidgetEventResult::NoRedraw,
            }
        } else {
            WidgetEventResult::NoRedraw
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> WidgetEventResult {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.does_bounds_intersect_mouse(&event) {
                    // This requires a bit of fancy calculation.  The main trick is remembering that
                    // we are using a *visual* index here - not what is the actual index!  Luckily, we keep track of that
                    // inside our linked copy of TableState!

                    // Note that y is assumed to be *relative*;
                    // we assume that y starts at where the list starts (and there are no gaps or whatever).
                    let y = usize::from(event.row - self.bounds.top());

                    if let Some(selected) = self.tui_state.selected() {
                        match y.cmp(&selected) {
                            Ordering::Less => {
                                let offset = selected - y;
                                return self.move_up(offset);
                            }
                            Ordering::Equal => {}
                            Ordering::Greater => {
                                let offset = y - selected;
                                return self.move_down(offset);
                            }
                        }
                    }
                }

                WidgetEventResult::NoRedraw
            }
            MouseEventKind::ScrollDown => self.move_down(1),
            MouseEventKind::ScrollUp => self.move_up(1),
            _ => WidgetEventResult::NoRedraw,
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}
