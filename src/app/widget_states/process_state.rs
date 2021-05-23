use std::collections::HashMap;

use unicode_segmentation::GraphemeCursor;

use tui::widgets::TableState;

use crate::{
    app::query::*,
    data_harvester::processes::{self, ProcessSorting},
};
use ProcessSorting::*;

use super::{AppScrollWidgetState, CanvasTableWidthState, CursorDirection, ScrollDirection};

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
    // FIXME: Move column width logic here!
    // pub hard_width: Option<u16>,
    // pub max_soft_width: Option<f64>,
}

pub struct ProcColumn {
    pub ordered_columns: Vec<ProcessSorting>,
    /// The y location of headers.  Since they're all aligned, it's just one value.
    pub column_header_y_loc: Option<u16>,
    /// The x start and end bounds for each header.
    pub column_header_x_locs: Option<Vec<(u16, u16)>>,
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
        let ordered_columns = vec![
            Count,
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
            User,
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
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                MemPercent => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("m"),
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                Mem => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: false,
                            shortcut: Some("m"),
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                ProcessName => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("n"),
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                Command => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: false,
                            shortcut: Some("n"),
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                Pid => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: Some("p"),
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                Count => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: false,
                            shortcut: None,
                            // hard_width: None,
                            // max_soft_width: None,
                        },
                    );
                }
                User => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: cfg!(target_family = "unix"),
                            shortcut: None,
                        },
                    );
                }
                _ => {
                    column_mapping.insert(
                        column,
                        ColumnInfo {
                            enabled: true,
                            shortcut: None,
                            // hard_width: None,
                            // max_soft_width: None,
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
            column_header_y_loc: None,
            column_header_x_locs: None,
        }
    }
}

impl ProcColumn {
    /// Returns its new status.
    pub fn toggle(&mut self, column: &ProcessSorting) -> Option<bool> {
        if let Some(mapping) = self.column_mapping.get_mut(column) {
            mapping.enabled = !(mapping.enabled);
            Some(mapping.enabled)
        } else {
            None
        }
    }

    pub fn try_set(&mut self, column: &ProcessSorting, setting: bool) -> Option<bool> {
        if let Some(mapping) = self.column_mapping.get_mut(column) {
            mapping.enabled = setting;
            Some(mapping.enabled)
        } else {
            None
        }
    }

    pub fn try_enable(&mut self, column: &ProcessSorting) -> Option<bool> {
        if let Some(mapping) = self.column_mapping.get_mut(column) {
            mapping.enabled = true;
            Some(mapping.enabled)
        } else {
            None
        }
    }

    pub fn try_disable(&mut self, column: &ProcessSorting) -> Option<bool> {
        if let Some(mapping) = self.column_mapping.get_mut(column) {
            mapping.enabled = false;
            Some(mapping.enabled)
        } else {
            None
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
                if let Some(col_map) = self.column_mapping.get(&column_type) {
                    if col_map.enabled {
                        Some(1)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .sum()
    }

    /// NOTE: ALWAYS call this when opening the sorted window.
    pub fn set_to_sorted_index_from_type(&mut self, proc_sorting_type: &ProcessSorting) {
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

    /// This function sets the scroll position based on the index.
    pub fn set_to_sorted_index_from_visual_index(&mut self, visual_index: usize) {
        self.current_scroll_position = visual_index;
        self.backup_prev_scroll_position = self.previous_scroll_position;
    }

    pub fn get_column_headers(
        &self, proc_sorting_type: &ProcessSorting, sort_reverse: bool,
    ) -> Vec<String> {
        const DOWN_ARROW: char = '▼';
        const UP_ARROW: char = '▲';

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
                    Some(format!(
                        "{}{}{}",
                        column_type.to_string(),
                        command_str.as_str(),
                        if proc_sorting_type == column_type {
                            if sort_reverse {
                                DOWN_ARROW
                            } else {
                                UP_ARROW
                            }
                        } else {
                            ' '
                        }
                    ))
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
    pub is_process_sort_descending: bool,
    pub is_using_command: bool,
    pub current_column_index: usize,
    pub is_sort_open: bool,
    pub columns: ProcColumn,
    pub is_tree_mode: bool,
    pub table_width_state: CanvasTableWidthState,
    pub requires_redraw: bool,
}

impl ProcWidgetState {
    pub fn init(
        is_case_sensitive: bool, is_match_whole_word: bool, is_use_regex: bool, is_grouped: bool,
        show_memory_as_values: bool, is_tree_mode: bool, is_using_command: bool,
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

        let (process_sorting_type, is_process_sort_descending) = if is_tree_mode {
            (processes::ProcessSorting::Pid, false)
        } else {
            (processes::ProcessSorting::CpuPercent, true)
        };

        // TODO: If we add customizable columns, this should pull from config
        let mut columns = ProcColumn::default();
        columns.set_to_sorted_index_from_type(&process_sorting_type);
        if is_grouped {
            // Normally defaults to showing by PID, toggle count on instead.
            columns.toggle(&ProcessSorting::Count);
            columns.toggle(&ProcessSorting::Pid);
        }
        if show_memory_as_values {
            // Normally defaults to showing by percent, toggle value on instead.
            columns.toggle(&ProcessSorting::Mem);
            columns.toggle(&ProcessSorting::MemPercent);
        }

        ProcWidgetState {
            process_search_state,
            is_grouped,
            scroll_state: AppScrollWidgetState::default(),
            process_sorting_type,
            is_process_sort_descending,
            is_using_command,
            current_column_index: 0,
            is_sort_open: false,
            columns,
            is_tree_mode,
            table_width_state: CanvasTableWidthState::default(),
            requires_redraw: false,
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
                self.is_process_sort_descending = !(self.is_process_sort_descending);
            } else {
                self.process_sorting_type = new_sort_type.clone();
                match self.process_sorting_type {
                    ProcessSorting::State
                    | ProcessSorting::Pid
                    | ProcessSorting::ProcessName
                    | ProcessSorting::Command => {
                        // Also invert anything that uses alphabetical sorting by default.
                        self.is_process_sort_descending = false;
                    }
                    _ => {
                        self.is_process_sort_descending = true;
                    }
                }
            }
        }
    }

    pub fn toggle_command_and_name(&mut self, is_using_command: bool) {
        if let Some(pn) = self
            .columns
            .column_mapping
            .get_mut(&ProcessSorting::ProcessName)
        {
            pn.enabled = !is_using_command;
        }
        if let Some(c) = self
            .columns
            .column_mapping
            .get_mut(&ProcessSorting::Command)
        {
            c.enabled = is_using_command;
        }
    }

    pub fn get_search_cursor_position(&self) -> usize {
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
