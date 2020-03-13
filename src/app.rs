use std::cmp::max;
use std::time::Instant;

use unicode_segmentation::GraphemeCursor;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use typed_builder::*;

use data_farmer::*;
use data_harvester::{processes, temperature};

use crate::{canvas, constants, utils::error::Result};

pub mod data_farmer;
pub mod data_harvester;
mod process_killer;

const MAX_SEARCH_LENGTH: usize = 200;

#[derive(Debug, Clone, Copy)]
pub enum WidgetPosition {
    Cpu,
    CpuLegend,
    Mem,
    Disk,
    Temp,
    Network,
    NetworkLegend,
    Process,
    ProcessSearch,
    BasicCpu,
    BasicMem,
    BasicNet,
}

impl WidgetPosition {
    pub fn is_widget_table(self) -> bool {
        match self {
            WidgetPosition::Disk
            | WidgetPosition::Process
            | WidgetPosition::ProcessSearch
            | WidgetPosition::Temp
            | WidgetPosition::CpuLegend => true,
            _ => false,
        }
    }

    pub fn is_widget_graph(self) -> bool {
        match self {
            WidgetPosition::Cpu | WidgetPosition::Network | WidgetPosition::Mem => true,
            _ => false,
        }
    }

    pub fn get_pretty_name(self) -> String {
        use WidgetPosition::*;
        match self {
            Cpu | BasicCpu | CpuLegend => "CPU",
            Mem | BasicMem => "Memory",
            Disk => "Disks",
            Temp => "Temperature",
            Network | BasicNet | NetworkLegend => "Network",
            Process | ProcessSearch => "Processes",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    UP,
    // DOWN means scrolling down --- this usually INCREMENTS
    DOWN,
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
}

pub struct AppScrollState {
    pub scroll_direction: ScrollDirection,
    pub process_scroll_state: AppScrollWidgetState,
    pub disk_scroll_state: AppScrollWidgetState,
    pub temp_scroll_state: AppScrollWidgetState,
    pub cpu_scroll_state: AppScrollWidgetState,
}

impl Default for AppScrollState {
    fn default() -> Self {
        AppScrollState {
            scroll_direction: ScrollDirection::DOWN,
            process_scroll_state: AppScrollWidgetState::default(),
            disk_scroll_state: AppScrollWidgetState::default(),
            temp_scroll_state: AppScrollWidgetState::default(),
            cpu_scroll_state: AppScrollWidgetState::default(),
        }
    }
}

/// AppSearchState deals with generic searching (I might do this in the future).
pub struct AppSearchState {
    pub is_enabled: bool,
    pub current_search_query: String,
    pub current_regex: Option<std::result::Result<regex::Regex, regex::Error>>,
    pub is_blank_search: bool,
    pub is_invalid_search: bool,
    pub grapheme_cursor: GraphemeCursor,
    pub cursor_direction: CursorDirection,
    pub cursor_bar: usize,
    /// This represents the position in terms of CHARACTERS, not graphemes
    pub char_cursor_position: usize,
}

impl Default for AppSearchState {
    fn default() -> Self {
        AppSearchState {
            is_enabled: false,
            current_search_query: String::default(),
            current_regex: None,
            is_invalid_search: false,
            is_blank_search: true,
            grapheme_cursor: GraphemeCursor::new(0, 0, true),
            cursor_direction: CursorDirection::RIGHT,
            cursor_bar: 0,
            char_cursor_position: 0,
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
    pub is_searching_with_pid: bool,
    pub is_ignoring_case: bool,
    pub is_searching_whole_word: bool,
    pub is_searching_with_regex: bool,
}

impl Default for ProcessSearchState {
    fn default() -> Self {
        ProcessSearchState {
            search_state: AppSearchState::default(),
            is_searching_with_pid: false,
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

#[derive(Default)]
pub struct AppDeleteDialogState {
    pub is_showing_dd: bool,
    pub is_on_yes: bool, // Defaults to "No"
}

pub enum AppHelpCategory {
    General,
    Process,
    Search,
}

pub struct AppHelpDialogState {
    pub is_showing_help: bool,
    pub current_category: AppHelpCategory,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
            current_category: AppHelpCategory::General,
        }
    }
}

/// AppConfigFields is meant to cover basic fields that would normally be set
/// by config files or launch options.
pub struct AppConfigFields {
    pub update_rate_in_milliseconds: u64,
    pub temperature_type: temperature::TemperatureType,
    pub use_dot: bool,
    pub left_legend: bool,
    pub show_average_cpu: bool,
    pub use_current_cpu_total: bool,
    pub show_disabled_data: bool,
    pub use_basic_mode: bool,
    pub default_time_value: u64,
    pub time_interval: u64,
    pub hide_time: bool,
    pub autohide_time: bool,
}

/// Network specific
pub struct NetState {
    pub is_showing_tray: bool,
    pub is_showing_rx: bool,
    pub is_showing_tx: bool,
    pub zoom_level: f64,
    pub current_display_time: u64,
    pub force_update: bool,
    pub autohide_timer: Option<Instant>,
}

impl NetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetState {
            is_showing_tray: false,
            is_showing_rx: true,
            is_showing_tx: true,
            zoom_level: 100.0,
            current_display_time,
            force_update: false,
            autohide_timer,
        }
    }
}

/// CPU specific
pub struct CpuState {
    pub is_showing_tray: bool,
    pub zoom_level: f64,
    pub core_show_vec: Vec<bool>,
    pub num_cpus_shown: u64,
    pub current_display_time: u64,
    pub force_update: bool,
    pub autohide_timer: Option<Instant>,
}

impl CpuState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        CpuState {
            is_showing_tray: false,
            zoom_level: 100.0,
            core_show_vec: Vec::new(),
            num_cpus_shown: 0,
            current_display_time,
            force_update: false,
            autohide_timer,
        }
    }
}

/// Memory specific
pub struct MemState {
    pub is_showing_tray: bool,
    pub is_showing_ram: bool,
    pub is_showing_swap: bool,
    pub zoom_level: f64,
    pub current_display_time: u64,
    pub force_update: bool,
    pub autohide_timer: Option<Instant>,
}

impl MemState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemState {
            is_showing_tray: false,
            is_showing_ram: true,
            is_showing_swap: true,
            zoom_level: 100.0,
            current_display_time,
            force_update: false,
            autohide_timer,
        }
    }
}

#[derive(TypedBuilder)]
pub struct App {
    #[builder(default=processes::ProcessSorting::CPU, setter(skip))]
    pub process_sorting_type: processes::ProcessSorting,

    #[builder(default = true, setter(skip))]
    pub process_sorting_reverse: bool,

    #[builder(default = false, setter(skip))]
    pub force_update_processes: bool,

    #[builder(default, setter(skip))]
    pub app_scroll_positions: AppScrollState,

    #[builder(default = false, setter(skip))]
    awaiting_second_char: bool,

    #[builder(default, setter(skip))]
    second_char: Option<char>,

    #[builder(default, setter(skip))]
    pub dd_err: Option<String>,

    #[builder(default, setter(skip))]
    to_delete_process_list: Option<(String, Vec<u32>)>,

    #[builder(default = false, setter(skip))]
    pub is_frozen: bool,

    #[builder(default = Instant::now(), setter(skip))]
    last_key_press: Instant,

    #[builder(default, setter(skip))]
    pub canvas_data: canvas::DisplayableData,

    #[builder(default = false)]
    enable_grouping: bool,

    #[builder(default, setter(skip))]
    pub data_collection: DataCollection,

    #[builder(default, setter(skip))]
    pub process_search_state: ProcessSearchState,

    #[builder(default, setter(skip))]
    pub delete_dialog_state: AppDeleteDialogState,

    #[builder(default, setter(skip))]
    pub help_dialog_state: AppHelpDialogState,

    #[builder(default = false, setter(skip))]
    pub is_expanded: bool,

    #[builder(default = false, setter(skip))]
    pub is_resized: bool,

    pub cpu_state: CpuState,
    pub mem_state: MemState,
    pub net_state: NetState,

    pub app_config_fields: AppConfigFields,
    pub current_widget_selected: WidgetPosition,
    pub previous_basic_table_selected: WidgetPosition,
}

impl App {
    pub fn reset(&mut self) {
        // Reset multi
        self.reset_multi_tap_keys();

        // Reset dialog state
        self.help_dialog_state.is_showing_help = false;
        self.delete_dialog_state.is_showing_dd = false;

        // Close search and reset it
        self.process_search_state.search_state.reset();
        self.force_update_processes = true;

        // Clear current delete list
        self.to_delete_process_list = None;
        self.dd_err = None;

        // Unfreeze.
        self.is_frozen = false;

        // Reset zoom
        self.reset_cpu_zoom();
        self.reset_mem_zoom();
        self.reset_net_zoom();

        // Reset data
        self.data_collection.reset();
    }

    pub fn on_esc(&mut self) {
        self.reset_multi_tap_keys();
        if self.is_in_dialog() {
            self.help_dialog_state.is_showing_help = false;
            self.help_dialog_state.current_category = AppHelpCategory::General;
            self.delete_dialog_state.is_showing_dd = false;
            self.delete_dialog_state.is_on_yes = false;
            self.to_delete_process_list = None;
            self.dd_err = None;
        } else if self.is_filtering_or_searching() {
            match self.current_widget_selected {
                WidgetPosition::Cpu | WidgetPosition::CpuLegend => {
                    self.cpu_state.is_showing_tray = false;
                    if self
                        .app_scroll_positions
                        .cpu_scroll_state
                        .current_scroll_position
                        >= self.cpu_state.num_cpus_shown
                    {
                        let new_position = max(0, self.cpu_state.num_cpus_shown as i64 - 1) as u64;
                        self.app_scroll_positions
                            .cpu_scroll_state
                            .current_scroll_position = new_position;
                        self.app_scroll_positions
                            .cpu_scroll_state
                            .previous_scroll_position = 0;
                    }
                }
                WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                    if self.process_search_state.search_state.is_enabled {
                        self.current_widget_selected = WidgetPosition::Process;
                        self.process_search_state.search_state.is_enabled = false;
                    }
                }
                WidgetPosition::Mem => {
                    self.mem_state.is_showing_tray = false;
                }
                WidgetPosition::Network => {
                    self.net_state.is_showing_tray = false;
                }
                _ => {}
            }
        } else if self.is_expanded {
            self.is_expanded = false;
            self.is_resized = true;
            if self.app_config_fields.use_basic_mode {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::Cpu | WidgetPosition::CpuLegend => WidgetPosition::BasicCpu,
                    WidgetPosition::Mem => WidgetPosition::BasicMem,
                    WidgetPosition::Network => WidgetPosition::BasicNet,
                    _ => self.current_widget_selected,
                }
            }
        }
    }

    fn is_filtering_or_searching(&self) -> bool {
        match self.current_widget_selected {
            WidgetPosition::Cpu | WidgetPosition::CpuLegend => self.cpu_state.is_showing_tray,
            // WidgetPosition::Mem => self.mem_state.is_showing_tray,
            // WidgetPosition::Network => self.net_state.is_showing_tray,
            WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                self.process_search_state.search_state.is_enabled
            }
            _ => false,
        }
    }

    fn reset_multi_tap_keys(&mut self) {
        self.awaiting_second_char = false;
        self.second_char = None;
    }

    fn is_in_dialog(&self) -> bool {
        self.help_dialog_state.is_showing_help || self.delete_dialog_state.is_showing_dd
    }

    pub fn toggle_grouping(&mut self) {
        // Disallow usage whilst in a dialog and only in processes
        if !self.is_in_dialog() {
            if let WidgetPosition::Process = self.current_widget_selected {
                self.enable_grouping = !(self.enable_grouping);
                self.force_update_processes = true;
            }
        }
    }

    pub fn on_tab(&mut self) {
        match self.current_widget_selected {
            WidgetPosition::Process => {
                self.toggle_grouping();
                if self.is_grouped() {
                    self.search_with_name();
                } else {
                    self.force_update_processes = true;
                }
            }
            WidgetPosition::ProcessSearch => {
                if !self.is_grouped() {
                    if self.process_search_state.is_searching_with_pid {
                        self.search_with_name();
                    } else {
                        self.search_with_pid();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn is_grouped(&self) -> bool {
        self.enable_grouping
    }

    pub fn on_space(&mut self) {
        match self.current_widget_selected {
            WidgetPosition::CpuLegend => {
                let curr_posn = self
                    .app_scroll_positions
                    .cpu_scroll_state
                    .current_scroll_position;
                if self.cpu_state.is_showing_tray
                    && curr_posn < self.data_collection.cpu_harvest.len() as u64
                {
                    self.cpu_state.core_show_vec[curr_posn as usize] =
                        !self.cpu_state.core_show_vec[curr_posn as usize];

                    if !self.app_config_fields.show_disabled_data {
                        if !self.cpu_state.core_show_vec[curr_posn as usize] {
                            self.cpu_state.num_cpus_shown -= 1;
                        } else {
                            self.cpu_state.num_cpus_shown += 1;
                        }
                    }
                }
            }
            WidgetPosition::Network => {}
            _ => {}
        }
    }

    pub fn on_slash(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget_selected {
                WidgetPosition::BasicCpu if self.is_expanded => {
                    self.current_widget_selected = WidgetPosition::Cpu;
                    self.cpu_state.is_showing_tray = true;
                }
                WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                    // Toggle on
                    self.process_search_state.search_state.is_enabled = true;
                    self.current_widget_selected = WidgetPosition::ProcessSearch;
                    if self.is_grouped() {
                        self.search_with_name();
                    }
                }
                WidgetPosition::Cpu | WidgetPosition::CpuLegend => {
                    self.cpu_state.is_showing_tray = true;
                    self.current_widget_selected = WidgetPosition::CpuLegend
                }
                // WidgetPosition::Mem => {
                // 	self.mem_state.is_showing_tray = true;
                // }
                // WidgetPosition::Network => {
                // 	self.net_state.is_showing_tray = true;
                // }
                _ => {}
            }
        }
    }

    pub fn is_searching(&self) -> bool {
        self.process_search_state.search_state.is_enabled
    }

    pub fn is_in_search_widget(&self) -> bool {
        if let WidgetPosition::ProcessSearch = self.current_widget_selected {
            true
        } else {
            false
        }
    }

    pub fn search_with_pid(&mut self) {
        if !self.is_in_dialog() && self.is_searching() {
            self.process_search_state.is_searching_with_pid = true;
            self.force_update_processes = true;
        }
    }

    pub fn search_with_name(&mut self) {
        if !self.is_in_dialog() && self.is_searching() {
            self.process_search_state.is_searching_with_pid = false;
            self.force_update_processes = true;
        }
    }

    pub fn get_current_search_query(&self) -> &String {
        &self.process_search_state.search_state.current_search_query
    }

    pub fn toggle_ignore_case(&mut self) {
        self.process_search_state.search_toggle_ignore_case();
        self.update_regex();
        self.force_update_processes = true;
    }

    pub fn toggle_search_whole_word(&mut self) {
        self.process_search_state.search_toggle_whole_word();
        self.update_regex();
        self.force_update_processes = true;
    }

    pub fn toggle_search_regex(&mut self) {
        self.process_search_state.search_toggle_regex();
        self.update_regex();
        self.force_update_processes = true;
    }

    pub fn update_regex(&mut self) {
        if self
            .process_search_state
            .search_state
            .current_search_query
            .is_empty()
        {
            self.process_search_state.search_state.is_invalid_search = false;
            self.process_search_state.search_state.is_blank_search = true;
        } else {
            let regex_string = &self.process_search_state.search_state.current_search_query;
            let escaped_regex: String;
            let final_regex_string = &format!(
                "{}{}{}{}",
                if self.process_search_state.is_searching_whole_word {
                    "^"
                } else {
                    ""
                },
                if self.process_search_state.is_ignoring_case {
                    "(?i)"
                } else {
                    ""
                },
                if !self.process_search_state.is_searching_with_regex {
                    escaped_regex = regex::escape(regex_string);
                    &escaped_regex
                } else {
                    regex_string
                },
                if self.process_search_state.is_searching_whole_word {
                    "$"
                } else {
                    ""
                },
            );

            let new_regex = regex::Regex::new(final_regex_string);
            self.process_search_state.search_state.is_blank_search = false;
            self.process_search_state.search_state.is_invalid_search = new_regex.is_err();

            self.process_search_state.search_state.current_regex = Some(new_regex);
        }
        self.app_scroll_positions
            .process_scroll_state
            .previous_scroll_position = 0;
        self.app_scroll_positions
            .process_scroll_state
            .current_scroll_position = 0;
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

    /// One of two functions allowed to run while in a dialog...
    pub fn on_enter(&mut self) {
        if self.delete_dialog_state.is_showing_dd {
            if self.delete_dialog_state.is_on_yes {
                // If within dd...
                if self.dd_err.is_none() {
                    // Also ensure that we didn't just fail a dd...
                    let dd_result = self.kill_highlighted_process();
                    self.delete_dialog_state.is_on_yes = false;

                    // Check if there was an issue... if so, inform the user.
                    if let Err(dd_err) = dd_result {
                        self.dd_err = Some(dd_err.to_string());
                    } else {
                        self.delete_dialog_state.is_showing_dd = false;
                    }
                }
            } else {
                self.delete_dialog_state.is_showing_dd = false;
            }
        } else if !self.is_in_dialog() {
            // Pop-out mode.  We ignore if in process search.

            match self.current_widget_selected {
                WidgetPosition::ProcessSearch => {}
                _ => {
                    self.is_expanded = true;
                    self.is_resized = true;
                }
            }

            if self.app_config_fields.use_basic_mode {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::BasicCpu => WidgetPosition::Cpu,
                    WidgetPosition::BasicMem => WidgetPosition::Mem,
                    WidgetPosition::BasicNet => WidgetPosition::Network,
                    _ => self.current_widget_selected,
                }
            }
        }
    }

    pub fn on_delete(&mut self) {
        match self.current_widget_selected {
            WidgetPosition::Process => self.start_dd(),
            WidgetPosition::ProcessSearch => {
                if self.process_search_state.search_state.is_enabled
                    && self.get_cursor_position()
                        < self
                            .process_search_state
                            .search_state
                            .current_search_query
                            .len()
                {
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .remove(self.get_cursor_position());

                    self.process_search_state.search_state.grapheme_cursor = GraphemeCursor::new(
                        self.get_cursor_position(),
                        self.process_search_state
                            .search_state
                            .current_search_query
                            .len(),
                        true,
                    );

                    self.update_regex();
                    self.force_update_processes = true;
                }
            }
            _ => {}
        }
    }

    /// Deletes an entire word till the next space or end
    #[allow(unused_variables)]
    pub fn skip_word_backspace(&mut self) {
        if let WidgetPosition::ProcessSearch = self.current_widget_selected {
            if self.process_search_state.search_state.is_enabled {}
        }
    }

    pub fn clear_search(&mut self) {
        if let WidgetPosition::ProcessSearch = self.current_widget_selected {
            self.force_update_processes = true;
            self.process_search_state.search_state.reset();
        }
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

    pub fn on_backspace(&mut self) {
        if let WidgetPosition::ProcessSearch = self.current_widget_selected {
            if self.process_search_state.search_state.is_enabled && self.get_cursor_position() > 0 {
                self.search_walk_back(self.get_cursor_position());

                let removed_char = self
                    .process_search_state
                    .search_state
                    .current_search_query
                    .remove(self.get_cursor_position());

                self.process_search_state.search_state.grapheme_cursor = GraphemeCursor::new(
                    self.get_cursor_position(),
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .len(),
                    true,
                );

                self.process_search_state.search_state.char_cursor_position -=
                    UnicodeWidthChar::width(removed_char).unwrap_or(0);
                self.process_search_state.search_state.cursor_direction = CursorDirection::LEFT;

                self.update_regex();
                self.force_update_processes = true;
            }
        }
    }

    pub fn get_current_regex_matcher(
        &self,
    ) -> &Option<std::result::Result<regex::Regex, regex::Error>> {
        &self.process_search_state.search_state.current_regex
    }

    pub fn on_up_key(&mut self) {
        if !self.is_in_dialog() {
            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
            } else {
                self.decrement_position_count();
            }
        }
    }

    pub fn on_down_key(&mut self) {
        if !self.is_in_dialog() {
            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
            } else {
                self.increment_position_count();
            }
        }
    }

    pub fn on_left_key(&mut self) {
        if !self.is_in_dialog() {
            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
                let prev_cursor = self.get_cursor_position();
                self.search_walk_back(self.get_cursor_position());
                if self.get_cursor_position() < prev_cursor {
                    let str_slice = &self.process_search_state.search_state.current_search_query
                        [self.get_cursor_position()..prev_cursor];
                    self.process_search_state.search_state.char_cursor_position -=
                        UnicodeWidthStr::width(str_slice);
                    self.process_search_state.search_state.cursor_direction = CursorDirection::LEFT;
                }
            }
        } else if self.delete_dialog_state.is_showing_dd && !self.delete_dialog_state.is_on_yes {
            self.delete_dialog_state.is_on_yes = true;
        }
    }

    pub fn on_right_key(&mut self) {
        if !self.is_in_dialog() {
            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
                let prev_cursor = self.get_cursor_position();
                self.search_walk_forward(self.get_cursor_position());
                if self.get_cursor_position() > prev_cursor {
                    let str_slice = &self.process_search_state.search_state.current_search_query
                        [prev_cursor..self.get_cursor_position()];
                    self.process_search_state.search_state.char_cursor_position +=
                        UnicodeWidthStr::width(str_slice);
                    self.process_search_state.search_state.cursor_direction =
                        CursorDirection::RIGHT;
                }
            }
        } else if self.delete_dialog_state.is_showing_dd && self.delete_dialog_state.is_on_yes {
            self.delete_dialog_state.is_on_yes = false;
        }
    }

    pub fn skip_cursor_beginning(&mut self) {
        if !self.is_in_dialog() {
            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
                self.process_search_state.search_state.grapheme_cursor = GraphemeCursor::new(
                    0,
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .len(),
                    true,
                );
                self.process_search_state.search_state.char_cursor_position = 0;
                self.process_search_state.search_state.cursor_direction = CursorDirection::LEFT;
            }
        }
    }

    pub fn skip_cursor_end(&mut self) {
        if !self.is_in_dialog() {
            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
                self.process_search_state.search_state.grapheme_cursor = GraphemeCursor::new(
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .len(),
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .len(),
                    true,
                );
                self.process_search_state.search_state.char_cursor_position =
                    UnicodeWidthStr::width(
                        self.process_search_state
                            .search_state
                            .current_search_query
                            .as_str(),
                    );
                self.process_search_state.search_state.cursor_direction = CursorDirection::RIGHT;
            }
        }
    }

    pub fn start_dd(&mut self) {
        if self
            .app_scroll_positions
            .process_scroll_state
            .current_scroll_position
            < self.canvas_data.finalized_process_data.len() as u64
        {
            let current_process = if self.is_grouped() {
                let group_pids = &self.canvas_data.finalized_process_data[self
                    .app_scroll_positions
                    .process_scroll_state
                    .current_scroll_position
                    as usize]
                    .group_pids;

                let mut ret = ("".to_string(), group_pids.clone());

                for pid in group_pids {
                    if let Some(process) = self.canvas_data.process_data.get(&pid) {
                        ret.0 = process.name.clone();
                        break;
                    }
                }
                ret
            } else {
                let process = self.canvas_data.finalized_process_data[self
                    .app_scroll_positions
                    .process_scroll_state
                    .current_scroll_position
                    as usize]
                    .clone();
                (process.name.clone(), vec![process.pid])
            };

            self.to_delete_process_list = Some(current_process);
            self.delete_dialog_state.is_showing_dd = true;
        }

        self.reset_multi_tap_keys();
    }

    pub fn on_char_key(&mut self, caught_char: char) {
        // Skip control code chars
        if caught_char.is_control() {
            return;
        }

        // Forbid any char key presses when showing a dialog box...
        if !self.is_in_dialog() {
            let current_key_press_inst = Instant::now();
            if current_key_press_inst
                .duration_since(self.last_key_press)
                .as_millis()
                > constants::MAX_KEY_TIMEOUT_IN_MILLISECONDS as u128
            {
                self.reset_multi_tap_keys();
            }
            self.last_key_press = current_key_press_inst;

            if let WidgetPosition::ProcessSearch = self.current_widget_selected {
                if UnicodeWidthStr::width(
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .as_str(),
                ) <= MAX_SEARCH_LENGTH
                {
                    self.process_search_state
                        .search_state
                        .current_search_query
                        .insert(self.get_cursor_position(), caught_char);

                    self.process_search_state.search_state.grapheme_cursor = GraphemeCursor::new(
                        self.get_cursor_position(),
                        self.process_search_state
                            .search_state
                            .current_search_query
                            .len(),
                        true,
                    );
                    self.search_walk_forward(self.get_cursor_position());

                    self.process_search_state.search_state.char_cursor_position +=
                        UnicodeWidthChar::width(caught_char).unwrap_or(0);

                    self.update_regex();
                    self.force_update_processes = true;
                    self.process_search_state.search_state.cursor_direction =
                        CursorDirection::RIGHT;
                }
            } else {
                match caught_char {
                    '/' => {
                        self.on_slash();
                    }
                    'd' => {
                        if let WidgetPosition::Process = self.current_widget_selected {
                            let mut is_first_d = true;
                            if let Some(second_char) = self.second_char {
                                if self.awaiting_second_char && second_char == 'd' {
                                    is_first_d = false;
                                    self.awaiting_second_char = false;
                                    self.second_char = None;

                                    self.start_dd();
                                }
                            }

                            if is_first_d {
                                self.awaiting_second_char = true;
                                self.second_char = Some('d');
                            }
                        }
                    }
                    'g' => {
                        let mut is_first_g = true;
                        if let Some(second_char) = self.second_char {
                            if self.awaiting_second_char && second_char == 'g' {
                                is_first_g = false;
                                self.awaiting_second_char = false;
                                self.second_char = None;
                                self.skip_to_first();
                            }
                        }

                        if is_first_g {
                            self.awaiting_second_char = true;
                            self.second_char = Some('g');
                        }
                    }
                    'G' => self.skip_to_last(),
                    'k' => self.decrement_position_count(),
                    'j' => self.increment_position_count(),
                    'f' => {
                        self.is_frozen = !self.is_frozen;
                        if self.is_frozen {
                            self.data_collection.set_frozen_time();
                        }
                    }
                    'c' => {
                        match self.process_sorting_type {
                            processes::ProcessSorting::CPU => {
                                self.process_sorting_reverse = !self.process_sorting_reverse
                            }
                            _ => {
                                self.process_sorting_type = processes::ProcessSorting::CPU;
                                self.process_sorting_reverse = true;
                            }
                        }
                        self.force_update_processes = true;
                        self.app_scroll_positions
                            .process_scroll_state
                            .current_scroll_position = 0;
                    }
                    'm' => {
                        match self.process_sorting_type {
                            processes::ProcessSorting::MEM => {
                                self.process_sorting_reverse = !self.process_sorting_reverse
                            }
                            _ => {
                                self.process_sorting_type = processes::ProcessSorting::MEM;
                                self.process_sorting_reverse = true;
                            }
                        }
                        self.force_update_processes = true;
                        self.app_scroll_positions
                            .process_scroll_state
                            .current_scroll_position = 0;
                    }
                    'p' => {
                        // Disable if grouping
                        if !self.enable_grouping {
                            match self.process_sorting_type {
                                processes::ProcessSorting::PID => {
                                    self.process_sorting_reverse = !self.process_sorting_reverse
                                }
                                _ => {
                                    self.process_sorting_type = processes::ProcessSorting::PID;
                                    self.process_sorting_reverse = false;
                                }
                            }
                            self.force_update_processes = true;
                            self.app_scroll_positions
                                .process_scroll_state
                                .current_scroll_position = 0;
                        }
                    }
                    'n' => {
                        match self.process_sorting_type {
                            processes::ProcessSorting::NAME => {
                                self.process_sorting_reverse = !self.process_sorting_reverse
                            }
                            _ => {
                                self.process_sorting_type = processes::ProcessSorting::NAME;
                                self.process_sorting_reverse = false;
                            }
                        }
                        self.force_update_processes = true;
                        self.app_scroll_positions
                            .process_scroll_state
                            .current_scroll_position = 0;
                    }
                    '?' => {
                        self.help_dialog_state.is_showing_help = true;
                    }
                    'H' => self.move_widget_selection_left(),
                    'L' => self.move_widget_selection_right(),
                    'K' => self.move_widget_selection_up(),
                    'J' => self.move_widget_selection_down(),
                    ' ' => self.on_space(),
                    '+' => self.zoom_in(),
                    '-' => self.zoom_out(),
                    '=' => self.reset_zoom(),
                    _ => {}
                }

                if let Some(second_char) = self.second_char {
                    if self.awaiting_second_char && caught_char != second_char {
                        self.awaiting_second_char = false;
                    }
                }
            }
        } else if self.help_dialog_state.is_showing_help {
            match caught_char {
                '1' => self.help_dialog_state.current_category = AppHelpCategory::General,
                '2' => self.help_dialog_state.current_category = AppHelpCategory::Process,
                '3' => self.help_dialog_state.current_category = AppHelpCategory::Search,
                _ => {}
            }
        }
    }

    pub fn kill_highlighted_process(&mut self) -> Result<()> {
        // Technically unnecessary but this is a good check...
        if let WidgetPosition::Process = self.current_widget_selected {
            if let Some(current_selected_processes) = &self.to_delete_process_list {
                for pid in &current_selected_processes.1 {
                    process_killer::kill_process_given_pid(*pid)?;
                }
            }
            self.to_delete_process_list = None;
        }
        Ok(())
    }

    pub fn get_to_delete_processes(&self) -> Option<(String, Vec<u32>)> {
        self.to_delete_process_list.clone()
    }

    // TODO: [MODULARITY] Do NOT hard code this in thu future!
    //
    // General idea for now:
    // CPU -(down)> MEM
    // MEM -(down)> Network, -(right)> TEMP
    // TEMP -(down)> Disk, -(left)> MEM, -(up)> CPU
    // Disk -(down)> Processes, -(left)> MEM, -(up)> TEMP
    // Network -(up)> MEM, -(right)> PROC
    // PROC -(up)> Disk, -(down)> PROC_SEARCH, -(left)> Network
    // PROC_SEARCH -(up)> PROC, -(left)> Network
    pub fn move_widget_selection_left(&mut self) {
        if !self.is_in_dialog() && !self.is_expanded {
            if self.app_config_fields.use_basic_mode {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::BasicNet => WidgetPosition::BasicMem,
                    WidgetPosition::Process => WidgetPosition::Disk,
                    WidgetPosition::ProcessSearch => WidgetPosition::Disk,
                    WidgetPosition::Disk => WidgetPosition::Temp,
                    WidgetPosition::Temp => WidgetPosition::Process,
                    _ => self.current_widget_selected,
                };
            } else {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::Cpu if self.app_config_fields.left_legend => {
                        WidgetPosition::CpuLegend
                    }
                    WidgetPosition::CpuLegend if !self.app_config_fields.left_legend => {
                        WidgetPosition::Cpu
                    }
                    WidgetPosition::Process => WidgetPosition::Network,
                    WidgetPosition::ProcessSearch => WidgetPosition::Network,
                    WidgetPosition::Disk => WidgetPosition::Mem,
                    WidgetPosition::Temp => WidgetPosition::Mem,
                    _ => self.current_widget_selected,
                };
            }
        } else if self.is_expanded {
            self.current_widget_selected = match self.current_widget_selected {
                WidgetPosition::Cpu if self.app_config_fields.left_legend => {
                    WidgetPosition::CpuLegend
                }
                WidgetPosition::CpuLegend if !self.app_config_fields.left_legend => {
                    WidgetPosition::Cpu
                }
                _ => self.current_widget_selected,
            }
        }

        self.reset_multi_tap_keys();
    }

    pub fn move_widget_selection_right(&mut self) {
        if !self.is_in_dialog() && !self.is_expanded {
            if self.app_config_fields.use_basic_mode {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::BasicMem => WidgetPosition::BasicNet,
                    WidgetPosition::Process => WidgetPosition::Temp,
                    WidgetPosition::ProcessSearch => WidgetPosition::Temp,
                    WidgetPosition::Disk => WidgetPosition::Process,
                    WidgetPosition::Temp => WidgetPosition::Disk,
                    _ => self.current_widget_selected,
                };
            } else {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::Cpu if !self.app_config_fields.left_legend => {
                        WidgetPosition::CpuLegend
                    }
                    WidgetPosition::CpuLegend if self.app_config_fields.left_legend => {
                        WidgetPosition::Cpu
                    }
                    WidgetPosition::Mem => WidgetPosition::Temp,
                    WidgetPosition::Network => WidgetPosition::Process,
                    _ => self.current_widget_selected,
                };
            }
        } else if self.is_expanded {
            self.current_widget_selected = match self.current_widget_selected {
                WidgetPosition::Cpu if !self.app_config_fields.left_legend => {
                    WidgetPosition::CpuLegend
                }
                WidgetPosition::CpuLegend if self.app_config_fields.left_legend => {
                    WidgetPosition::Cpu
                }
                _ => self.current_widget_selected,
            }
        }

        self.reset_multi_tap_keys();
    }

    pub fn move_widget_selection_up(&mut self) {
        if !self.is_in_dialog() && !self.is_expanded {
            if self.app_config_fields.use_basic_mode {
                if self.current_widget_selected.is_widget_table() {
                    self.previous_basic_table_selected = self.current_widget_selected;
                }
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::BasicMem => WidgetPosition::BasicCpu,
                    WidgetPosition::BasicNet => WidgetPosition::BasicCpu,
                    WidgetPosition::Process => WidgetPosition::BasicMem,
                    WidgetPosition::ProcessSearch => WidgetPosition::Process,
                    WidgetPosition::Temp => WidgetPosition::BasicMem,
                    WidgetPosition::Disk => WidgetPosition::BasicMem,
                    _ => self.current_widget_selected,
                };
            } else {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::Mem => WidgetPosition::Cpu,
                    WidgetPosition::Network => WidgetPosition::Mem,
                    WidgetPosition::Process => WidgetPosition::Disk,
                    WidgetPosition::ProcessSearch => WidgetPosition::Process,
                    WidgetPosition::Temp => WidgetPosition::Cpu,
                    WidgetPosition::Disk => WidgetPosition::Temp,
                    _ => self.current_widget_selected,
                };
            }
        } else if self.is_expanded {
            self.current_widget_selected = match self.current_widget_selected {
                WidgetPosition::ProcessSearch => WidgetPosition::Process,
                _ => self.current_widget_selected,
            };
        }

        self.reset_multi_tap_keys();
    }

    pub fn move_widget_selection_down(&mut self) {
        if !self.is_in_dialog() && !self.is_expanded {
            if self.app_config_fields.use_basic_mode {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::BasicMem => self.previous_basic_table_selected,
                    WidgetPosition::BasicNet => self.previous_basic_table_selected,
                    WidgetPosition::BasicCpu => WidgetPosition::BasicMem,
                    WidgetPosition::Process => {
                        if self.is_searching() {
                            WidgetPosition::ProcessSearch
                        } else {
                            WidgetPosition::Process
                        }
                    }
                    _ => self.current_widget_selected,
                };
            } else {
                self.current_widget_selected = match self.current_widget_selected {
                    WidgetPosition::Cpu | WidgetPosition::CpuLegend => WidgetPosition::Mem,
                    WidgetPosition::Mem => WidgetPosition::Network,
                    WidgetPosition::Temp => WidgetPosition::Disk,
                    WidgetPosition::Disk => WidgetPosition::Process,
                    WidgetPosition::Process => {
                        if self.is_searching() {
                            WidgetPosition::ProcessSearch
                        } else {
                            WidgetPosition::Process
                        }
                    }
                    _ => self.current_widget_selected,
                };
            }
        } else if self.is_expanded {
            self.current_widget_selected = match self.current_widget_selected {
                WidgetPosition::Process => {
                    if self.is_searching() {
                        WidgetPosition::ProcessSearch
                    } else {
                        WidgetPosition::Process
                    }
                }
                _ => self.current_widget_selected,
            };
        }

        self.reset_multi_tap_keys();
    }

    pub fn skip_to_first(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget_selected {
                WidgetPosition::Process => {
                    self.app_scroll_positions
                        .process_scroll_state
                        .current_scroll_position = 0
                }
                WidgetPosition::Temp => {
                    self.app_scroll_positions
                        .temp_scroll_state
                        .current_scroll_position = 0
                }
                WidgetPosition::Disk => {
                    self.app_scroll_positions
                        .disk_scroll_state
                        .current_scroll_position = 0
                }
                WidgetPosition::CpuLegend => {
                    self.app_scroll_positions
                        .cpu_scroll_state
                        .current_scroll_position = 0
                }

                _ => {}
            }
            self.app_scroll_positions.scroll_direction = ScrollDirection::UP;
            self.reset_multi_tap_keys();
        }
    }

    pub fn skip_to_last(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget_selected {
                WidgetPosition::Process => {
                    self.app_scroll_positions
                        .process_scroll_state
                        .current_scroll_position =
                        self.canvas_data.finalized_process_data.len() as u64 - 1
                }
                WidgetPosition::Temp => {
                    self.app_scroll_positions
                        .temp_scroll_state
                        .current_scroll_position =
                        self.canvas_data.temp_sensor_data.len() as u64 - 1
                }
                WidgetPosition::Disk => {
                    self.app_scroll_positions
                        .disk_scroll_state
                        .current_scroll_position = self.canvas_data.disk_data.len() as u64 - 1
                }
                WidgetPosition::CpuLegend => {
                    self.app_scroll_positions
                        .cpu_scroll_state
                        .current_scroll_position = self.canvas_data.cpu_data.len() as u64 - 1;
                }
                _ => {}
            }
            self.app_scroll_positions.scroll_direction = ScrollDirection::DOWN;
            self.reset_multi_tap_keys();
        }
    }

    pub fn decrement_position_count(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget_selected {
                WidgetPosition::Process => self.change_process_position(-1),
                WidgetPosition::Temp => self.change_temp_position(-1),
                WidgetPosition::Disk => self.change_disk_position(-1),
                WidgetPosition::CpuLegend => self.change_cpu_table_position(-1),
                _ => {}
            }
            self.app_scroll_positions.scroll_direction = ScrollDirection::UP;
            self.reset_multi_tap_keys();
        }
    }

    pub fn increment_position_count(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget_selected {
                WidgetPosition::Process => self.change_process_position(1),
                WidgetPosition::Temp => self.change_temp_position(1),
                WidgetPosition::Disk => self.change_disk_position(1),
                WidgetPosition::CpuLegend => self.change_cpu_table_position(1),
                _ => {}
            }
            self.app_scroll_positions.scroll_direction = ScrollDirection::DOWN;
            self.reset_multi_tap_keys();
        }
    }

    fn change_cpu_table_position(&mut self, num_to_change_by: i64) {
        let current_posn = self
            .app_scroll_positions
            .cpu_scroll_state
            .current_scroll_position;

        let cap = if self.is_filtering_or_searching() {
            self.canvas_data.cpu_data.len() as u64
        } else {
            self.cpu_state.num_cpus_shown
        };

        if current_posn as i64 + num_to_change_by >= 0
            && current_posn as i64 + num_to_change_by < cap as i64
        {
            self.app_scroll_positions
                .cpu_scroll_state
                .current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
        }
    }

    fn change_process_position(&mut self, num_to_change_by: i64) {
        let current_posn = self
            .app_scroll_positions
            .process_scroll_state
            .current_scroll_position;

        if current_posn as i64 + num_to_change_by >= 0
            && current_posn as i64 + num_to_change_by
                < self.canvas_data.finalized_process_data.len() as i64
        {
            self.app_scroll_positions
                .process_scroll_state
                .current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
        }
    }

    fn change_temp_position(&mut self, num_to_change_by: i64) {
        let current_posn = self
            .app_scroll_positions
            .temp_scroll_state
            .current_scroll_position;

        if current_posn as i64 + num_to_change_by >= 0
            && current_posn as i64 + num_to_change_by
                < self.canvas_data.temp_sensor_data.len() as i64
        {
            self.app_scroll_positions
                .temp_scroll_state
                .current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
        }
    }

    fn change_disk_position(&mut self, num_to_change_by: i64) {
        let current_posn = self
            .app_scroll_positions
            .disk_scroll_state
            .current_scroll_position;

        if current_posn as i64 + num_to_change_by >= 0
            && current_posn as i64 + num_to_change_by < self.canvas_data.disk_data.len() as i64
        {
            self.app_scroll_positions
                .disk_scroll_state
                .current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
        }
    }

    pub fn handle_scroll_up(&mut self) {
        if self.current_widget_selected.is_widget_graph() {
            self.zoom_in();
        } else if self.current_widget_selected.is_widget_table() {
            self.decrement_position_count();
        }
    }

    pub fn handle_scroll_down(&mut self) {
        if self.current_widget_selected.is_widget_graph() {
            self.zoom_out();
        } else if self.current_widget_selected.is_widget_table() {
            self.increment_position_count();
        }
    }

    fn zoom_out(&mut self) {
        match self.current_widget_selected {
            WidgetPosition::Cpu => {
                let new_time =
                    self.cpu_state.current_display_time + self.app_config_fields.time_interval;
                if new_time <= constants::STALE_MAX_MILLISECONDS {
                    self.cpu_state.current_display_time = new_time;
                    self.cpu_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.cpu_state.autohide_timer = Some(Instant::now());
                    }
                } else if self.cpu_state.current_display_time != constants::STALE_MAX_MILLISECONDS {
                    self.cpu_state.current_display_time = constants::STALE_MAX_MILLISECONDS;
                    self.cpu_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.cpu_state.autohide_timer = Some(Instant::now());
                    }
                }
            }
            WidgetPosition::Mem => {
                let new_time =
                    self.mem_state.current_display_time + self.app_config_fields.time_interval;
                if new_time <= constants::STALE_MAX_MILLISECONDS {
                    self.mem_state.current_display_time = new_time;
                    self.mem_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.mem_state.autohide_timer = Some(Instant::now());
                    }
                } else if self.mem_state.current_display_time != constants::STALE_MAX_MILLISECONDS {
                    self.mem_state.current_display_time = constants::STALE_MAX_MILLISECONDS;
                    self.mem_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.mem_state.autohide_timer = Some(Instant::now());
                    }
                }
            }
            WidgetPosition::Network => {
                let new_time =
                    self.net_state.current_display_time + self.app_config_fields.time_interval;
                if new_time <= constants::STALE_MAX_MILLISECONDS {
                    self.net_state.current_display_time = new_time;
                    self.net_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.net_state.autohide_timer = Some(Instant::now());
                    }
                } else if self.net_state.current_display_time != constants::STALE_MAX_MILLISECONDS {
                    self.net_state.current_display_time = constants::STALE_MAX_MILLISECONDS;
                    self.net_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.net_state.autohide_timer = Some(Instant::now());
                    }
                }
            }
            _ => {}
        }
    }

    fn zoom_in(&mut self) {
        match self.current_widget_selected {
            WidgetPosition::Cpu => {
                let new_time =
                    self.cpu_state.current_display_time - self.app_config_fields.time_interval;
                if new_time >= constants::STALE_MIN_MILLISECONDS {
                    self.cpu_state.current_display_time = new_time;
                    self.cpu_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.cpu_state.autohide_timer = Some(Instant::now());
                    }
                } else if self.cpu_state.current_display_time != constants::STALE_MIN_MILLISECONDS {
                    self.cpu_state.current_display_time = constants::STALE_MIN_MILLISECONDS;
                    self.cpu_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.cpu_state.autohide_timer = Some(Instant::now());
                    }
                }
            }
            WidgetPosition::Mem => {
                let new_time =
                    self.mem_state.current_display_time - self.app_config_fields.time_interval;
                if new_time >= constants::STALE_MIN_MILLISECONDS {
                    self.mem_state.current_display_time = new_time;
                    self.mem_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.mem_state.autohide_timer = Some(Instant::now());
                    }
                } else if self.mem_state.current_display_time != constants::STALE_MIN_MILLISECONDS {
                    self.mem_state.current_display_time = constants::STALE_MIN_MILLISECONDS;
                    self.mem_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.mem_state.autohide_timer = Some(Instant::now());
                    }
                }
            }
            WidgetPosition::Network => {
                let new_time =
                    self.net_state.current_display_time - self.app_config_fields.time_interval;
                if new_time >= constants::STALE_MIN_MILLISECONDS {
                    self.net_state.current_display_time = new_time;
                    self.net_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.net_state.autohide_timer = Some(Instant::now());
                    }
                } else if self.net_state.current_display_time != constants::STALE_MIN_MILLISECONDS {
                    self.net_state.current_display_time = constants::STALE_MIN_MILLISECONDS;
                    self.net_state.force_update = true;
                    if self.app_config_fields.autohide_time {
                        self.net_state.autohide_timer = Some(Instant::now());
                    }
                }
            }
            _ => {}
        }
    }

    fn reset_cpu_zoom(&mut self) {
        self.cpu_state.current_display_time = self.app_config_fields.default_time_value;
        self.cpu_state.force_update = true;
        if self.app_config_fields.autohide_time {
            self.cpu_state.autohide_timer = Some(Instant::now());
        }
    }

    fn reset_mem_zoom(&mut self) {
        self.mem_state.current_display_time = self.app_config_fields.default_time_value;
        self.mem_state.force_update = true;
        if self.app_config_fields.autohide_time {
            self.mem_state.autohide_timer = Some(Instant::now());
        }
    }

    fn reset_net_zoom(&mut self) {
        self.net_state.current_display_time = self.app_config_fields.default_time_value;
        self.net_state.force_update = true;
        if self.app_config_fields.autohide_time {
            self.net_state.autohide_timer = Some(Instant::now());
        }
    }

    fn reset_zoom(&mut self) {
        match self.current_widget_selected {
            WidgetPosition::Cpu => self.reset_cpu_zoom(),
            WidgetPosition::Mem => self.reset_mem_zoom(),
            WidgetPosition::Network => self.reset_net_zoom(),
            _ => {}
        }
    }
}
