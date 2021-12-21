use std::cmp::{min, Ordering};
use tui::{layout::Rect, widgets::TableState};

use crate::tuine::Status;

#[derive(Debug, PartialEq, Eq)]
enum ScrollDirection {
    Up,
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}

/// We save the previous window index for future reference, but we must invalidate if the area changes.
#[derive(PartialEq, Default)]
struct WindowIndex {
    index: usize,
    cached_area: Rect,
}

pub struct ScrollState {
    /// The currently selected index. Do *NOT* directly update this, use the helper functions!
    current_index: usize,

    /// The "window index" is the "start" of the displayed data range, used for drawing purposes.
    window_index: WindowIndex,

    /// The direction we're currently scrolling in.
    scroll_direction: ScrollDirection,

    /// How many items to keep track of.
    num_items: usize,

    /// tui-rs' internal table state; used to keep track of the *visually* selected index.
    tui_state: TableState,
}

impl PartialEq for ScrollState {
    fn eq(&self, other: &Self) -> bool {
        self.current_index == other.current_index
            && self.window_index == other.window_index
            && self.scroll_direction == other.scroll_direction
            && self.num_items == other.num_items
            && (self.tui_state.selected() == other.tui_state.selected())
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        let mut tui_state = TableState::default();
        tui_state.select(Some(0));
        Self {
            num_items: 0,
            tui_state,
            current_index: 0,
            window_index: WindowIndex::default(),
            scroll_direction: ScrollDirection::default(),
        }
    }
}

impl ScrollState {
    /// Returns the currently selected index.
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Sets the current scroll index to the given one, or the maximal one possible.
    pub fn checked_set_index(&mut self, new_index: usize) -> Status {
        let new_index = if new_index < self.num_items {
            new_index
        } else {
            self.num_items.saturating_sub(1)
        };

        if new_index == self.current_index {
            Status::Ignored
        } else {
            self.set_index(new_index);
            Status::Captured
        }
    }

    /// Sets the current scroll index to the first possible one.
    pub fn jump_to_first(&mut self) -> Status {
        self.checked_set_index(0)
    }

    /// Sets the current scroll index to the last possible one.
    pub fn jump_to_last(&mut self) -> Status {
        let last_index = self.num_items.saturating_sub(1);
        self.checked_set_index(last_index)
    }

    /// Increments the current scroll index by an amount. This has the effect of scrolling down.
    pub fn move_down(&mut self, change_by: usize) -> Status {
        let new_index = self.current_index + change_by;
        self.checked_set_index(new_index)
    }

    /// Decrements the current scroll index by an amount. This has the effect of scrolling up.
    pub fn move_up(&mut self, change_by: usize) -> Status {
        let new_index = self.current_index.saturating_sub(change_by);
        self.checked_set_index(new_index)
    }

    /// Sets the number of items currently in the list.
    pub(crate) fn set_num_items(&mut self, num_items: usize) {
        self.num_items = num_items;

        if num_items <= self.current_index {
            self.current_index = num_items.saturating_sub(1);
        }
    }

    pub(crate) fn num_items(&self) -> usize {
        self.num_items
    }

    pub(crate) fn tui_state(&mut self) -> &mut TableState {
        &mut self.tui_state
    }

    pub(crate) fn display_start_index(&mut self, bounds: Rect, num_visible_rows: usize) -> usize {
        // So it's probably confusing - what is the "window index"?
        // The idea is that we display a "window" of data in tables that *contains* the currently selected index.
        if self.window_index.cached_area != bounds {
            self.window_index.index = 0;
            self.window_index.cached_area = bounds;
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

    /// Sets the index based on a visual index. Note this does NOT do bounds checking.
    pub(crate) fn set_visual_index(&mut self, visual_index: usize) -> Status {
        let selected = self.tui_state().selected().unwrap_or(0);
        match visual_index.cmp(&selected) {
            Ordering::Less => {
                let offset = selected - visual_index;
                self.move_up(offset)
            }
            Ordering::Greater => {
                let offset = visual_index - selected;
                self.move_down(offset)
            }
            Ordering::Equal => Status::Ignored,
        }
    }

    /// Sets the index along with scroll direction. Note that this DOES NOT
    /// check if the set index is valid - use one of the public functions
    /// for that!
    fn set_index(&mut self, new_index: usize) {
        match new_index.cmp(&self.current_index) {
            Ordering::Greater => {
                self.current_index = new_index;
                self.scroll_direction = ScrollDirection::Down;
            }
            Ordering::Less => {
                self.current_index = new_index;
                self.scroll_direction = ScrollDirection::Up;
            }
            Ordering::Equal => { /* Do nothing */ }
        }
    }
}

#[cfg(test)]
mod test {
    use tui::layout::Rect;

    use crate::tuine::{text_table::table_scroll_state::ScrollDirection, Status};

    use super::ScrollState;

    const NUM_ITEMS: usize = 100;

    fn create_scroll_state(num_items: usize) -> ScrollState {
        let mut state = ScrollState::default();
        state.set_num_items(num_items);

        state
    }

    #[test]
    fn jumping_works() {
        let mut state = create_scroll_state(NUM_ITEMS);

        assert_eq!(state.jump_to_last(), Status::Captured);
        assert_eq!(state.current_index(), NUM_ITEMS - 1);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        assert_eq!(state.jump_to_last(), Status::Ignored);
        assert_eq!(state.current_index(), NUM_ITEMS - 1);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        assert_eq!(state.jump_to_first(), Status::Captured);
        assert_eq!(state.current_index(), 0);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);

        assert_eq!(state.jump_to_first(), Status::Ignored);
        assert_eq!(state.current_index(), 0);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);
    }

    #[test]
    fn moving_works() {
        let mut state = create_scroll_state(NUM_ITEMS);

        assert_eq!(state.move_down(50), Status::Captured);
        assert_eq!(state.current_index(), 50);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        assert_eq!(state.move_down(0), Status::Ignored);
        assert_eq!(state.current_index(), 50);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        assert_eq!(state.move_up(30), Status::Captured);
        assert_eq!(state.current_index(), 20);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);

        assert_eq!(state.move_up(0), Status::Ignored);
        assert_eq!(state.current_index(), 20);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);
    }

    #[test]
    fn moving_limits() {
        let mut state = create_scroll_state(NUM_ITEMS);

        assert_eq!(state.move_down(100), Status::Captured);
        assert_eq!(state.current_index(), 99);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        assert_eq!(state.move_down(100), Status::Ignored);
        assert_eq!(state.current_index(), 99);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        assert_eq!(state.move_up(100), Status::Captured);
        assert_eq!(state.current_index(), 0);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);

        assert_eq!(state.move_up(100), Status::Ignored);
        assert_eq!(state.current_index(), 0);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);
    }

    #[test]
    fn setting_index_works() {
        let mut state = create_scroll_state(NUM_ITEMS);

        state.checked_set_index(50);
        assert_eq!(state.current_index(), 50);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        state.checked_set_index(30);
        assert_eq!(state.current_index(), 30);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);

        state.checked_set_index(99);
        assert_eq!(state.current_index(), 99);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        state.checked_set_index(100);
        assert_eq!(state.current_index(), 99);
        assert_eq!(state.scroll_direction, ScrollDirection::Down);

        state.checked_set_index(0);
        assert_eq!(state.current_index(), 0);
        assert_eq!(state.scroll_direction, ScrollDirection::Up);
    }

    #[test]
    fn new_size_adjusts() {
        let mut state = create_scroll_state(NUM_ITEMS);

        state.move_down(50);
        assert_eq!(state.current_index(), 50);

        state.set_num_items(10);
        assert_eq!(state.num_items(), 10);
        assert_eq!(state.current_index(), 9);

        state.set_num_items(100);
        assert_eq!(state.num_items(), 100);
        assert_eq!(state.current_index(), 9);
    }

    #[test]
    fn getting_list_start() {
        const WINDOW_SIZE: usize = 10;
        const END_INDEX: usize = NUM_ITEMS - WINDOW_SIZE;

        let mut state = create_scroll_state(NUM_ITEMS);
        let bounds = Rect::default();

        state.checked_set_index(0);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 0);

        state.checked_set_index(5);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 0);

        state.checked_set_index(9);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 0);

        state.checked_set_index(10);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 1);

        state.checked_set_index(9);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 1);

        state.jump_to_last();
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), END_INDEX);

        state.move_up(1);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), END_INDEX);

        state.move_up(8);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), END_INDEX);

        state.move_up(1);
        assert_eq!(
            state.display_start_index(bounds, WINDOW_SIZE),
            END_INDEX - 1
        );

        state.move_up(1);
        assert_eq!(
            state.display_start_index(bounds, WINDOW_SIZE),
            END_INDEX - 2
        );

        state.move_down(1);
        assert_eq!(
            state.display_start_index(bounds, WINDOW_SIZE),
            END_INDEX - 2
        );

        state.move_down(9);
        assert_eq!(
            state.display_start_index(bounds, WINDOW_SIZE),
            END_INDEX - 1
        );

        state.move_down(1);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), END_INDEX);
    }

    #[test]
    fn setting_visual_index() {
        const WINDOW_SIZE: usize = 10;
        let bounds = Rect::default();

        let mut state = create_scroll_state(NUM_ITEMS);
        state.checked_set_index(10);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 1);

        state.set_visual_index(5);
        assert_eq!(state.display_start_index(bounds, WINDOW_SIZE), 1);
    }
}
