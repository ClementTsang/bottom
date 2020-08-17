use std::{collections::HashMap, time::Instant};

use unicode_segmentation::GraphemeCursor;

use tui::widgets::TableState;

use crate::{
    app::{layout_manager::BottomWidgetType, query::*},
    constants,
    data_harvester::processes::{self, ProcessSorting},
};

#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    Up,
    // DOWN means scrolling down --- this usually INCREMENTS
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}

#[derive(Debug)]
pub enum CursorDirection {
    Left,
    Right,
}

/// AppScrollWidgetState deals with fields for a scrollable app's current state.
#[derive(Default)]
pub struct AppScrollWidgetState {
    pub current_scroll_position: usize,
    pub previous_scroll_position: usize,
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
            cursor_direction: CursorDirection::Right,
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

pub struct ColumnInfo {
    pub enabled: bool,
    pub shortcut: Option<&'static str>,
}

pub struct ProcColumn {
    pub ordered_columns: Vec<ProcessSorting>,
    pub column_mapping: HashMap<ProcessSorting, ColumnInfo>,
    pub longest_header_len: u16,
    pub column_state: TableState,
    pub scroll_direction: ScrollDirection,
    pub current_scroll_position: usize,
    pub previous_scroll_position: usize,
    pub backup_prev_scroll_position: usize,
}

impl Default for ProcColumn {
    fn default() -> Self {
        use ProcessSorting::*;
        let ordered_columns = vec![
            Pid,
            ProcessName,
            Command,
            CpuPercent,
            Mem,
            MemPercent,
            ReadPerSecond,
            WritePerSecond,
            TotalRead,
            TotalWrite,
            State,
        ];

        let mut column_mapping = HashMap::new();
        let mut longest_header_len = 0;
        for column in ordered_columns.clone() {
            longest_header_len = std::cmp::max(longest_header_len, column.to_string().len());
            match column {
                CpuPercent => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("c"),
                        },
                    );
                }
                MemPercent => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("m"),
                        },
                    );
                }
                Mem => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: false,
                            shortcut: Some("m"),
                        },
                    );
                }
                ProcessName => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("n"),
                        },
                    );
                }
                Command => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: false,
                            shortcut: Some("n"),
                        },
                    );
                }
                Pid => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("p"),
                        },
                    );
                }
                _ => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: None,
                        },
                    );
                }
            }
        }
        let longest_header_len = longest_header_len as u16;

        ProcColumn {
            ordered_columns,
            column_mapping,
            longest_header_len,
            column_state: TableState::default(),
            scroll_direction: ScrollDirection::default(),
            current_scroll_position: 0,
            previous_scroll_position: 0,
            backup_prev_scroll_position: 0,
        }
    }
}

impl ProcColumn {
    pub fn toggle(&mut self, column: &ProcessSorting) {
        if let Some(mapping) = self.column_mapping.get_mut(column) {
            mapping.enabled = !(mapping.enabled);
        }
    }

    pub fn is_enabled(&self, column: &ProcessSorting) -> bool {
        if let Some(mapping) = self.column_mapping.get(column) {
            mapping.enabled
        } else {
            false
        }
    }

    pub fn get_enabled_columns_len(&self) -> usize {
        self.ordered_columns
            .iter()
            .filter_map(|column_type| {
                if self.column_mapping.get(&column_type).unwrap().enabled {
                    Some(1)
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn set_to_sorted_index(&mut self, proc_sorting_type: &ProcessSorting) {
        // TODO [Custom Columns]: If we add custom columns, this may be needed!  Since column indices will change, this runs the risk of OOB.  So, when you change columns, CALL THIS AND ADAPT!
        let mut true_index = 0;
        for column in &self.ordered_columns {
            if *column == *proc_sorting_type {
                break;
            }
            if self.column_mapping.get(column).unwrap().enabled {
                true_index += 1;
            }
        }

        self.current_scroll_position = true_index;
        self.backup_prev_scroll_position = self.previous_scroll_position;
    }

    pub fn get_column_headers(
        &self, proc_sorting_type: &ProcessSorting, sort_reverse: bool,
    ) -> Vec<String> {
        // TODO: Gonna have to figure out how to do left/right GUI notation if we add it.
        self.ordered_columns
            .iter()
            .filter_map(|column_type| {
                let mapping = self.column_mapping.get(&column_type).unwrap();
                let mut command_str = String::default();
                if let Some(command) = mapping.shortcut {
                    command_str = format!("({})", command);
                }

                if mapping.enabled {
                    Some(if proc_sorting_type == column_type {
                        column_type.to_string()
                            + command_str.as_str()
                            + if sort_reverse { "▼" } else { "▲" }
                    } else {
                        column_type.to_string() + command_str.as_str()
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct ProcWidgetState {
    pub process_search_state: ProcessSearchState,
    pub is_grouped: bool,
    pub scroll_state: AppScrollWidgetState,
    pub process_sorting_type: processes::ProcessSorting,
    pub process_sorting_reverse: bool,
    pub is_using_command: bool,
    pub current_column_index: usize,
    pub is_sort_open: bool,
    pub columns: ProcColumn,
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

        let process_sorting_type = processes::ProcessSorting::CpuPercent;

        // TODO: If we add customizable columns, this should pull from config
        let mut columns = ProcColumn::default();
        columns.set_to_sorted_index(&process_sorting_type);

        ProcWidgetState {
            process_search_state,
            is_grouped,
            scroll_state: AppScrollWidgetState::default(),
            process_sorting_type,
            process_sorting_reverse: true,
            is_using_command: false,
            current_column_index: 0,
            is_sort_open: false,
            columns,
        }
    }

    /// Updates sorting when using the column list.
    /// ...this really should be part of the ProcColumn struct (along with the sorting fields),
    /// but I'm too lazy.
    ///
    /// Sorry, future me, you're gonna have to refactor this later.  Too busy getting
    /// the feature to work in the first place!  :)
    pub fn update_sorting_with_columns(&mut self) {
        let mut true_index = 0;
        let mut enabled_index = 0;
        let target_itx = self.columns.current_scroll_position;
        for column in &self.columns.ordered_columns {
            let enabled = self.columns.column_mapping.get(column).unwrap().enabled;
            if enabled_index == target_itx && enabled {
                break;
            }
            if enabled {
                enabled_index += 1;
            }
            true_index += 1;
        }

        if let Some(new_sort_type) = self.columns.ordered_columns.get(true_index) {
            if *new_sort_type == self.process_sorting_type {
                // Just reverse the search if we're reselecting!
                self.process_sorting_reverse = !(self.process_sorting_reverse);
            } else {
                self.process_sorting_type = new_sort_type.clone();
                match self.process_sorting_type {
                    ProcessSorting::State
                    | ProcessSorting::Pid
                    | ProcessSorting::ProcessName
                    | ProcessSorting::Command => {
                        // Also invert anything that uses alphabetical sorting by default.
                        self.process_sorting_reverse = false;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn toggle_command_and_name(&mut self, is_using_command: bool) {
        self.columns
            .column_mapping
            .get_mut(&ProcessSorting::ProcessName)
            .unwrap()
            .enabled = !is_using_command;
        self.columns
            .column_mapping
            .get_mut(&ProcessSorting::Command)
            .unwrap()
            .enabled = is_using_command;
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
            // debug!("Parsed query: {:#?}", parsed_query);

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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut ProcWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&ProcWidgetState> {
        self.widget_states.get(&widget_id)
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut NetWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&NetWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

pub struct CpuWidgetState {
    pub current_display_time: u64,
    pub is_legend_hidden: bool,
    pub autohide_timer: Option<Instant>,
    pub scroll_state: AppScrollWidgetState,
    pub is_multi_graph_mode: bool,
}

impl CpuWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            autohide_timer,
            scroll_state: AppScrollWidgetState::default(),
            is_multi_graph_mode: false,
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut CpuWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&CpuWidgetState> {
        self.widget_states.get(&widget_id)
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut MemWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&MemWidgetState> {
        self.widget_states.get(&widget_id)
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut TempWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&TempWidgetState> {
        self.widget_states.get(&widget_id)
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut DiskWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&DiskWidgetState> {
        self.widget_states.get(&widget_id)
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut BatteryWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&BatteryWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

#[derive(Default)]
pub struct ParagraphScrollState {
    pub current_scroll_index: u16,
    pub max_scroll_index: u16,
}
