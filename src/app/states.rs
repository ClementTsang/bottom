use std::ops::Range;

use hashbrown::HashMap;
use indexmap::IndexMap;
use unicode_ellipsis::grapheme_width;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

use crate::{
    app::layout_manager::BottomWidgetType,
    constants,
    widgets::{
        BatteryWidgetState, CpuWidgetState, DiskTableWidget, MemWidgetState, NetWidgetState,
        ProcWidgetState, TempWidgetState, query::ProcessQuery,
    },
};

pub struct AppWidgetStates {
    pub cpu_state: CpuState,
    pub mem_state: MemState,
    pub net_state: NetState,
    pub proc_state: ProcState,
    pub temp_state: TempState,
    pub disk_state: DiskState,
    pub battery_state: AppBatteryState,
    pub basic_table_widget_state: Option<BasicTableWidgetState>,
}

#[derive(Debug)]
pub enum CursorDirection {
    Left,
    Right,
}

pub struct AppHelpDialogState {
    pub is_showing_help: bool,
    pub height: u16,
    pub scroll_state: ParagraphScrollState,
    pub index_shortcuts: Vec<u16>,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
            height: 0,
            scroll_state: ParagraphScrollState::default(),
            index_shortcuts: vec![0; constants::HELP_TEXT.len()],
        }
    }
}

/// AppSearchState deals with generic searching (I might do this in the future).
pub struct AppSearchState {
    pub is_enabled: bool,
    pub current_search_query: String,
    pub is_blank_search: bool,
    pub is_invalid_search: bool,
    pub grapheme_cursor: GraphemeCursor,
    pub cursor_direction: CursorDirection,

    pub display_start_char_index: usize,
    pub size_mappings: IndexMap<usize, Range<usize>>,

    /// The query. TODO: Merge this as one enum.
    pub query: Option<ProcessQuery>,
    pub error_message: Option<String>,
}

impl Default for AppSearchState {
    fn default() -> Self {
        AppSearchState {
            is_enabled: false,
            current_search_query: String::default(),
            is_invalid_search: false,
            is_blank_search: true,
            grapheme_cursor: GraphemeCursor::new(0, 0, true),
            cursor_direction: CursorDirection::Right,
            display_start_char_index: 0,
            size_mappings: IndexMap::default(),
            query: None,
            error_message: None,
        }
    }
}

impl AppSearchState {
    /// Resets the [`AppSearchState`] to its default state, albeit still
    /// enabled.
    pub fn reset(&mut self) {
        *self = AppSearchState {
            is_enabled: self.is_enabled,
            ..AppSearchState::default()
        }
    }

    /// Returns whether the [`AppSearchState`] has an invalid or blank search.
    pub fn is_invalid_or_blank_search(&self) -> bool {
        self.is_blank_search || self.is_invalid_search
    }

    /// Sets the starting grapheme index to draw from.
    pub fn get_start_position(&mut self, available_width: usize, is_force_redraw: bool) {
        // Remember - the number of columns != the number of grapheme slots/sizes, you
        // cannot use index to determine this reliably!

        let start_index = if is_force_redraw {
            0
        } else {
            self.display_start_char_index
        };
        let cursor_index = self.grapheme_cursor.cur_cursor();

        if let Some(start_range) = self.size_mappings.get(&start_index) {
            let cursor_range = self
                .size_mappings
                .get(&cursor_index)
                .cloned()
                .unwrap_or_else(|| {
                    self.size_mappings
                        .last()
                        .map(|(_, r)| r.end..(r.end + 1))
                        .unwrap_or(start_range.end..(start_range.end + 1))
                });

            // Cases to handle in both cases:
            // - The current start index can show the cursor's word.
            // - The current start index cannot show the cursor's word.
            //
            // What differs is how we "scroll" based on the cursor movement direction.

            self.display_start_char_index = match self.cursor_direction {
                CursorDirection::Right => {
                    if start_range.start + available_width >= cursor_range.end {
                        // Use the current index.
                        start_index
                    } else if cursor_range.end >= available_width {
                        // If the current position is past the last visible element, skip until we
                        // see it.

                        let mut index = 0;
                        for i in 0..(cursor_index + 1) {
                            if let Some(r) = self.size_mappings.get(&i) {
                                if r.start + available_width >= cursor_range.end {
                                    index = i;
                                    break;
                                }
                            }
                        }

                        index
                    } else {
                        0
                    }
                }
                CursorDirection::Left => {
                    if cursor_range.start < start_range.end {
                        let mut index = 0;
                        for i in cursor_index..(self.current_search_query.len()) {
                            if let Some(r) = self.size_mappings.get(&i) {
                                if r.start + available_width >= cursor_range.end {
                                    index = i;
                                    break;
                                }
                            }
                        }
                        index
                    } else {
                        start_index
                    }
                }
            };
        } else {
            // If we fail here somehow, just reset to 0 index + scroll left.
            self.display_start_char_index = 0;
            self.cursor_direction = CursorDirection::Left;
        };
    }

    pub(crate) fn walk_forward(&mut self) {
        // TODO: Add tests for this.
        let start_position = self.grapheme_cursor.cur_cursor();
        let chunk = &self.current_search_query[start_position..];

        match self.grapheme_cursor.next_boundary(chunk, start_position) {
            Ok(_) => {}
            Err(err) => match err {
                GraphemeIncomplete::PreContext(ctx) => {
                    // Provide the entire string as context. Not efficient but should resolve
                    // failures.
                    self.grapheme_cursor
                        .provide_context(&self.current_search_query[0..ctx], 0);

                    self.grapheme_cursor
                        .next_boundary(chunk, start_position)
                        .unwrap();
                }
                _ => panic!("{err:?}"),
            },
        }
    }

    pub(crate) fn walk_backward(&mut self) {
        // TODO: Add tests for this.
        let start_position = self.grapheme_cursor.cur_cursor();
        let chunk = &self.current_search_query[..start_position];

        match self.grapheme_cursor.prev_boundary(chunk, 0) {
            Ok(_) => {}
            Err(err) => match err {
                GraphemeIncomplete::PreContext(ctx) => {
                    // Provide the entire string as context. Not efficient but should resolve
                    // failures.
                    self.grapheme_cursor
                        .provide_context(&self.current_search_query[0..ctx], 0);

                    self.grapheme_cursor.prev_boundary(chunk, 0).unwrap();
                }
                _ => panic!("{err:?}"),
            },
        }
    }

    pub(crate) fn update_sizes(&mut self) {
        self.size_mappings.clear();
        let mut curr_offset = 0;
        for (index, grapheme) in
            UnicodeSegmentation::grapheme_indices(self.current_search_query.as_str(), true)
        {
            let width = grapheme_width(grapheme);
            let end = curr_offset + width;

            self.size_mappings.insert(index, curr_offset..end);

            curr_offset = end;
        }

        self.size_mappings.shrink_to_fit();
    }
}

pub struct ProcState {
    pub widget_states: HashMap<u64, ProcWidgetState>,
}

impl ProcState {
    pub fn init(widget_states: HashMap<u64, ProcWidgetState>) -> Self {
        ProcState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut ProcWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&ProcWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

pub struct NetState {
    pub widget_states: HashMap<u64, NetWidgetState>,
}

impl NetState {
    pub fn init(widget_states: HashMap<u64, NetWidgetState>) -> Self {
        NetState { widget_states }
    }
}

pub struct CpuState {
    pub widget_states: HashMap<u64, CpuWidgetState>,
}

impl CpuState {
    pub fn init(widget_states: HashMap<u64, CpuWidgetState>) -> Self {
        CpuState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut CpuWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&CpuWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

pub struct MemState {
    pub widget_states: HashMap<u64, MemWidgetState>,
}

impl MemState {
    pub fn init(widget_states: HashMap<u64, MemWidgetState>) -> Self {
        MemState { widget_states }
    }
}

pub struct TempState {
    pub widget_states: HashMap<u64, TempWidgetState>,
}

impl TempState {
    pub fn init(widget_states: HashMap<u64, TempWidgetState>) -> Self {
        TempState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut TempWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&TempWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

pub struct DiskState {
    pub widget_states: HashMap<u64, DiskTableWidget>,
}

impl DiskState {
    pub fn init(widget_states: HashMap<u64, DiskTableWidget>) -> Self {
        DiskState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut DiskTableWidget> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&DiskTableWidget> {
        self.widget_states.get(&widget_id)
    }
}
pub struct BasicTableWidgetState {
    // Since this is intended (currently) to only be used for ONE widget, that's
    // how it's going to be written.  If we want to allow for multiple of these,
    // then we can expand outwards with a normal BasicTableState and a hashmap
    pub currently_displayed_widget_type: BottomWidgetType,
    pub currently_displayed_widget_id: u64,
    pub left_tlc: Option<(u16, u16)>,
    pub left_brc: Option<(u16, u16)>,
    pub right_tlc: Option<(u16, u16)>,
    pub right_brc: Option<(u16, u16)>,
}

pub struct AppBatteryState {
    pub widget_states: HashMap<u64, BatteryWidgetState>,
}

impl AppBatteryState {
    pub fn init(widget_states: HashMap<u64, BatteryWidgetState>) -> Self {
        AppBatteryState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut BatteryWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }
}

#[derive(Default)]
pub struct ParagraphScrollState {
    pub current_scroll_index: u16,
    pub max_scroll_index: u16,
}

#[cfg(test)]
mod test {
    use super::*;

    fn move_right(state: &mut AppSearchState) {
        state.walk_forward();
        state.cursor_direction = CursorDirection::Right;
    }

    fn move_left(state: &mut AppSearchState) {
        state.walk_backward();
        state.cursor_direction = CursorDirection::Left;
    }

    #[test]
    fn search_cursor_moves() {
        let mut state = AppSearchState::default();
        state.current_search_query = "Hi, 你好! 🇦🇶".to_string();
        state.grapheme_cursor = GraphemeCursor::new(0, state.current_search_query.len(), true);
        state.update_sizes();

        // Moving right.
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 0);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 1);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 2);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 3);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 4);
        assert_eq!(state.display_start_char_index, 2);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 7);
        assert_eq!(state.display_start_char_index, 4);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 10);
        assert_eq!(state.display_start_char_index, 7);

        move_right(&mut state);
        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 12);
        assert_eq!(state.display_start_char_index, 10);

        // Moving left.
        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 11);
        assert_eq!(state.display_start_char_index, 10);

        move_left(&mut state);
        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 7);
        assert_eq!(state.display_start_char_index, 7);

        move_left(&mut state);
        move_left(&mut state);
        move_left(&mut state);
        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 1);
        assert_eq!(state.display_start_char_index, 1);

        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 0);
        assert_eq!(state.display_start_char_index, 0);
    }
}
