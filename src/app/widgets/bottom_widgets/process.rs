use std::{borrow::Cow, collections::HashMap};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use float_ord::FloatOrd;
use itertools::{Either, Itertools};
use unicode_segmentation::GraphemeCursor;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Borders, TableState},
    Frame,
};

use crate::{
    app::{
        data_harvester::processes::ProcessHarvest,
        event::{ComponentEventResult, MultiKey, MultiKeyResult, ReturnSignal, SelectionAction},
        query::*,
        text_table::DesiredColumnWidth,
        widgets::tui_stuff::BlockBuilder,
        DataCollection,
    },
    canvas::Painter,
    data_conversion::get_string_with_bytes,
    data_harvester::processes::{self, ProcessSorting},
    options::{layout_options::LayoutRule, ProcessDefaults},
    utils::error::BottomError,
};
use ProcessSorting::*;

use crate::app::{
    does_bound_intersect_coordinate,
    sort_text_table::{SimpleSortableColumn, SortStatus, SortableColumn},
    text_table::TextTableData,
    AppScrollWidgetState, CanvasTableWidthState, Component, CursorDirection, ScrollDirection,
    SortMenu, SortableTextTable, TextInput, Widget,
};

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
                if let Some(col_map) = self.column_mapping.get(column_type) {
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
                let mapping = self.column_mapping.get(column_type).unwrap();
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
            let parsed_query = parse_query(
                self.get_current_search_query(),
                self.process_search_state.is_searching_whole_word,
                self.process_search_state.is_ignoring_case,
                self.process_search_state.is_searching_with_regex,
            );
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

#[derive(Default)]
pub struct ProcState {
    pub widget_states: HashMap<u64, ProcWidgetState>,
    pub force_update: Option<u64>,
    pub force_update_all: bool,
}

impl ProcState {
    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut ProcWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&ProcWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

/// The currently selected part of a [`ProcessManager`]
#[derive(PartialEq, Eq, Clone, Copy)]
enum ProcessManagerSelection {
    Processes,
    Sort,
    Search,
}

#[derive(Default)]
/// The state of the search modifiers.
struct SearchModifiers {
    enable_case_sensitive: bool,
    enable_whole_word: bool,
    enable_regex: bool,
}

enum FlexColumn {
    Flex(f64),
    Hard(Option<u16>),
}

pub enum ProcessSortType {
    Pid,
    Count,
    Name,
    Command,
    Cpu,
    Mem,
    MemPercent,
    Rps,
    Wps,
    TotalRead,
    TotalWrite,
    User,
    State,
}

impl ProcessSortType {
    fn to_str(&self) -> &'static str {
        match self {
            ProcessSortType::Pid => "PID",
            ProcessSortType::Count => "Count",
            ProcessSortType::Name => "Name",
            ProcessSortType::Command => "Command",
            ProcessSortType::Cpu => "CPU%",
            ProcessSortType::Mem => "Mem",
            ProcessSortType::MemPercent => "Mem%",
            ProcessSortType::Rps => "R/s",
            ProcessSortType::Wps => "W/s",
            ProcessSortType::TotalRead => "T.Read",
            ProcessSortType::TotalWrite => "T.Write",
            ProcessSortType::User => "User",
            ProcessSortType::State => "State",
        }
    }

    fn shortcut(&self) -> Option<KeyEvent> {
        match self {
            ProcessSortType::Pid => Some(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)),
            ProcessSortType::Count => None,
            ProcessSortType::Name => Some(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
            ProcessSortType::Command => None,
            ProcessSortType::Cpu => Some(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            ProcessSortType::Mem => Some(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)),
            ProcessSortType::MemPercent => {
                Some(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE))
            }
            ProcessSortType::Rps => None,
            ProcessSortType::Wps => None,
            ProcessSortType::TotalRead => None,
            ProcessSortType::TotalWrite => None,
            ProcessSortType::User => None,
            ProcessSortType::State => None,
        }
    }

    fn column_type(&self) -> FlexColumn {
        use FlexColumn::*;

        match self {
            ProcessSortType::Pid => Hard(Some(7)),
            ProcessSortType::Count => Hard(Some(8)),
            ProcessSortType::Name => Flex(0.3),
            ProcessSortType::Command => Flex(0.7),
            ProcessSortType::Cpu => Hard(Some(8)),
            ProcessSortType::Mem => Hard(Some(8)),
            ProcessSortType::MemPercent => Hard(Some(8)),
            ProcessSortType::Rps => Hard(Some(8)),
            ProcessSortType::Wps => Hard(Some(8)),
            ProcessSortType::TotalRead => Hard(Some(7)),
            ProcessSortType::TotalWrite => Hard(Some(8)),
            ProcessSortType::User => Flex(0.08),
            ProcessSortType::State => Hard(Some(8)),
        }
    }

    fn default_descending(&self) -> bool {
        match self {
            ProcessSortType::Pid => false,
            ProcessSortType::Count => true,
            ProcessSortType::Name => false,
            ProcessSortType::Command => false,
            ProcessSortType::Cpu => true,
            ProcessSortType::Mem => true,
            ProcessSortType::MemPercent => true,
            ProcessSortType::Rps => true,
            ProcessSortType::Wps => true,
            ProcessSortType::TotalRead => true,
            ProcessSortType::TotalWrite => true,
            ProcessSortType::User => false,
            ProcessSortType::State => false,
        }
    }
}

/// A thin wrapper around a [`SortableColumn`] to help keep track of
/// how to sort given a chosen column.
pub struct ProcessSortColumn {
    /// The underlying column.
    sortable_column: SimpleSortableColumn,

    /// The *type* of column. Useful for determining how to sort.
    sort_type: ProcessSortType,
}

impl ProcessSortColumn {
    pub fn new(sort_type: ProcessSortType) -> Self {
        let sortable_column = {
            let name = sort_type.to_str().into();
            let shortcut = sort_type.shortcut();
            let default_descending = sort_type.default_descending();

            match sort_type.column_type() {
                FlexColumn::Flex(max_percentage) => SimpleSortableColumn::new_flex(
                    name,
                    shortcut,
                    default_descending,
                    max_percentage,
                ),
                FlexColumn::Hard(hard_length) => {
                    SimpleSortableColumn::new_hard(name, shortcut, default_descending, hard_length)
                }
            }
        };

        Self {
            sortable_column,
            sort_type,
        }
    }
}

impl SortableColumn for ProcessSortColumn {
    fn original_name(&self) -> &Cow<'static, str> {
        self.sortable_column.original_name()
    }

    fn shortcut(&self) -> &Option<(KeyEvent, String)> {
        self.sortable_column.shortcut()
    }

    fn default_descending(&self) -> bool {
        self.sortable_column.default_descending()
    }

    fn sorting_status(&self) -> SortStatus {
        self.sortable_column.sorting_status()
    }

    fn set_sorting_status(&mut self, sorting_status: SortStatus) {
        self.sortable_column.set_sorting_status(sorting_status)
    }

    fn display_name(&self) -> Cow<'static, str> {
        self.sortable_column.display_name()
    }

    fn get_desired_width(&self) -> &DesiredColumnWidth {
        self.sortable_column.get_desired_width()
    }

    fn get_x_bounds(&self) -> Option<(u16, u16)> {
        self.sortable_column.get_x_bounds()
    }

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>) {
        self.sortable_column.set_x_bounds(x_bounds)
    }
}

/// A searchable, sortable table to manage processes.
pub struct ProcessManager {
    bounds: Rect,
    process_table: SortableTextTable<ProcessSortColumn>,
    sort_menu: SortMenu,
    search_block_bounds: Rect,

    search_input: TextInput,

    dd_multi: MultiKey,

    selected: ProcessManagerSelection,
    prev_selected: ProcessManagerSelection,

    in_tree_mode: bool,
    show_sort: bool,
    show_search: bool,

    search_modifiers: SearchModifiers,

    display_data: TextTableData,

    process_filter: Option<Result<Query, BottomError>>,

    block_border: Borders,

    width: LayoutRule,
    height: LayoutRule,

    show_scroll_index: bool,
}

impl ProcessManager {
    /// Creates a new [`ProcessManager`].
    pub fn new(process_defaults: &ProcessDefaults) -> Self {
        let process_table_columns = vec![
            ProcessSortColumn::new(ProcessSortType::Pid),
            ProcessSortColumn::new(ProcessSortType::Name),
            ProcessSortColumn::new(ProcessSortType::Cpu),
            ProcessSortColumn::new(ProcessSortType::MemPercent),
            ProcessSortColumn::new(ProcessSortType::Rps),
            ProcessSortColumn::new(ProcessSortType::Wps),
            ProcessSortColumn::new(ProcessSortType::TotalRead),
            ProcessSortColumn::new(ProcessSortType::TotalWrite),
            #[cfg(target_family = "unix")]
            ProcessSortColumn::new(ProcessSortType::User),
            ProcessSortColumn::new(ProcessSortType::State),
        ];

        let mut manager = Self {
            bounds: Rect::default(),
            sort_menu: SortMenu::new(process_table_columns.len()),
            process_table: SortableTextTable::new(process_table_columns).default_sort_index(2),
            search_input: TextInput::default(),
            search_block_bounds: Rect::default(),
            dd_multi: MultiKey::register(vec!['d', 'd']), // TODO: Maybe use something static...
            selected: ProcessManagerSelection::Processes,
            prev_selected: ProcessManagerSelection::Processes,
            in_tree_mode: false,
            show_sort: false,
            show_search: false,
            search_modifiers: SearchModifiers::default(),
            display_data: Default::default(),
            process_filter: None,
            block_border: Borders::ALL,
            width: LayoutRule::default(),
            height: LayoutRule::default(),
            show_scroll_index: false,
        };

        manager.set_tree_mode(process_defaults.is_tree);
        manager
    }

    /// Sets the block border style.
    pub fn basic_mode(mut self, basic_mode: bool) -> Self {
        if basic_mode {
            self.block_border = *crate::constants::SIDE_BORDERS;
        }

        self
    }

    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: LayoutRule) -> Self {
        self.height = height;
        self
    }

    fn set_tree_mode(&mut self, in_tree_mode: bool) {
        self.in_tree_mode = in_tree_mode;
    }

    /// Sets whether to show the scroll index.
    pub fn show_scroll_index(mut self, show_scroll_index: bool) -> Self {
        self.show_scroll_index = show_scroll_index;
        self
    }

    fn open_search(&mut self) -> ComponentEventResult {
        if let ProcessManagerSelection::Search = self.selected {
            ComponentEventResult::NoRedraw
        } else {
            self.show_search = true;
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Search;
            ComponentEventResult::Redraw
        }
    }

    fn open_sort(&mut self) -> ComponentEventResult {
        if let ProcessManagerSelection::Sort = self.selected {
            ComponentEventResult::NoRedraw
        } else {
            self.sort_menu
                .set_index(self.process_table.current_sorting_column_index());
            self.show_sort = true;
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Sort;
            ComponentEventResult::Redraw
        }
    }

    /// Returns whether the process manager is searching the current term with the restriction that it must
    /// match entire word.
    pub fn is_searching_whole_word(&self) -> bool {
        self.search_modifiers.enable_whole_word
    }

    /// Returns whether the process manager is searching the current term using regex.
    pub fn is_searching_with_regex(&self) -> bool {
        self.search_modifiers.enable_regex
    }

    /// Returns whether the process manager is searching the current term with the restriction that case-sensitivity
    /// matters.
    pub fn is_case_sensitive(&self) -> bool {
        self.search_modifiers.enable_case_sensitive
    }

    fn is_using_command(&self) -> bool {
        matches!(
            self.process_table.columns()[1].sort_type,
            ProcessSortType::Command
        )
    }

    fn toggle_command(&mut self) -> ComponentEventResult {
        if self.is_using_command() {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Name), 1);
        } else {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Command), 1);
        }

        // Invalidate row cache.
        self.process_table.invalidate_cached_columns();

        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    fn is_grouped(&self) -> bool {
        matches!(
            self.process_table.columns()[0].sort_type,
            ProcessSortType::Count
        )
    }

    fn toggle_grouped(&mut self) -> ComponentEventResult {
        if self.is_grouped() {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Pid), 0);

            self.process_table
                .add_column(ProcessSortColumn::new(ProcessSortType::State), 8);
            #[cfg(target_family = "unix")]
            {
                self.process_table
                    .add_column(ProcessSortColumn::new(ProcessSortType::User), 8);
            }
        } else {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Count), 0);

            #[cfg(target_family = "unix")]
            {
                self.process_table.remove_column(9, Some(2));
            }
            self.process_table.remove_column(8, Some(2));
        }

        // Invalidate row cache.
        self.process_table.invalidate_cached_columns();

        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    fn hide_sort(&mut self) {
        self.show_sort = false;
        if let ProcessManagerSelection::Sort = self.selected {
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Processes;
        }
    }

    fn hide_search(&mut self) {
        self.show_search = false;
        if let ProcessManagerSelection::Search = self.selected {
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Processes;
        }
    }
}

impl Component for ProcessManager {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        // "Global" handling:

        if let KeyCode::Esc = event.code {
            match self.selected {
                ProcessManagerSelection::Processes => {
                    if self.show_sort {
                        self.hide_sort();
                        return ComponentEventResult::Redraw;
                    } else if self.show_search {
                        self.hide_search();
                        return ComponentEventResult::Redraw;
                    }
                }
                ProcessManagerSelection::Sort if self.show_sort => {
                    self.hide_sort();
                    return ComponentEventResult::Redraw;
                }
                ProcessManagerSelection::Search if self.show_search => {
                    self.hide_search();
                    return ComponentEventResult::Redraw;
                }
                _ => {}
            }
        }

        match self.selected {
            ProcessManagerSelection::Processes => {
                // Try to catch some stuff first...
                if event.modifiers.is_empty() {
                    match event.code {
                        KeyCode::Tab => {
                            // Handle grouping/ungrouping
                            return self.toggle_grouped();
                        }
                        KeyCode::Char('P') => {
                            // Show full command/process name
                            return self.toggle_command();
                        }
                        KeyCode::Char('d') => {
                            match self.dd_multi.input('d') {
                                MultiKeyResult::Completed => {
                                    // Kill the selected process(es)
                                }
                                MultiKeyResult::Accepted | MultiKeyResult::Rejected => {
                                    return ComponentEventResult::NoRedraw;
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            return self.open_search();
                        }
                        KeyCode::Char('%') => {
                            // Handle switching memory usage type
                        }
                        KeyCode::Char('+') => {
                            // Expand a branch
                        }
                        KeyCode::Char('-') => {
                            // Collapse a branch
                        }
                        KeyCode::Char('t') | KeyCode::F(5) => {
                            self.in_tree_mode = !self.in_tree_mode;
                            return ComponentEventResult::Redraw;
                        }
                        KeyCode::Char('s') | KeyCode::F(6) => {
                            return self.open_sort();
                        }
                        KeyCode::F(9) => {
                            // Kill the selected process(es)
                        }
                        _ => {}
                    }
                } else if let KeyModifiers::CONTROL = event.modifiers {
                    if let KeyCode::Char('f') = event.code {
                        return self.open_search();
                    }
                } else if let KeyModifiers::SHIFT = event.modifiers {
                    if let KeyCode::Char('P') = event.code {
                        // Show full command/process name
                        return self.toggle_command();
                    }
                }

                self.process_table.handle_key_event(event)
            }
            ProcessManagerSelection::Sort => {
                if event.modifiers.is_empty() {
                    match event.code {
                        KeyCode::Enter => {
                            self.process_table
                                .set_sort_index(self.sort_menu.current_index());
                            return ComponentEventResult::Signal(ReturnSignal::Update);
                        }
                        KeyCode::Char('/') => {
                            return self.open_search();
                        }
                        _ => {}
                    }
                }

                self.sort_menu.handle_key_event(event)
            }
            ProcessManagerSelection::Search => {
                if event.modifiers.is_empty() {
                    match event.code {
                        KeyCode::F(1) => {}
                        KeyCode::F(2) => {}
                        KeyCode::F(3) => {}
                        _ => {}
                    }
                } else if let KeyModifiers::ALT = event.modifiers {
                    match event.code {
                        KeyCode::Char('c') | KeyCode::Char('C') => {}
                        KeyCode::Char('w') | KeyCode::Char('W') => {}
                        KeyCode::Char('r') | KeyCode::Char('R') => {}
                        _ => {}
                    }
                }

                let handle_output = self.search_input.handle_key_event(event);
                if let ComponentEventResult::Signal(ReturnSignal::Update) = handle_output {
                    self.process_filter = Some(parse_query(
                        self.search_input.query(),
                        self.is_searching_whole_word(),
                        !self.is_case_sensitive(),
                        self.is_searching_with_regex(),
                    ));
                }

                handle_output
            }
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        match &event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.process_table.does_border_intersect_mouse(&event) {
                    if let ProcessManagerSelection::Processes = self.selected {
                        self.process_table.handle_mouse_event(event)
                    } else {
                        self.prev_selected = self.selected;
                        self.selected = ProcessManagerSelection::Processes;
                        match self.process_table.handle_mouse_event(event) {
                            ComponentEventResult::Unhandled
                            | ComponentEventResult::Redraw
                            | ComponentEventResult::NoRedraw => ComponentEventResult::Redraw,
                            ComponentEventResult::Signal(s) => ComponentEventResult::Signal(s),
                        }
                    }
                } else if self.sort_menu.does_border_intersect_mouse(&event) {
                    if let ProcessManagerSelection::Sort = self.selected {
                        self.sort_menu.handle_mouse_event(event)
                    } else {
                        self.prev_selected = self.selected;
                        self.selected = ProcessManagerSelection::Sort;
                        self.sort_menu.handle_mouse_event(event);
                        ComponentEventResult::Redraw
                    }
                } else if does_bound_intersect_coordinate(
                    event.column,
                    event.row,
                    self.search_block_bounds,
                ) {
                    if let ProcessManagerSelection::Search = self.selected {
                        self.search_input.handle_mouse_event(event)
                    } else {
                        self.prev_selected = self.selected;
                        self.selected = ProcessManagerSelection::Search;
                        self.search_input.handle_mouse_event(event);
                        ComponentEventResult::Redraw
                    }
                } else {
                    ComponentEventResult::Unhandled
                }
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => match self.selected {
                ProcessManagerSelection::Processes => self.process_table.handle_mouse_event(event),
                ProcessManagerSelection::Sort => self.sort_menu.handle_mouse_event(event),
                ProcessManagerSelection::Search => self.search_input.handle_mouse_event(event),
            },
            _ => ComponentEventResult::Unhandled,
        }
    }
}

impl Widget for ProcessManager {
    fn get_pretty_name(&self) -> &'static str {
        "Processes"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
        expanded: bool,
    ) {
        let area = if self.show_search {
            let search_constraints: [Constraint; 2] = [
                Constraint::Min(0),
                if self.block_border.contains(Borders::TOP) {
                    Constraint::Length(5)
                } else {
                    Constraint::Length(3)
                },
            ];
            const INTERNAL_SEARCH_CONSTRAINTS: [Constraint; 2] = [Constraint::Length(1); 2];

            let vertical_split_area = Layout::default()
                .margin(0)
                .direction(Direction::Vertical)
                .constraints(search_constraints)
                .split(area);

            let is_search_selected =
                selected && matches!(self.selected, ProcessManagerSelection::Search);

            // TODO: [Redesign] this currently uses a separate box - maybe fold this into the main box?
            let search_block = BlockBuilder::new("")
                .selected(is_search_selected)
                .hide_title(true)
                .build(painter, vertical_split_area[1]);

            self.search_block_bounds = vertical_split_area[1];

            let internal_split_area = Layout::default()
                .margin(0)
                .direction(Direction::Vertical)
                .constraints(INTERNAL_SEARCH_CONSTRAINTS)
                .split(search_block.inner(vertical_split_area[1]));

            if !internal_split_area.is_empty() {
                self.search_input.draw_text_input(
                    painter,
                    f,
                    internal_split_area[0],
                    is_search_selected,
                );
            }

            if internal_split_area.len() == 2 {
                // TODO: Draw buttons
            }

            f.render_widget(search_block, vertical_split_area[1]);

            vertical_split_area[0]
        } else {
            area
        };

        let area = if self.show_sort {
            const SORT_CONSTRAINTS: [Constraint; 2] = [Constraint::Length(10), Constraint::Min(0)];

            let horizontal_split_area = Layout::default()
                .margin(0)
                .direction(Direction::Horizontal)
                .constraints(SORT_CONSTRAINTS)
                .split(area);

            let sort_block = self
                .block()
                .selected(selected && matches!(self.selected, ProcessManagerSelection::Sort))
                .hide_title(true);
            self.sort_menu.draw_sort_menu(
                painter,
                f,
                self.process_table.columns(),
                sort_block,
                horizontal_split_area[0],
            );

            horizontal_split_area[1]
        } else {
            area
        };

        let process_selected =
            selected && matches!(self.selected, ProcessManagerSelection::Processes);
        let process_block = self
            .block()
            .selected(process_selected)
            .borders(self.block_border)
            .show_esc(expanded && !self.show_sort && !self.show_search);

        self.process_table.draw_tui_table(
            painter,
            f,
            &self.display_data,
            process_block,
            area,
            process_selected,
            self.show_scroll_index,
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        let mut id_pid_map: HashMap<String, ProcessHarvest>;

        let filtered_iter = data_collection.process_harvest.iter().filter(|process| {
            if let Some(Ok(query)) = &self.process_filter {
                query.check(process, self.is_using_command())
            } else {
                true
            }
        });

        let filtered_grouped_iter = if self.is_grouped() {
            id_pid_map = HashMap::new();
            filtered_iter.for_each(|process_harvest| {
                let id = if self.is_using_command() {
                    &process_harvest.command
                } else {
                    &process_harvest.name
                };

                if let Some(grouped_process_harvest) = id_pid_map.get_mut(id) {
                    grouped_process_harvest.cpu_usage_percent += process_harvest.cpu_usage_percent;
                    grouped_process_harvest.mem_usage_bytes += process_harvest.mem_usage_bytes;
                    grouped_process_harvest.mem_usage_percent += process_harvest.mem_usage_percent;
                    grouped_process_harvest.read_bytes_per_sec +=
                        process_harvest.read_bytes_per_sec;
                    grouped_process_harvest.write_bytes_per_sec +=
                        process_harvest.write_bytes_per_sec;
                    grouped_process_harvest.total_read_bytes += process_harvest.total_read_bytes;
                    grouped_process_harvest.total_write_bytes += process_harvest.total_write_bytes;
                } else {
                    id_pid_map.insert(id.clone(), process_harvest.clone());
                }
            });

            Either::Left(id_pid_map.values())
        } else {
            Either::Right(filtered_iter)
        };

        let filtered_sorted_iter = if let ProcessSortType::Count =
            self.process_table.current_sorting_column().sort_type
        {
            let mut v = filtered_grouped_iter.collect::<Vec<_>>();
            v.sort_by_cached_key(|k| {
                if self.is_using_command() {
                    data_collection
                        .process_cmd_pid_map
                        .get(&k.command)
                        .map(|v| v.len())
                        .unwrap_or(0)
                } else {
                    data_collection
                        .process_name_pid_map
                        .get(&k.name)
                        .map(|v| v.len())
                        .unwrap_or(0)
                }
            });
            Either::Left(v.into_iter())
        } else {
            Either::Right(filtered_grouped_iter.sorted_by(
                match self.process_table.current_sorting_column().sort_type {
                    ProcessSortType::Pid => {
                        |a: &&ProcessHarvest, b: &&ProcessHarvest| a.pid.cmp(&b.pid)
                    }
                    ProcessSortType::Count => {
                        // This case should be impossible by the above check.
                        unreachable!()
                    }
                    ProcessSortType::Name => {
                        |a: &&ProcessHarvest, b: &&ProcessHarvest| a.name.cmp(&b.name)
                    }
                    ProcessSortType::Command => {
                        |a: &&ProcessHarvest, b: &&ProcessHarvest| a.command.cmp(&b.command)
                    }
                    ProcessSortType::Cpu => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        FloatOrd(a.cpu_usage_percent).cmp(&FloatOrd(b.cpu_usage_percent))
                    },
                    ProcessSortType::Mem => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.mem_usage_bytes.cmp(&b.mem_usage_bytes)
                    },
                    ProcessSortType::MemPercent => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        FloatOrd(a.mem_usage_percent).cmp(&FloatOrd(b.mem_usage_percent))
                    },
                    ProcessSortType::Rps => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.read_bytes_per_sec.cmp(&b.read_bytes_per_sec)
                    },
                    ProcessSortType::Wps => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.write_bytes_per_sec.cmp(&b.write_bytes_per_sec)
                    },
                    ProcessSortType::TotalRead => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.total_read_bytes.cmp(&b.total_read_bytes)
                    },
                    ProcessSortType::TotalWrite => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.total_write_bytes.cmp(&b.total_write_bytes)
                    },
                    ProcessSortType::User => {
                        #[cfg(target_family = "unix")]
                        {
                            |a: &&ProcessHarvest, b: &&ProcessHarvest| a.user.cmp(&b.user)
                        }
                        #[cfg(not(target_family = "unix"))]
                        {
                            |_a: &&ProcessHarvest, _b: &&ProcessHarvest| std::cmp::Ordering::Equal
                        }
                    }
                    ProcessSortType::State => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.process_state.cmp(&b.process_state)
                    },
                },
            ))
        };

        self.display_data = if let SortStatus::SortDescending = self
            .process_table
            .current_sorting_column()
            .sortable_column
            .sorting_status()
        {
            Either::Left(filtered_sorted_iter.rev())
        } else {
            Either::Right(filtered_sorted_iter)
        }
        .map(|process| {
            self.process_table
                .columns()
                .iter()
                .map(|column| match &column.sort_type {
                    ProcessSortType::Pid => (process.pid.to_string().into(), None, None),
                    ProcessSortType::Count => (
                        if self.is_using_command() {
                            data_collection
                                .process_cmd_pid_map
                                .get(&process.command)
                                .map(|v| v.len())
                                .unwrap_or(0)
                                .to_string()
                                .into()
                        } else {
                            data_collection
                                .process_name_pid_map
                                .get(&process.name)
                                .map(|v| v.len())
                                .unwrap_or(0)
                                .to_string()
                                .into()
                        },
                        None,
                        None,
                    ),
                    ProcessSortType::Name => (process.name.clone().into(), None, None),
                    ProcessSortType::Command => (process.command.clone().into(), None, None),
                    ProcessSortType::Cpu => (
                        format!("{:.1}%", process.cpu_usage_percent).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::Mem => (
                        get_string_with_bytes(process.mem_usage_bytes).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::MemPercent => (
                        format!("{:.1}%", process.mem_usage_percent).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::Rps => (
                        get_string_with_bytes(process.read_bytes_per_sec).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::Wps => (
                        get_string_with_bytes(process.write_bytes_per_sec).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::TotalRead => (
                        get_string_with_bytes(process.total_read_bytes).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::TotalWrite => (
                        get_string_with_bytes(process.total_write_bytes).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::User => (process.user.clone(), None, None),
                    ProcessSortType::State => (
                        process.process_state.clone().into(),
                        None, // Currently disabled; what happens if you try to sort in the shortened form?
                        None,
                    ),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }

    fn handle_widget_selection_left(&mut self) -> SelectionAction {
        if self.show_sort {
            if let ProcessManagerSelection::Processes = self.selected {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Sort;
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }

    fn handle_widget_selection_right(&mut self) -> SelectionAction {
        if self.show_sort {
            if let ProcessManagerSelection::Sort = self.selected {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Processes;
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }

    fn handle_widget_selection_up(&mut self) -> SelectionAction {
        if self.show_search {
            if let ProcessManagerSelection::Search = self.selected {
                let prev = self.prev_selected;
                self.prev_selected = self.selected;
                if self.show_sort && prev == ProcessManagerSelection::Sort {
                    self.selected = ProcessManagerSelection::Sort;
                } else {
                    self.selected = ProcessManagerSelection::Processes;
                }
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }

    fn handle_widget_selection_down(&mut self) -> SelectionAction {
        if self.show_search {
            if let ProcessManagerSelection::Processes = self.selected {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Search;
                SelectionAction::Handled
            } else if self.show_sort && self.selected == ProcessManagerSelection::Sort {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Search;
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }
}
