use std::{collections::HashMap, time::Instant};

use unicode_segmentation::GraphemeCursor;

use tui::widgets::TableState;

use crate::{
    app::{layout_manager::BottomWidgetType, query::*},
    constants,
    data_harvester::processes,
};

#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    UP,
    // DOWN means scrolling down --- this usually INCREMENTS
    DOWN,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::DOWN
    }
}

#[derive(Debug)]
pub enum CursorDirection {
    LEFT,
    RIGHT,
}

/// AppScrollWidgetState deals with fields for a scrollable app's current state.
#[derive(Default)]
pub struct AppScrollWidgetState {
    pub current_scroll_position: u64,
    pub previous_scroll_position: u64,
    pub scroll_direction: ScrollDirection,
    pub table_state: TableState,
}

#[derive(Default)]
pub struct AppDeleteDialogState {
    pub is_showing_dd: bool,
    pub is_on_yes: bool, // Defaults to "No"
}

pub struct AppHelpDialogState {
    pub is_showing_help: bool,
    pub scroll_state: ParagraphScrollState,
    pub index_shortcuts: Vec<u16>,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
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
    pub cursor_bar: usize,
    /// This represents the position in terms of CHARACTERS, not graphemes
    pub char_cursor_position: usize,
    /// The query
    pub query: Option<Query>,
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
            cursor_direction: CursorDirection::RIGHT,
            cursor_bar: 0,
            char_cursor_position: 0,
            query: None,
            error_message: None,
        }
    }
}

impl AppSearchState {
    /// Returns a reset but still enabled app search state
    pub fn reset(&mut self) {
        *self = AppSearchState {
            is_enabled: self.is_enabled,
            ..AppSearchState::default()
        }
    }

    pub fn is_invalid_or_blank_search(&self) -> bool {
        self.is_blank_search || self.is_invalid_search
    }
}

/// ProcessSearchState only deals with process' search's current settings and state.
pub struct ProcessSearchState {
    pub search_state: AppSearchState,
    pub is_ignoring_case: bool,
    pub is_searching_whole_word: bool,
    pub is_searching_with_regex: bool,
}

impl Default for ProcessSearchState {
    fn default() -> Self {
        ProcessSearchState {
            search_state: AppSearchState::default(),
            is_ignoring_case: true,
            is_searching_whole_word: false,
            is_searching_with_regex: false,
        }
    }
}

impl ProcessSearchState {
    pub fn search_toggle_ignore_case(&mut self) {
        self.is_ignoring_case = !self.is_ignoring_case;
    }

    pub fn search_toggle_whole_word(&mut self) {
        self.is_searching_whole_word = !self.is_searching_whole_word;
    }

    pub fn search_toggle_regex(&mut self) {
        self.is_searching_with_regex = !self.is_searching_with_regex;
    }
}

pub struct ProcWidgetState {
    pub process_search_state: ProcessSearchState,
    pub is_grouped: bool,
    pub scroll_state: AppScrollWidgetState,
    pub process_sorting_type: processes::ProcessSorting,
    pub process_sorting_reverse: bool,
}

impl ProcWidgetState {
    pub fn init(
        is_case_sensitive: bool, is_match_whole_word: bool, is_use_regex: bool, is_grouped: bool,
    ) -> Self {
        let mut process_search_state = ProcessSearchState::default();
        if is_case_sensitive {
            // By default it's off
            process_search_state.search_toggle_ignore_case();
        }
        if is_match_whole_word {
            process_search_state.search_toggle_whole_word();
        }
        if is_use_regex {
            process_search_state.search_toggle_regex();
        }

        ProcWidgetState {
            process_search_state,
            is_grouped,
            scroll_state: AppScrollWidgetState::default(),
            process_sorting_type: processes::ProcessSorting::CPU,
            process_sorting_reverse: true,
        }
    }

    pub fn get_cursor_position(&self) -> usize {
        self.process_search_state
            .search_state
            .grapheme_cursor
            .cur_cursor()
    }

    pub fn get_char_cursor_position(&self) -> usize {
        self.process_search_state.search_state.char_cursor_position
    }

    pub fn is_search_enabled(&self) -> bool {
        self.process_search_state.search_state.is_enabled
    }

    pub fn get_current_search_query(&self) -> &String {
        &self.process_search_state.search_state.current_search_query
    }

    pub fn update_query(&mut self) {
        if self
            .process_search_state
            .search_state
            .current_search_query
            .is_empty()
        {
            self.process_search_state.search_state.is_blank_search = true;
            self.process_search_state.search_state.is_invalid_search = false;
            self.process_search_state.search_state.error_message = None;
        } else {
            let parsed_query = self.parse_query();
            // debug!("PQ: {:#?}", parsed_query);

            if let Ok(parsed_query) = parsed_query {
                self.process_search_state.search_state.query = Some(parsed_query);
                self.process_search_state.search_state.is_blank_search = false;
                self.process_search_state.search_state.is_invalid_search = false;
                self.process_search_state.search_state.error_message = None;
            } else if let Err(err) = parsed_query {
                self.process_search_state.search_state.is_blank_search = false;
                self.process_search_state.search_state.is_invalid_search = true;
                self.process_search_state.search_state.error_message = Some(err.to_string());
            }
        }
        self.scroll_state.previous_scroll_position = 0;
        self.scroll_state.current_scroll_position = 0;
    }

    pub fn clear_search(&mut self) {
        self.process_search_state.search_state.reset();
    }

    pub fn search_walk_forward(&mut self, start_position: usize) {
        self.process_search_state
            .search_state
            .grapheme_cursor
            .next_boundary(
                &self.process_search_state.search_state.current_search_query[start_position..],
                start_position,
            )
            .unwrap();
    }

    pub fn search_walk_back(&mut self, start_position: usize) {
        self.process_search_state
            .search_state
            .grapheme_cursor
            .prev_boundary(
                &self.process_search_state.search_state.current_search_query[..start_position],
                0,
            )
            .unwrap();
    }
}

pub struct ProcState {
    pub widget_states: HashMap<u64, ProcWidgetState>,
    pub force_update: Option<u64>,
    pub force_update_all: bool,
}

impl ProcState {
    pub fn init(widget_states: HashMap<u64, ProcWidgetState>) -> Self {
        ProcState {
            widget_states,
            force_update: None,
            force_update_all: false,
        }
    }
}

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}

pub struct NetState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, NetWidgetState>,
}

impl NetState {
    pub fn init(widget_states: HashMap<u64, NetWidgetState>) -> Self {
        NetState {
            force_update: None,
            widget_states,
        }
    }
}

pub struct CpuWidgetState {
    pub current_display_time: u64,
    pub is_legend_hidden: bool,
    pub autohide_timer: Option<Instant>,
    pub scroll_state: AppScrollWidgetState,
}

impl CpuWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            autohide_timer,
            scroll_state: AppScrollWidgetState::default(),
        }
    }
}

pub struct CpuState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, CpuWidgetState>,
}

impl CpuState {
    pub fn init(widget_states: HashMap<u64, CpuWidgetState>) -> Self {
        CpuState {
            force_update: None,
            widget_states,
        }
    }
}

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl MemWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}

pub struct MemState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, MemWidgetState>,
}

impl MemState {
    pub fn init(widget_states: HashMap<u64, MemWidgetState>) -> Self {
        MemState {
            force_update: None,
            widget_states,
        }
    }
}

pub struct TempWidgetState {
    pub scroll_state: AppScrollWidgetState,
}

impl TempWidgetState {
    pub fn init() -> Self {
        TempWidgetState {
            scroll_state: AppScrollWidgetState::default(),
        }
    }
}

pub struct TempState {
    pub widget_states: HashMap<u64, TempWidgetState>,
}

impl TempState {
    pub fn init(widget_states: HashMap<u64, TempWidgetState>) -> Self {
        TempState { widget_states }
    }
}

pub struct DiskWidgetState {
    pub scroll_state: AppScrollWidgetState,
}

impl DiskWidgetState {
    pub fn init() -> Self {
        DiskWidgetState {
            scroll_state: AppScrollWidgetState::default(),
        }
    }
}

pub struct DiskState {
    pub widget_states: HashMap<u64, DiskWidgetState>,
}

impl DiskState {
    pub fn init(widget_states: HashMap<u64, DiskWidgetState>) -> Self {
        DiskState { widget_states }
    }
}
pub struct BasicTableWidgetState {
    // Since this is intended (currently) to only be used for ONE widget, that's
    // how it's going to be written.  If we want to allow for multiple of these,
    // then we can expand outwards with a normal BasicTableState and a hashmap
    pub currently_displayed_widget_type: BottomWidgetType,
    pub currently_displayed_widget_id: u64,
    pub widget_id: i64,
}

#[derive(Default)]
pub struct BatteryWidgetState {
    pub currently_selected_battery_index: usize,
}

pub struct BatteryState {
    pub widget_states: HashMap<u64, BatteryWidgetState>,
}

impl BatteryState {
    pub fn init(widget_states: HashMap<u64, BatteryWidgetState>) -> Self {
        BatteryState { widget_states }
    }
}

#[derive(Default)]
pub struct ParagraphScrollState {
    pub current_scroll_index: u16,
    pub max_scroll_index: u16,
}
