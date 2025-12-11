pub mod data;
pub mod filter;
pub mod layout_manager;
pub mod states;

use std::time::Instant;

use concat_string::concat_string;
use data::*;
use filter::*;
use hashbrown::HashMap;
use layout_manager::*;
pub use states::*;
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

use crate::{
    canvas::{
        components::time_graph::LegendPosition, dialogs::process_kill_dialog::ProcessKillDialog,
    },
    constants,
    utils::data_units::DataUnit,
    widgets::{ProcWidgetColumn, ProcWidgetMode, TreeCollapsed},
};

const STALE_MIN_MILLISECONDS: u64 = 30 * 1000; // Lowest is 30 seconds

#[derive(Debug, Clone, Eq, PartialEq, Default, Copy)]
pub enum AxisScaling {
    #[default]
    Log,
    Linear,
}

/// AppConfigFields is meant to cover basic fields that would normally be set
/// by config files or launch options.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum GraphStyle {
    #[default]
    Braille,
    Dot,
    Block,
    Filled,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AppConfigFields {
    pub update_rate: u64,
    pub temperature_type: TemperatureType,
    pub graph_style: GraphStyle,
    pub cpu_left_legend: bool,
    pub show_average_cpu: bool, // TODO: Unify this in CPU options
    pub use_current_cpu_total: bool,
    pub unnormalized_cpu: bool,
    pub get_process_threads: bool,
    pub use_basic_mode: bool,
    pub default_time_value: u64,
    pub time_interval: u64,
    pub hide_time: bool,
    pub autohide_time: bool,
    pub use_old_network_legend: bool,
    pub table_gap: u16,
    pub disable_click: bool,
    pub disable_keys: bool,
    pub enable_gpu: bool,
    pub enable_cache_memory: bool,
    pub show_table_scroll_position: bool,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
    pub is_advanced_kill: bool,
    pub is_read_only: bool,
    #[cfg(target_os = "linux")]
    pub hide_k_threads: bool,
    #[cfg(feature = "zfs")]
    pub free_arc: bool,
    pub memory_legend_position: Option<LegendPosition>,
    // TODO: Remove these, move network details state-side.
    pub network_unit_type: DataUnit,
    pub network_legend_position: Option<LegendPosition>,
    pub network_scale_type: AxisScaling,
    pub network_use_binary_prefix: bool,
    pub retention_ms: u64,
    pub dedicated_average_row: bool,
    pub default_tree_collapse: bool,
}

/// For filtering out information
#[derive(Debug, Clone)]
pub struct DataFilters {
    pub disk_filter: Option<Filter>,
    pub mount_filter: Option<Filter>,
    pub temp_filter: Option<Filter>,
    pub net_filter: Option<Filter>,
}

pub struct App {
    awaiting_second_char: bool,
    second_char: Option<char>,
    pub data_store: DataStore,
    last_key_press: Instant,
    pub(crate) process_kill_dialog: ProcessKillDialog,
    pub help_dialog_state: AppHelpDialogState,
    pub is_expanded: bool,
    pub is_force_redraw: bool,
    pub is_determining_widget_boundary: bool,
    pub basic_mode_use_percent: bool,
    pub states: AppWidgetStates,
    pub app_config_fields: AppConfigFields,
    pub widget_map: HashMap<u64, BottomWidget>,
    pub current_widget: BottomWidget,
    pub used_widgets: UsedWidgets,
    pub filters: DataFilters,
}

impl App {
    /// Create a new [`App`].
    pub fn new(
        app_config_fields: AppConfigFields, states: AppWidgetStates,
        widget_map: HashMap<u64, BottomWidget>, current_widget: BottomWidget,
        used_widgets: UsedWidgets, filters: DataFilters, is_expanded: bool,
    ) -> Self {
        Self {
            awaiting_second_char: false,
            second_char: None,
            data_store: DataStore::default(),
            last_key_press: Instant::now(),
            process_kill_dialog: ProcessKillDialog::default(),
            help_dialog_state: AppHelpDialogState::default(),
            is_expanded,
            is_force_redraw: false,
            is_determining_widget_boundary: false,
            basic_mode_use_percent: false,
            states,
            app_config_fields,
            widget_map,
            current_widget,
            used_widgets,
            filters,
        }
    }

    /// Update the data in the [`App`].
    pub fn update_data(&mut self) {
        let data_source = self.data_store.get_data();

        // FIXME: (points_rework_v1) maybe separate PR but would it make more sense to store references of data?
        // Would it also make more sense to move the "data set" step to the draw step, and make it only set if force
        // update is set here?
        for proc in self.states.proc_state.widget_states.values_mut() {
            if proc.force_update_data {
                proc.set_table_data(data_source);
            }
        }

        for temp in self.states.temp_state.widget_states.values_mut() {
            if temp.force_update_data {
                temp.set_table_data(&data_source.temp_data);
            }
        }

        for cpu in self.states.cpu_state.widget_states.values_mut() {
            if cpu.force_update_data {
                cpu.set_legend_data(&data_source.cpu_harvest);
            }
        }

        for disk in self.states.disk_state.widget_states.values_mut() {
            if disk.force_update_data {
                disk.set_table_data(data_source);
            }
        }

        #[cfg(feature = "gpu")]
        for gpu in self.states.gpu_state.widget_states.values_mut() {
            gpu.set_legend_data(&data_source.agnostic_gpu_harvest);
        }
    }

    pub fn reset(&mut self) {
        // Reset multi
        self.reset_multi_tap_keys();

        // Reset dialog state
        self.help_dialog_state.is_showing_help = false;
        self.process_kill_dialog.reset();

        // Close all searches and reset it
        self.states
            .proc_state
            .widget_states
            .values_mut()
            .for_each(|state| {
                state.proc_search.search_state.reset();
            });

        self.data_store.reset();

        // Reset zoom
        self.reset_cpu_zoom();
        self.reset_mem_zoom();
        self.reset_net_zoom();
    }

    pub fn should_get_widget_bounds(&self) -> bool {
        self.is_force_redraw || self.is_determining_widget_boundary
    }

    pub fn on_esc(&mut self) {
        self.reset_multi_tap_keys();

        if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_esc();
            self.is_force_redraw = true;
        } else if self.help_dialog_state.is_showing_help {
            self.help_dialog_state.is_showing_help = false;
            self.help_dialog_state.scroll_state.current_scroll_index = 0;
            self.is_force_redraw = true;
        } else {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(pws) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        if pws.is_search_enabled() || pws.is_sort_open {
                            pws.proc_search.search_state.is_enabled = false;
                            pws.is_sort_open = false;
                            self.is_force_redraw = true;
                            return;
                        }
                    }
                }
                BottomWidgetType::ProcSearch => {
                    if let Some(pws) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if pws.is_search_enabled() {
                            pws.proc_search.search_state.is_enabled = false;
                            self.move_widget_selection(&WidgetDirection::Up);
                            self.is_force_redraw = true;
                            return;
                        }
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(pws) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        if pws.is_sort_open {
                            pws.is_sort_open = false;
                            self.move_widget_selection(&WidgetDirection::Right);
                            self.is_force_redraw = true;
                            return;
                        }
                    }
                }
                _ => {}
            }

            if self.is_expanded {
                self.is_expanded = false;
                self.is_force_redraw = true;
            }
        }
    }

    pub fn is_in_search_widget(&self) -> bool {
        matches!(
            self.current_widget.widget_type,
            BottomWidgetType::ProcSearch
        )
    }

    fn reset_multi_tap_keys(&mut self) {
        self.awaiting_second_char = false;
        self.second_char = None;
    }

    fn is_in_dialog(&self) -> bool {
        self.help_dialog_state.is_showing_help || self.process_kill_dialog.is_open()
    }

    fn ignore_normal_keybinds(&self) -> bool {
        self.is_in_dialog()
    }

    pub fn on_tab(&mut self) {
        // Allow usage whilst only in processes

        if !self.ignore_normal_keybinds() {
            if let BottomWidgetType::Proc = self.current_widget.widget_type {
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    proc_widget_state.toggle_tab();
                }
            }
        }
    }

    pub fn on_slash(&mut self) {
        if !self.ignore_normal_keybinds() {
            match &self.current_widget.widget_type {
                BottomWidgetType::Proc | BottomWidgetType::ProcSort => {
                    // Toggle on
                    if let Some(proc_widget_state) = self.states.proc_state.get_mut_widget_state(
                        self.current_widget.widget_id
                            - match &self.current_widget.widget_type {
                                BottomWidgetType::ProcSort => 2,
                                _ => 0,
                            },
                    ) {
                        proc_widget_state.proc_search.search_state.is_enabled = true;
                        self.move_widget_selection(&WidgetDirection::Down);
                        self.is_force_redraw = true;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn toggle_sort_menu(&mut self) {
        let widget_id = self.current_widget.widget_id
            - match &self.current_widget.widget_type {
                BottomWidgetType::Proc => 0,
                BottomWidgetType::ProcSort => 2,
                _ => 0,
            };

        if let Some(pws) = self.states.proc_state.get_mut_widget_state(widget_id) {
            pws.is_sort_open = !pws.is_sort_open;
            pws.force_rerender = true;

            // If the sort is now open, move left. Otherwise, if the proc sort was selected,
            // force move right.
            if pws.is_sort_open {
                pws.sort_table.set_position(pws.table.sort_index());
                self.move_widget_selection(&WidgetDirection::Left);
            } else if let BottomWidgetType::ProcSort = self.current_widget.widget_type {
                self.move_widget_selection(&WidgetDirection::Right);
            }
            self.is_force_redraw = true;
        }
    }

    pub fn invert_sort(&mut self) {
        match &self.current_widget.widget_type {
            BottomWidgetType::Proc | BottomWidgetType::ProcSort => {
                let widget_id = self.current_widget.widget_id
                    - match &self.current_widget.widget_type {
                        BottomWidgetType::Proc => 0,
                        BottomWidgetType::ProcSort => 2,
                        _ => 0,
                    };

                if let Some(pws) = self.states.proc_state.get_mut_widget_state(widget_id) {
                    pws.table.toggle_order();
                    pws.force_data_update();
                }
            }
            _ => {}
        }
    }

    pub fn toggle_percentages(&mut self) {
        match &self.current_widget.widget_type {
            BottomWidgetType::BasicMem => {
                self.basic_mode_use_percent = !self.basic_mode_use_percent; // Oh god this is so lazy.
            }
            BottomWidgetType::Proc => {
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    proc_widget_state.toggle_mem_percentage();
                }
            }

            _ => {}
        }
    }

    pub fn toggle_ignore_case(&mut self) {
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                proc_widget_state.proc_search.search_toggle_ignore_case();
                proc_widget_state.update_query();
            }
        }
    }

    pub fn toggle_search_whole_word(&mut self) {
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                proc_widget_state.proc_search.search_toggle_whole_word();
                proc_widget_state.update_query();
            }
        }
    }

    pub fn toggle_search_regex(&mut self) {
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                proc_widget_state.proc_search.search_toggle_regex();
                proc_widget_state.update_query();
            }
        }
    }

    pub fn toggle_tree_mode(&mut self) {
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id))
        {
            match proc_widget_state.mode {
                ProcWidgetMode::Tree { .. } => {
                    proc_widget_state.mode = ProcWidgetMode::Normal;
                    proc_widget_state.force_rerender_and_update();
                }
                ProcWidgetMode::Normal => {
                    proc_widget_state.mode = ProcWidgetMode::Tree(TreeCollapsed::new(
                        self.app_config_fields.default_tree_collapse,
                    ));
                    proc_widget_state.force_rerender_and_update();
                }
                ProcWidgetMode::Grouped => {}
            }
        }
    }

    /// One of two functions allowed to run while in a dialog...
    pub fn on_enter(&mut self) {
        if self.process_kill_dialog.is_open() {
            // Not the best way of doing things for now but works as glue.
            self.process_kill_dialog.on_enter();
        } else if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::ProcSearch => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if proc_widget_state.is_search_enabled() {
                            proc_widget_state.proc_search.search_state.is_enabled = false;
                            self.move_widget_selection(&WidgetDirection::Up);
                            self.is_force_redraw = true;
                        }
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        proc_widget_state.use_sort_table_value();
                        if proc_widget_state.is_sort_open {
                            proc_widget_state.is_sort_open = false;
                            self.move_widget_selection(&WidgetDirection::Right);
                            self.is_force_redraw = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn on_delete(&mut self) {
        match self.current_widget.widget_type {
            BottomWidgetType::ProcSearch => {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget
                        && proc_widget_state.proc_search.search_state.is_enabled
                        && proc_widget_state.cursor_char_index()
                            < proc_widget_state
                                .proc_search
                                .search_state
                                .current_search_query
                                .len()
                    {
                        let current_cursor = proc_widget_state.cursor_char_index();
                        proc_widget_state.search_walk_forward();

                        let _ = proc_widget_state
                            .proc_search
                            .search_state
                            .current_search_query
                            .drain(current_cursor..proc_widget_state.cursor_char_index());

                        proc_widget_state.proc_search.search_state.grapheme_cursor =
                            GraphemeCursor::new(
                                current_cursor,
                                proc_widget_state
                                    .proc_search
                                    .search_state
                                    .current_search_query
                                    .len(),
                                true,
                            );

                        proc_widget_state.update_query();
                    }
                }
            }
            BottomWidgetType::Proc => {
                self.kill_current_process();
            }
            _ => {}
        }
    }

    pub fn on_backspace(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            let is_in_search_widget = self.is_in_search_widget();
            if let Some(proc_widget_state) = self
                .states
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                if is_in_search_widget
                    && proc_widget_state.proc_search.search_state.is_enabled
                    && proc_widget_state.cursor_char_index() > 0
                {
                    let current_cursor = proc_widget_state.cursor_char_index();
                    proc_widget_state.search_walk_back();

                    // Remove the indices in between.
                    let _ = proc_widget_state
                        .proc_search
                        .search_state
                        .current_search_query
                        .drain(proc_widget_state.cursor_char_index()..current_cursor);

                    proc_widget_state.proc_search.search_state.grapheme_cursor =
                        GraphemeCursor::new(
                            proc_widget_state.cursor_char_index(),
                            proc_widget_state
                                .proc_search
                                .search_state
                                .current_search_query
                                .len(),
                            true,
                        );

                    proc_widget_state.proc_search.search_state.cursor_direction =
                        CursorDirection::Left;

                    proc_widget_state.update_query();
                }
            }
        }
    }

    pub fn on_up_key(&mut self) {
        if !self.is_in_dialog() {
            self.decrement_position_count();
            self.reset_multi_tap_keys();
        } else if self.help_dialog_state.is_showing_help {
            self.help_scroll_up();
            self.reset_multi_tap_keys();
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_up_key();
        }
    }

    pub fn on_down_key(&mut self) {
        if !self.is_in_dialog() {
            self.increment_position_count();
            self.reset_multi_tap_keys();
        } else if self.help_dialog_state.is_showing_help {
            self.help_scroll_down();
            self.reset_multi_tap_keys();
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_down_key();
        }
    }

    pub fn on_left_key(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.collapse_current_tree_branch_entry();
                    }
                }
                BottomWidgetType::ProcSearch => {
                    let is_in_search_widget = self.is_in_search_widget();
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if is_in_search_widget {
                            let prev_cursor = proc_widget_state.cursor_char_index();
                            proc_widget_state.search_walk_back();
                            if proc_widget_state.cursor_char_index() < prev_cursor {
                                proc_widget_state.proc_search.search_state.cursor_direction =
                                    CursorDirection::Left;
                            }
                        }
                    }
                }
                BottomWidgetType::Battery => {
                    #[cfg(feature = "battery")]
                    if self.data_store.get_data().battery_harvest.len() > 1 {
                        if let Some(battery_widget_state) = self
                            .states
                            .battery_state
                            .get_mut_widget_state(self.current_widget.widget_id)
                        {
                            if battery_widget_state.currently_selected_battery_index > 0 {
                                battery_widget_state.currently_selected_battery_index -= 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_left_key();
        }
    }

    pub fn on_right_key(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.expand_current_tree_branch_entry();
                    }
                }
                BottomWidgetType::ProcSearch => {
                    let is_in_search_widget = self.is_in_search_widget();
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if is_in_search_widget {
                            let prev_cursor = proc_widget_state.cursor_char_index();
                            proc_widget_state.search_walk_forward();
                            if proc_widget_state.cursor_char_index() > prev_cursor {
                                proc_widget_state.proc_search.search_state.cursor_direction =
                                    CursorDirection::Right;
                            }
                        }
                    }
                }
                BottomWidgetType::Battery => {
                    #[cfg(feature = "battery")]
                    {
                        let battery_count = self.data_store.get_data().battery_harvest.len();
                        if battery_count > 1 {
                            if let Some(battery_widget_state) = self
                                .states
                                .battery_state
                                .get_mut_widget_state(self.current_widget.widget_id)
                            {
                                if battery_widget_state.currently_selected_battery_index
                                    < battery_count - 1
                                {
                                    battery_widget_state.currently_selected_battery_index += 1;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_right_key();
        }
    }

    pub fn on_space_key(&mut self) {
        if !self.is_in_dialog() {
            if self.current_widget.widget_type == BottomWidgetType::Proc {
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    proc_widget_state.toggle_current_tree_branch_entry();
                }
            }
        } else if self.process_kill_dialog.is_open() {
            // Either select the current option,
            // or scroll to the next one
        }
    }

    pub fn on_page_up(&mut self) {
        if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_page_up();
        } else if self.help_dialog_state.is_showing_help {
            let current = &mut self.help_dialog_state.scroll_state.current_scroll_index;
            let amount = self.help_dialog_state.height;
            *current = current.saturating_sub(amount);
        } else if self.current_widget.widget_type.is_widget_table() {
            if let (Some((_tlc_x, tlc_y)), Some((_brc_x, brc_y))) = (
                &self.current_widget.top_left_corner,
                &self.current_widget.bottom_right_corner,
            ) {
                let border_offset = u16::from(self.is_drawing_border());
                let header_offset = self.header_offset(&self.current_widget);
                let height = brc_y - tlc_y - 2 * border_offset - header_offset;
                self.change_position_count(-(height as i64));
            }
        }
    }

    pub fn on_page_down(&mut self) {
        if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_page_down();
        } else if self.help_dialog_state.is_showing_help {
            let current = self.help_dialog_state.scroll_state.current_scroll_index;
            let amount = self.help_dialog_state.height;

            self.help_scroll_to_or_max(current + amount);
        } else if self.current_widget.widget_type.is_widget_table() {
            if let (Some((_tlc_x, tlc_y)), Some((_brc_x, brc_y))) = (
                &self.current_widget.top_left_corner,
                &self.current_widget.bottom_right_corner,
            ) {
                let border_offset = u16::from(self.is_drawing_border());
                let header_offset = self.header_offset(&self.current_widget);
                let height = brc_y - tlc_y - 2 * border_offset - header_offset;
                self.change_position_count(height as i64);
            }
        }
    }

    pub fn scroll_half_page_up(&mut self) {
        if self.help_dialog_state.is_showing_help {
            let current = &mut self.help_dialog_state.scroll_state.current_scroll_index;
            let amount = self.help_dialog_state.height / 2;

            *current = current.saturating_sub(amount);
        } else if self.current_widget.widget_type.is_widget_table() {
            if let (Some((_tlc_x, tlc_y)), Some((_brc_x, brc_y))) = (
                &self.current_widget.top_left_corner,
                &self.current_widget.bottom_right_corner,
            ) {
                let border_offset = u16::from(self.is_drawing_border());
                let header_offset = self.header_offset(&self.current_widget);
                let height = brc_y - tlc_y - 2 * border_offset - header_offset;
                self.change_position_count(-(height as i64) / 2);
            }
        }
    }

    pub fn scroll_half_page_down(&mut self) {
        if self.help_dialog_state.is_showing_help {
            let current = self.help_dialog_state.scroll_state.current_scroll_index;
            let amount = self.help_dialog_state.height / 2;

            self.help_scroll_to_or_max(current + amount);
        } else if self.current_widget.widget_type.is_widget_table() {
            if let (Some((_tlc_x, tlc_y)), Some((_brc_x, brc_y))) = (
                &self.current_widget.top_left_corner,
                &self.current_widget.bottom_right_corner,
            ) {
                let border_offset = u16::from(self.is_drawing_border());
                let header_offset = self.header_offset(&self.current_widget);
                let height = brc_y - tlc_y - 2 * border_offset - header_offset;
                self.change_position_count(height as i64 / 2);
            }
        }
    }

    pub fn skip_cursor_beginning(&mut self) {
        if !self.ignore_normal_keybinds() {
            if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget {
                        proc_widget_state.proc_search.search_state.grapheme_cursor =
                            GraphemeCursor::new(
                                0,
                                proc_widget_state
                                    .proc_search
                                    .search_state
                                    .current_search_query
                                    .len(),
                                true,
                            );

                        proc_widget_state.proc_search.search_state.cursor_direction =
                            CursorDirection::Left;
                    }
                }
            }
        }
    }

    pub fn skip_cursor_end(&mut self) {
        if !self.ignore_normal_keybinds() {
            if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget {
                        let query_len = proc_widget_state
                            .proc_search
                            .search_state
                            .current_search_query
                            .len();

                        proc_widget_state.proc_search.search_state.grapheme_cursor =
                            GraphemeCursor::new(query_len, query_len, true);
                        proc_widget_state.proc_search.search_state.cursor_direction =
                            CursorDirection::Right;
                    }
                }
            }
        }
    }

    pub fn clear_search(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            if let Some(proc_widget_state) = self
                .states
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                proc_widget_state.clear_search();
            }
        }
    }

    pub fn clear_previous_word(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            if let Some(proc_widget_state) = self
                .states
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                // Traverse backwards from the current cursor location until you hit
                // non-whitespace characters, then continue to traverse (and
                // delete) backwards until you hit a whitespace character.  Halt.

                // So... first, let's get our current cursor position in terms of char indices.
                let end_index = proc_widget_state.cursor_char_index();

                // Then, let's crawl backwards until we hit our location, and store the
                // "head"...
                let query = proc_widget_state.current_search_query();
                let mut start_index = 0;
                let mut saw_non_whitespace = false;

                for (itx, c) in query
                    .chars()
                    .rev()
                    .enumerate()
                    .skip(query.len() - end_index)
                {
                    if c.is_whitespace() {
                        if saw_non_whitespace {
                            start_index = query.len() - itx;
                            break;
                        }
                    } else {
                        saw_non_whitespace = true;
                    }
                }

                let _ = proc_widget_state
                    .proc_search
                    .search_state
                    .current_search_query
                    .drain(start_index..end_index);

                proc_widget_state.proc_search.search_state.grapheme_cursor = GraphemeCursor::new(
                    start_index,
                    proc_widget_state
                        .proc_search
                        .search_state
                        .current_search_query
                        .len(),
                    true,
                );

                proc_widget_state.proc_search.search_state.cursor_direction = CursorDirection::Left;

                proc_widget_state.update_query();
            }
        }
    }

    pub fn on_char_key(&mut self, caught_char: char) {
        // Skip control code chars
        if caught_char.is_control() {
            return;
        }

        const MAX_KEY_TIMEOUT_IN_MILLISECONDS: u64 = 1000;

        // Forbid any char key presses when showing a dialog box...
        if !self.ignore_normal_keybinds() {
            let current_key_press_inst = Instant::now();
            if current_key_press_inst
                .duration_since(self.last_key_press)
                .as_millis()
                > MAX_KEY_TIMEOUT_IN_MILLISECONDS.into()
            {
                self.reset_multi_tap_keys();
            }
            self.last_key_press = current_key_press_inst;

            if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget && proc_widget_state.is_search_enabled() {
                        proc_widget_state
                            .proc_search
                            .search_state
                            .current_search_query
                            .insert(proc_widget_state.cursor_char_index(), caught_char);

                        proc_widget_state.proc_search.search_state.grapheme_cursor =
                            GraphemeCursor::new(
                                proc_widget_state.cursor_char_index(),
                                proc_widget_state
                                    .proc_search
                                    .search_state
                                    .current_search_query
                                    .len(),
                                true,
                            );
                        proc_widget_state.search_walk_forward();

                        proc_widget_state.update_query();
                        proc_widget_state.proc_search.search_state.cursor_direction =
                            CursorDirection::Right;

                        return;
                    }
                }
            }
            self.handle_char(caught_char);
        } else if self.help_dialog_state.is_showing_help {
            match caught_char {
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    let potential_index = caught_char.to_digit(10);
                    if let Some(potential_index) = potential_index {
                        let potential_index = potential_index as usize;
                        if (potential_index) < self.help_dialog_state.index_shortcuts.len() {
                            self.help_scroll_to_or_max(
                                self.help_dialog_state.index_shortcuts[potential_index],
                            );
                        }
                    }
                }
                'j' | 'k' | 'g' | 'G' => self.handle_char(caught_char),
                _ => {}
            }
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_char(caught_char);
        }
    }

    /// Kill the currently selected process if we are in the process widget.
    ///
    /// TODO: This ideally gets abstracted out into a separate widget.
    pub(crate) fn kill_current_process(&mut self) {
        if self.app_config_fields.is_read_only {
            return;
        }

        if let Some(pws) = self
            .states
            .proc_state
            .widget_states
            .get(&self.current_widget.widget_id)
        {
            if let Some(current) = pws.table.current_item() {
                let id = current.id.to_string();
                if let Some(pids) = pws
                    .id_pid_map
                    .get(&id)
                    .cloned()
                    .or_else(|| Some(vec![current.pid]))
                {
                    let current_process = (id, pids);

                    let use_simple_selection = {
                        cfg_if::cfg_if! {
                            if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))] {
                                !self.app_config_fields.is_advanced_kill
                            } else {
                                true
                            }
                        }
                    };

                    self.process_kill_dialog.start_process_kill(
                        current_process.0,
                        current_process.1,
                        use_simple_selection,
                    );

                    // TODO: I don't think most of this is needed.
                    self.is_determining_widget_boundary = true;
                }
            }
        }
    }

    // FIXME: Refactor this system...
    fn handle_char(&mut self, caught_char: char) {
        match caught_char {
            '/' => {
                self.on_slash();
            }
            'd' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    let mut is_first_d = true;
                    if let Some(second_char) = self.second_char {
                        if self.awaiting_second_char && second_char == 'd' {
                            is_first_d = false;
                            self.awaiting_second_char = false;
                            self.second_char = None;

                            self.reset_multi_tap_keys();

                            self.kill_current_process();
                        }
                    }

                    if is_first_d {
                        self.awaiting_second_char = true;
                        self.second_char = Some('d');
                    }
                } else if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(0);
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
            'k' => self.on_up_key(),
            'j' => self.on_down_key(),
            'f' => self.data_store.toggle_frozen(),
            'c' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.select_column(ProcWidgetColumn::Cpu);
                    }
                }
            }
            'm' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.select_column(ProcWidgetColumn::Mem);
                    }
                } else if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(1);
                }
            }
            'p' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.select_column(ProcWidgetColumn::PidOrCount);
                    }
                } else if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(5);
                }
            }
            'P' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.toggle_command();
                    }
                }
            }
            'n' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.select_column(ProcWidgetColumn::ProcNameOrCommand);
                    }
                } else if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(3);
                }
            }
            #[cfg(feature = "gpu")]
            'M' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.select_column(ProcWidgetColumn::GpuMem);
                    }
                }
            }
            #[cfg(feature = "gpu")]
            'C' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.select_column(ProcWidgetColumn::GpuUtil);
                    }
                }
            }
            '?' => {
                self.help_dialog_state.is_showing_help = true;
                self.is_force_redraw = true;
            }
            'H' | 'A' => self.move_widget_selection(&WidgetDirection::Left),
            'L' | 'D' => self.move_widget_selection(&WidgetDirection::Right),
            'K' | 'W' => self.move_widget_selection(&WidgetDirection::Up),
            'J' | 'S' => self.move_widget_selection(&WidgetDirection::Down),
            't' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    self.toggle_tree_mode()
                } else if let Some(temp) = self
                    .states
                    .temp_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    temp.table.set_sort_index(1);
                    temp.force_data_update();
                } else if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(4);
                }
            }
            '+' => self.on_plus(),
            '-' => self.on_minus(),
            '=' => self.reset_zoom(),
            'e' => self.toggle_expand_widget(),
            's' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    self.toggle_sort_menu()
                } else if let Some(temp) = self
                    .states
                    .temp_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    temp.table.set_sort_index(0);
                    temp.force_data_update();
                    self.is_force_redraw = true;
                }
            }
            'u' => {
                if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(2);
                }
            }
            'r' => {
                if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(6);
                }
            }
            'w' => {
                if let Some(disk) = self
                    .states
                    .disk_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    disk.set_index(7);
                }
            }
            'I' => self.invert_sort(),
            '%' => self.toggle_percentages(),
            #[cfg(target_os = "linux")]
            'z' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .widget_states
                        .get_mut(&self.current_widget.widget_id)
                    {
                        proc_widget_state.toggle_k_thread();
                    }
                }
            }
            _ => {}
        }

        if let Some(second_char) = self.second_char {
            if self.awaiting_second_char && caught_char != second_char {
                self.awaiting_second_char = false;
            }
        }
    }

    fn toggle_expand_widget(&mut self) {
        if self.is_expanded {
            self.is_expanded = false;
            self.is_force_redraw = true;
        } else {
            self.expand_widget();
        }
    }

    fn expand_widget(&mut self) {
        // TODO: [BASIC] Expansion in basic mode.
        if !self.ignore_normal_keybinds() && !self.app_config_fields.use_basic_mode {
            // Pop-out mode.  We ignore if in process search.

            match self.current_widget.widget_type {
                BottomWidgetType::ProcSearch => {}
                _ => {
                    self.is_expanded = true;
                    self.is_force_redraw = true;
                }
            }
        }
    }

    pub fn move_widget_selection(&mut self, direction: &WidgetDirection) {
        // Since we only want to call reset once, we do it like this to avoid
        // redundant calls on recursion.
        self.move_widget_selection_logic(direction);
        self.reset_multi_tap_keys();
    }

    fn move_widget_selection_logic(&mut self, direction: &WidgetDirection) {
        // The actual logic for widget movement.

        // We follow these following steps:
        // 1. Send a movement signal in `direction`.
        // 2. Check if this new widget we've landed on is hidden.  If not, halt.
        // 3. If it hidden, loop and either send:
        //    - A signal equal to the current direction, if it is opposite of the reflection.
        //    - Reflection direction.

        if !self.ignore_normal_keybinds() && !self.is_expanded {
            if let Some(new_widget_id) = &(match direction {
                WidgetDirection::Left => self.current_widget.left_neighbour,
                WidgetDirection::Right => self.current_widget.right_neighbour,
                WidgetDirection::Up => self.current_widget.up_neighbour,
                WidgetDirection::Down => self.current_widget.down_neighbour,
            }) {
                if let Some(new_widget) = self.widget_map.get(new_widget_id) {
                    match &new_widget.widget_type {
                        BottomWidgetType::Temp
                        | BottomWidgetType::Proc
                        | BottomWidgetType::ProcSort
                        | BottomWidgetType::Disk
                        | BottomWidgetType::Battery
                            if self.states.basic_table_widget_state.is_some()
                                && (*direction == WidgetDirection::Left
                                    || *direction == WidgetDirection::Right) =>
                        {
                            // Gotta do this for the sort widget
                            if let BottomWidgetType::ProcSort = new_widget.widget_type {
                                if let Some(proc_widget_state) = self
                                    .states
                                    .proc_state
                                    .widget_states
                                    .get(&(new_widget_id - 2))
                                {
                                    if proc_widget_state.is_sort_open {
                                        self.current_widget = new_widget.clone();
                                    } else if let Some(next_new_widget_id) = match direction {
                                        WidgetDirection::Left => new_widget.left_neighbour,
                                        _ => new_widget.right_neighbour,
                                    } {
                                        if let Some(next_new_widget) =
                                            self.widget_map.get(&next_new_widget_id)
                                        {
                                            self.current_widget = next_new_widget.clone();
                                        }
                                    }
                                }
                            } else {
                                self.current_widget = new_widget.clone();
                            }

                            if let Some(basic_table_widget_state) =
                                &mut self.states.basic_table_widget_state
                            {
                                basic_table_widget_state.currently_displayed_widget_id =
                                    self.current_widget.widget_id;
                                basic_table_widget_state.currently_displayed_widget_type =
                                    self.current_widget.widget_type.clone();
                            }

                            // And let's not forget:
                            self.is_determining_widget_boundary = true;
                        }
                        BottomWidgetType::BasicTables => {
                            match &direction {
                                WidgetDirection::Up => {
                                    // Note this case would fail if it moved up into a hidden
                                    // widget, but it's for basic so whatever, it's all hard-coded
                                    // right now anyways...
                                    if let Some(next_new_widget_id) = new_widget.up_neighbour {
                                        if let Some(next_new_widget) =
                                            self.widget_map.get(&next_new_widget_id)
                                        {
                                            self.current_widget = next_new_widget.clone();
                                        }
                                    }
                                }
                                WidgetDirection::Down => {
                                    // Assuming we're in basic mode (BasicTables), then
                                    // we want to move DOWN to the currently shown widget.
                                    if let Some(basic_table_widget_state) =
                                        &mut self.states.basic_table_widget_state
                                    {
                                        // We also want to move towards Proc if we had set it to
                                        // ProcSort.
                                        if let BottomWidgetType::ProcSort =
                                            basic_table_widget_state.currently_displayed_widget_type
                                        {
                                            basic_table_widget_state
                                                .currently_displayed_widget_type =
                                                BottomWidgetType::Proc;
                                            basic_table_widget_state
                                                .currently_displayed_widget_id -= 2;
                                        }

                                        if let Some(next_new_widget) = self.widget_map.get(
                                            &basic_table_widget_state.currently_displayed_widget_id,
                                        ) {
                                            self.current_widget = next_new_widget.clone();
                                        }
                                    }
                                }
                                _ => self.current_widget = new_widget.clone(),
                            }
                        }
                        _ if new_widget.parent_reflector.is_some() => {
                            // It may be hidden...
                            if let Some((parent_direction, offset)) = &new_widget.parent_reflector {
                                if direction.is_opposite(parent_direction) {
                                    // Keep going in the current direction if hidden...
                                    // unless we hit a wall of sorts.
                                    let option_next_neighbour_id = match &direction {
                                        WidgetDirection::Left => new_widget.left_neighbour,
                                        WidgetDirection::Right => new_widget.right_neighbour,
                                        WidgetDirection::Up => new_widget.up_neighbour,
                                        WidgetDirection::Down => new_widget.down_neighbour,
                                    };
                                    match &new_widget.widget_type {
                                        BottomWidgetType::CpuLegend => {
                                            if let Some(cpu_widget_state) = self
                                                .states
                                                .cpu_state
                                                .widget_states
                                                .get(&(new_widget_id - *offset))
                                            {
                                                if cpu_widget_state.is_legend_hidden {
                                                    if let Some(next_neighbour_id) =
                                                        option_next_neighbour_id
                                                    {
                                                        if let Some(next_neighbour_widget) =
                                                            self.widget_map.get(&next_neighbour_id)
                                                        {
                                                            self.current_widget =
                                                                next_neighbour_widget.clone();
                                                        }
                                                    }
                                                } else {
                                                    self.current_widget = new_widget.clone();
                                                }
                                            }
                                        }
                                        BottomWidgetType::ProcSearch
                                        | BottomWidgetType::ProcSort => {
                                            if let Some(proc_widget_state) = self
                                                .states
                                                .proc_state
                                                .widget_states
                                                .get(&(new_widget_id - *offset))
                                            {
                                                match &new_widget.widget_type {
                                                    BottomWidgetType::ProcSearch => {
                                                        if !proc_widget_state.is_search_enabled() {
                                                            if let Some(next_neighbour_id) =
                                                                option_next_neighbour_id
                                                            {
                                                                if let Some(next_neighbour_widget) =
                                                                    self.widget_map
                                                                        .get(&next_neighbour_id)
                                                                {
                                                                    self.current_widget =
                                                                        next_neighbour_widget
                                                                            .clone();
                                                                }
                                                            }
                                                        } else {
                                                            self.current_widget =
                                                                new_widget.clone();
                                                        }
                                                    }
                                                    BottomWidgetType::ProcSort => {
                                                        if !proc_widget_state.is_sort_open {
                                                            if let Some(next_neighbour_id) =
                                                                option_next_neighbour_id
                                                            {
                                                                if let Some(next_neighbour_widget) =
                                                                    self.widget_map
                                                                        .get(&next_neighbour_id)
                                                                {
                                                                    self.current_widget =
                                                                        next_neighbour_widget
                                                                            .clone();
                                                                }
                                                            }
                                                        } else {
                                                            self.current_widget =
                                                                new_widget.clone();
                                                        }
                                                    }
                                                    _ => {
                                                        self.current_widget = new_widget.clone();
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            self.current_widget = new_widget.clone();
                                        }
                                    }
                                } else {
                                    // Reflect
                                    match &new_widget.widget_type {
                                        BottomWidgetType::CpuLegend => {
                                            if let Some(cpu_widget_state) = self
                                                .states
                                                .cpu_state
                                                .widget_states
                                                .get(&(new_widget_id - *offset))
                                            {
                                                if cpu_widget_state.is_legend_hidden {
                                                    if let Some(parent_cpu_widget) = self
                                                        .widget_map
                                                        .get(&(new_widget_id - *offset))
                                                    {
                                                        self.current_widget =
                                                            parent_cpu_widget.clone();
                                                    }
                                                } else {
                                                    self.current_widget = new_widget.clone();
                                                }
                                            }
                                        }
                                        BottomWidgetType::ProcSearch
                                        | BottomWidgetType::ProcSort => {
                                            if let Some(proc_widget_state) = self
                                                .states
                                                .proc_state
                                                .widget_states
                                                .get(&(new_widget_id - *offset))
                                            {
                                                match &new_widget.widget_type {
                                                    BottomWidgetType::ProcSearch => {
                                                        if !proc_widget_state.is_search_enabled() {
                                                            if let Some(parent_proc_widget) = self
                                                                .widget_map
                                                                .get(&(new_widget_id - *offset))
                                                            {
                                                                self.current_widget =
                                                                    parent_proc_widget.clone();
                                                            }
                                                        } else {
                                                            self.current_widget =
                                                                new_widget.clone();
                                                        }
                                                    }
                                                    BottomWidgetType::ProcSort => {
                                                        if !proc_widget_state.is_sort_open {
                                                            if let Some(parent_proc_widget) = self
                                                                .widget_map
                                                                .get(&(new_widget_id - *offset))
                                                            {
                                                                self.current_widget =
                                                                    parent_proc_widget.clone();
                                                            }
                                                        } else {
                                                            self.current_widget =
                                                                new_widget.clone();
                                                        }
                                                    }
                                                    _ => {
                                                        self.current_widget = new_widget.clone();
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            self.current_widget = new_widget.clone();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // Cannot be hidden, does not special treatment.
                            self.current_widget = new_widget.clone();
                        }
                    }

                    let mut reflection_dir: Option<WidgetDirection> = None;
                    if let Some((parent_direction, offset)) = &self.current_widget.parent_reflector
                    {
                        match &self.current_widget.widget_type {
                            BottomWidgetType::CpuLegend => {
                                if let Some(cpu_widget_state) = self
                                    .states
                                    .cpu_state
                                    .widget_states
                                    .get(&(self.current_widget.widget_id - *offset))
                                {
                                    if cpu_widget_state.is_legend_hidden {
                                        reflection_dir = Some(parent_direction.clone());
                                    }
                                }
                            }
                            BottomWidgetType::ProcSearch | BottomWidgetType::ProcSort => {
                                if let Some(proc_widget_state) = self
                                    .states
                                    .proc_state
                                    .widget_states
                                    .get(&(self.current_widget.widget_id - *offset))
                                {
                                    match &self.current_widget.widget_type {
                                        BottomWidgetType::ProcSearch => {
                                            if !proc_widget_state.is_search_enabled() {
                                                reflection_dir = Some(parent_direction.clone());
                                            }
                                        }
                                        BottomWidgetType::ProcSort => {
                                            if !proc_widget_state.is_sort_open {
                                                reflection_dir = Some(parent_direction.clone());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Some(ref_dir) = &reflection_dir {
                        self.move_widget_selection_logic(ref_dir);
                    }
                }
            }
        } else {
            match direction {
                WidgetDirection::Left => self.handle_left_expanded_movement(),
                WidgetDirection::Right => self.handle_right_expanded_movement(),
                WidgetDirection::Up => {
                    if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                        if let Some(current_widget) =
                            self.widget_map.get(&self.current_widget.widget_id)
                        {
                            if let Some(new_widget_id) = current_widget.up_neighbour {
                                if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                                    self.current_widget = new_widget.clone();
                                }
                            }
                        }
                    }
                }
                WidgetDirection::Down => match &self.current_widget.widget_type {
                    BottomWidgetType::Proc | BottomWidgetType::ProcSort => {
                        let widget_id = self.current_widget.widget_id
                            - match &self.current_widget.widget_type {
                                BottomWidgetType::ProcSort => 2,
                                _ => 0,
                            };
                        if let Some(current_widget) = self.widget_map.get(&widget_id) {
                            if let Some(new_widget_id) = current_widget.down_neighbour {
                                if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                                    if let Some(proc_widget_state) =
                                        self.states.proc_state.get_widget_state(widget_id)
                                    {
                                        if proc_widget_state.is_search_enabled() {
                                            self.current_widget = new_widget.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    fn handle_left_expanded_movement(&mut self) {
        if let BottomWidgetType::Proc = self.current_widget.widget_type {
            if let Some(new_widget_id) = self.current_widget.left_neighbour {
                if let Some(proc_widget_state) = self
                    .states
                    .proc_state
                    .widget_states
                    .get(&self.current_widget.widget_id)
                {
                    if proc_widget_state.is_sort_open {
                        if let Some(proc_sort_widget) = self.widget_map.get(&new_widget_id) {
                            self.current_widget = proc_sort_widget.clone(); // TODO: Could I remove this clone w/ static references?
                        }
                    }
                }
            }
        } else if self.app_config_fields.cpu_left_legend {
            if let BottomWidgetType::Cpu = self.current_widget.widget_type {
                if let Some(current_widget) = self.widget_map.get(&self.current_widget.widget_id) {
                    if let Some(cpu_widget_state) = self
                        .states
                        .cpu_state
                        .widget_states
                        .get(&self.current_widget.widget_id)
                    {
                        if !cpu_widget_state.is_legend_hidden {
                            if let Some(new_widget_id) = current_widget.left_neighbour {
                                if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                                    self.current_widget = new_widget.clone();
                                }
                            }
                        }
                    }
                }
            }
        } else if let BottomWidgetType::CpuLegend = self.current_widget.widget_type {
            if let Some(current_widget) = self.widget_map.get(&self.current_widget.widget_id) {
                if let Some(new_widget_id) = current_widget.left_neighbour {
                    if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                        self.current_widget = new_widget.clone();
                    }
                }
            }
        }
    }

    fn handle_right_expanded_movement(&mut self) {
        if let BottomWidgetType::ProcSort = self.current_widget.widget_type {
            if let Some(new_widget_id) = self.current_widget.right_neighbour {
                if let Some(proc_sort_widget) = self.widget_map.get(&new_widget_id) {
                    self.current_widget = proc_sort_widget.clone();
                }
            }
        } else if self.app_config_fields.cpu_left_legend {
            if let BottomWidgetType::CpuLegend = self.current_widget.widget_type {
                if let Some(current_widget) = self.widget_map.get(&self.current_widget.widget_id) {
                    if let Some(new_widget_id) = current_widget.right_neighbour {
                        if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                            self.current_widget = new_widget.clone();
                        }
                    }
                }
            }
        } else if let BottomWidgetType::Cpu = self.current_widget.widget_type {
            if let Some(current_widget) = self.widget_map.get(&self.current_widget.widget_id) {
                if let Some(cpu_widget_state) = self
                    .states
                    .cpu_state
                    .widget_states
                    .get(&self.current_widget.widget_id)
                {
                    if !cpu_widget_state.is_legend_hidden {
                        if let Some(new_widget_id) = current_widget.right_neighbour {
                            if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                                self.current_widget = new_widget.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn skip_to_first(&mut self) {
        if !self.ignore_normal_keybinds() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.table.scroll_to_first();
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        proc_widget_state.sort_table.scroll_to_first();
                    }
                }
                BottomWidgetType::Temp => {
                    if let Some(temp_widget_state) = self
                        .states
                        .temp_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        temp_widget_state.table.scroll_to_first();
                    }
                }
                BottomWidgetType::Disk => {
                    if let Some(disk_widget_state) = self
                        .states
                        .disk_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        disk_widget_state.table.scroll_to_first();
                    }
                }
                BottomWidgetType::CpuLegend => {
                    if let Some(cpu_widget_state) = self
                        .states
                        .cpu_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        cpu_widget_state.table.scroll_to_first();
                    }
                }
                #[cfg(feature = "gpu")]
                BottomWidgetType::Gpu => {
                    if let Some(gpu_widget_state) = self
                        .states
                        .gpu_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        gpu_widget_state.table.scroll_to_first();
                    }
                }

                _ => {}
            }
            self.reset_multi_tap_keys();
        } else if self.help_dialog_state.is_showing_help {
            self.help_dialog_state.scroll_state.current_scroll_index = 0;
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.go_to_first();
        }
    }

    pub fn skip_to_last(&mut self) {
        if !self.ignore_normal_keybinds() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.table.scroll_to_last();
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(proc_widget_state) = self
                        .states
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        proc_widget_state.sort_table.scroll_to_last();
                    }
                }
                BottomWidgetType::Temp => {
                    if let Some(temp_widget_state) = self
                        .states
                        .temp_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        temp_widget_state.table.scroll_to_last();
                    }
                }
                BottomWidgetType::Disk => {
                    if let Some(disk_widget_state) = self
                        .states
                        .disk_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        if !self.data_store.get_data().disk_harvest.is_empty() {
                            disk_widget_state.table.scroll_to_last();
                        }
                    }
                }
                BottomWidgetType::CpuLegend => {
                    if let Some(cpu_widget_state) = self
                        .states
                        .cpu_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        cpu_widget_state.table.scroll_to_last();
                    }
                }
                #[cfg(feature = "gpu")]
                BottomWidgetType::Gpu => {
                    if let Some(gpu_widget_state) = self
                        .states
                        .gpu_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        gpu_widget_state.table.scroll_to_last();
                    }
                }
                _ => {}
            }
            self.reset_multi_tap_keys();
        } else if self.help_dialog_state.is_showing_help {
            self.help_dialog_state.scroll_state.current_scroll_index =
                self.help_dialog_state.scroll_state.max_scroll_index;
        } else if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.go_to_last();
        }
    }

    pub fn decrement_position_count(&mut self) {
        self.change_position_count(-1);
    }

    pub fn increment_position_count(&mut self) {
        self.change_position_count(1);
    }

    fn change_position_count(&mut self, amount: i64) {
        if !self.ignore_normal_keybinds() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    self.change_process_position(amount);
                }
                BottomWidgetType::ProcSort => self.change_process_sort_position(amount),
                BottomWidgetType::Temp => self.change_temp_position(amount),
                BottomWidgetType::Disk => self.change_disk_position(amount),
                BottomWidgetType::CpuLegend => self.change_cpu_legend_position(amount),
                #[cfg(feature = "gpu")]
                BottomWidgetType::Gpu => self.change_gpu_legend_position(amount),
                _ => {}
            }
        }
    }

    fn change_process_sort_position(&mut self, num_to_change_by: i64) {
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .get_mut_widget_state(self.current_widget.widget_id - 2)
        {
            proc_widget_state
                .sort_table
                .increment_position(num_to_change_by);
        }
    }

    #[cfg(feature = "gpu")]
    fn change_gpu_legend_position(&mut self, num_to_change_by: i64) {
        if let Some(gpu_widget_state) = self
            .states
            .gpu_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id))
        {
            gpu_widget_state.table.increment_position(num_to_change_by);
        }
    }

    fn change_cpu_legend_position(&mut self, num_to_change_by: i64) {
        if let Some(cpu_widget_state) = self
            .states
            .cpu_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            cpu_widget_state.table.increment_position(num_to_change_by);
        }
    }

    /// Returns the new position.
    fn change_process_position(&mut self, num_to_change_by: i64) -> Option<usize> {
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .get_mut_widget_state(self.current_widget.widget_id)
        {
            proc_widget_state.table.increment_position(num_to_change_by)
        } else {
            None
        }
    }

    fn change_temp_position(&mut self, num_to_change_by: i64) {
        if let Some(temp_widget_state) = self
            .states
            .temp_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            temp_widget_state.table.increment_position(num_to_change_by);
        }
    }

    fn change_disk_position(&mut self, num_to_change_by: i64) {
        if let Some(disk_widget_state) = self
            .states
            .disk_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            disk_widget_state.table.increment_position(num_to_change_by);
        }
    }

    fn help_scroll_up(&mut self) {
        if self.help_dialog_state.scroll_state.current_scroll_index > 0 {
            self.help_dialog_state.scroll_state.current_scroll_index -= 1;
        }
    }

    fn help_scroll_down(&mut self) {
        if self.help_dialog_state.scroll_state.current_scroll_index
            < self.help_dialog_state.scroll_state.max_scroll_index
        {
            self.help_dialog_state.scroll_state.current_scroll_index += 1;
        }
    }

    fn help_scroll_to_or_max(&mut self, new_position: u16) {
        if new_position <= self.help_dialog_state.scroll_state.max_scroll_index {
            self.help_dialog_state.scroll_state.current_scroll_index = new_position;
        } else {
            self.help_dialog_state.scroll_state.current_scroll_index =
                self.help_dialog_state.scroll_state.max_scroll_index;
        }
    }

    pub fn handle_scroll_up(&mut self) {
        if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_scroll_up();
        } else if self.help_dialog_state.is_showing_help {
            self.help_scroll_up();
        } else if self.current_widget.widget_type.is_widget_graph() {
            self.zoom_in();
        } else if self.current_widget.widget_type.is_widget_table() {
            self.decrement_position_count();
        }
    }

    pub fn handle_scroll_down(&mut self) {
        if self.process_kill_dialog.is_open() {
            self.process_kill_dialog.on_scroll_down();
        } else if self.help_dialog_state.is_showing_help {
            self.help_scroll_down();
        } else if self.current_widget.widget_type.is_widget_graph() {
            self.zoom_out();
        } else if self.current_widget.widget_type.is_widget_table() {
            self.increment_position_count();
        }
    }

    fn on_plus(&mut self) {
        if let BottomWidgetType::Proc = self.current_widget.widget_type {
            // Toggle collapsing if tree
            self.toggle_collapsing_process_branch();
        } else {
            self.zoom_in();
        }
    }

    fn on_minus(&mut self) {
        if let BottomWidgetType::Proc = self.current_widget.widget_type {
            // Toggle collapsing if tree
            self.toggle_collapsing_process_branch();
        } else {
            self.zoom_out();
        }
    }

    fn toggle_collapsing_process_branch(&mut self) {
        if let Some(pws) = self
            .states
            .proc_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            pws.toggle_current_tree_branch_entry();
        }
    }

    fn zoom_out(&mut self) {
        match self.current_widget.widget_type {
            BottomWidgetType::Cpu => {
                if let Some(cpu_widget_state) = self
                    .states
                    .cpu_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = cpu_widget_state
                        .current_display_time
                        .saturating_add(self.app_config_fields.time_interval);

                    if new_time <= self.app_config_fields.retention_ms {
                        cpu_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if cpu_widget_state.current_display_time
                        != self.app_config_fields.retention_ms
                    {
                        cpu_widget_state.current_display_time = self.app_config_fields.retention_ms;
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Mem => {
                if let Some(mem_widget_state) = self
                    .states
                    .mem_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = mem_widget_state
                        .current_display_time
                        .saturating_add(self.app_config_fields.time_interval);

                    if new_time <= self.app_config_fields.retention_ms {
                        mem_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if mem_widget_state.current_display_time
                        != self.app_config_fields.retention_ms
                    {
                        mem_widget_state.current_display_time = self.app_config_fields.retention_ms;
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Net => {
                if let Some(net_widget_state) = self
                    .states
                    .net_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = net_widget_state
                        .current_display_time
                        .saturating_add(self.app_config_fields.time_interval);

                    if new_time <= self.app_config_fields.retention_ms {
                        net_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if net_widget_state.current_display_time
                        != self.app_config_fields.retention_ms
                    {
                        net_widget_state.current_display_time = self.app_config_fields.retention_ms;
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            #[cfg(feature = "gpu")]
            BottomWidgetType::Gpu => {
                if let Some(gpu_widget_state) = self
                    .states
                    .gpu_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = gpu_widget_state
                        .current_display_time
                        .saturating_add(self.app_config_fields.time_interval);

                    if new_time <= self.app_config_fields.retention_ms {
                        gpu_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            gpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if gpu_widget_state.current_display_time
                        != self.app_config_fields.retention_ms
                    {
                        gpu_widget_state.current_display_time = self.app_config_fields.retention_ms;
                        if self.app_config_fields.autohide_time {
                            gpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn zoom_in(&mut self) {
        match self.current_widget.widget_type {
            BottomWidgetType::Cpu => {
                if let Some(cpu_widget_state) = self
                    .states
                    .cpu_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = cpu_widget_state
                        .current_display_time
                        .saturating_sub(self.app_config_fields.time_interval);

                    if new_time >= STALE_MIN_MILLISECONDS {
                        cpu_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if cpu_widget_state.current_display_time != STALE_MIN_MILLISECONDS {
                        cpu_widget_state.current_display_time = STALE_MIN_MILLISECONDS;
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Mem => {
                if let Some(mem_widget_state) = self
                    .states
                    .mem_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = mem_widget_state
                        .current_display_time
                        .saturating_sub(self.app_config_fields.time_interval);

                    if new_time >= STALE_MIN_MILLISECONDS {
                        mem_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if mem_widget_state.current_display_time != STALE_MIN_MILLISECONDS {
                        mem_widget_state.current_display_time = STALE_MIN_MILLISECONDS;
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Net => {
                if let Some(net_widget_state) = self
                    .states
                    .net_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = net_widget_state
                        .current_display_time
                        .saturating_sub(self.app_config_fields.time_interval);

                    if new_time >= STALE_MIN_MILLISECONDS {
                        net_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if net_widget_state.current_display_time != STALE_MIN_MILLISECONDS {
                        net_widget_state.current_display_time = STALE_MIN_MILLISECONDS;
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            #[cfg(feature = "gpu")]
            BottomWidgetType::Gpu => {
                if let Some(gpu_widget_state) = self
                    .states
                    .gpu_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = gpu_widget_state
                        .current_display_time
                        .saturating_sub(self.app_config_fields.time_interval);

                    if new_time >= STALE_MIN_MILLISECONDS {
                        gpu_widget_state.current_display_time = new_time;
                        if self.app_config_fields.autohide_time {
                            gpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if gpu_widget_state.current_display_time != STALE_MIN_MILLISECONDS {
                        gpu_widget_state.current_display_time = STALE_MIN_MILLISECONDS;
                        if self.app_config_fields.autohide_time {
                            gpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn reset_cpu_zoom(&mut self) {
        if let Some(cpu_widget_state) = self
            .states
            .cpu_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            cpu_widget_state.current_display_time = self.app_config_fields.default_time_value;
            if self.app_config_fields.autohide_time {
                cpu_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    fn reset_mem_zoom(&mut self) {
        if let Some(mem_widget_state) = self
            .states
            .mem_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            mem_widget_state.current_display_time = self.app_config_fields.default_time_value;
            if self.app_config_fields.autohide_time {
                mem_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    fn reset_net_zoom(&mut self) {
        if let Some(net_widget_state) = self
            .states
            .net_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            net_widget_state.current_display_time = self.app_config_fields.default_time_value;
            if self.app_config_fields.autohide_time {
                net_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    #[cfg(feature = "gpu")]
    fn reset_gpu_zoom(&mut self) {
        if let Some(gpu_widget_state) = self
            .states
            .gpu_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            gpu_widget_state.current_display_time = self.app_config_fields.default_time_value;
            if self.app_config_fields.autohide_time {
                gpu_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    fn reset_zoom(&mut self) {
        match self.current_widget.widget_type {
            BottomWidgetType::Cpu => self.reset_cpu_zoom(),
            BottomWidgetType::Mem => self.reset_mem_zoom(),
            BottomWidgetType::Net => self.reset_net_zoom(),
            #[cfg(feature = "gpu")]
            BottomWidgetType::Gpu => self.reset_gpu_zoom(),
            _ => {}
        }
    }

    /// Moves the mouse to the widget that was clicked on, then propagates the
    /// click down to be handled by the widget specifically.
    pub fn on_left_mouse_up(&mut self, x: u16, y: u16) {
        // Pretty dead simple - iterate through the widget map and go to the widget
        // where the click is within.

        // TODO: [REFACTOR] might want to refactor this, it's really ugly.
        // TODO: [REFACTOR] Might wanna refactor ALL state things in general, currently
        // everything is grouped up as an app state.  We should separate stuff
        // like event state and gui state and etc.

        // TODO: [MOUSE] double click functionality...?  We would do this above all
        // other actions and SC if needed.

        // Short circuit if we're in basic table... we might have to handle the basic
        // table arrow case here...

        if let Some(bt) = &mut self.states.basic_table_widget_state {
            if let (
                Some((left_tlc_x, left_tlc_y)),
                Some((left_brc_x, left_brc_y)),
                Some((right_tlc_x, right_tlc_y)),
                Some((right_brc_x, right_brc_y)),
            ) = (bt.left_tlc, bt.left_brc, bt.right_tlc, bt.right_brc)
            {
                if (x >= left_tlc_x && y >= left_tlc_y) && (x < left_brc_x && y < left_brc_y) {
                    // Case for the left "button" in the simple arrow.
                    if let Some(new_widget) =
                        self.widget_map.get(&(bt.currently_displayed_widget_id))
                    {
                        // We have to move to the current table widget first...
                        self.current_widget = new_widget.clone();

                        if let BottomWidgetType::Proc = &new_widget.widget_type {
                            if let Some(proc_widget_state) = self
                                .states
                                .proc_state
                                .get_widget_state(new_widget.widget_id)
                            {
                                if proc_widget_state.is_sort_open {
                                    self.move_widget_selection(&WidgetDirection::Left);
                                }
                            }
                        }
                        self.move_widget_selection(&WidgetDirection::Left);
                        return;
                    }
                } else if (x >= right_tlc_x && y >= right_tlc_y)
                    && (x < right_brc_x && y < right_brc_y)
                {
                    // Case for the right "button" in the simple arrow.
                    if let Some(new_widget) =
                        self.widget_map.get(&(bt.currently_displayed_widget_id))
                    {
                        // We have to move to the current table widget first...
                        self.current_widget = new_widget.clone();

                        if let BottomWidgetType::ProcSort = &new_widget.widget_type {
                            if let Some(proc_widget_state) = self
                                .states
                                .proc_state
                                .get_widget_state(new_widget.widget_id - 2)
                            {
                                if proc_widget_state.is_sort_open {
                                    self.move_widget_selection(&WidgetDirection::Right);
                                }
                            }
                        }
                    }
                    self.move_widget_selection(&WidgetDirection::Right);
                    // Bit extra logic to ensure you always land on a proc widget, not the sort
                    if let BottomWidgetType::ProcSort = &self.current_widget.widget_type {
                        self.move_widget_selection(&WidgetDirection::Right);
                    }
                    return;
                }
            }
        }

        // Second short circuit --- are we in the dd dialog state?  If so, only check
        // yes/no/signals and bail after.
        if self.process_kill_dialog.is_open() && self.process_kill_dialog.on_click(x, y) {
            return;
        }

        let mut failed_to_get = true;
        for (new_widget_id, widget) in &self.widget_map {
            if let (Some((tlc_x, tlc_y)), Some((brc_x, brc_y))) =
                (widget.top_left_corner, widget.bottom_right_corner)
            {
                if (x >= tlc_x && y >= tlc_y) && (x < brc_x && y < brc_y) {
                    if let Some(new_widget) = self.widget_map.get(new_widget_id) {
                        self.current_widget = new_widget.clone();
                        match &self.current_widget.widget_type {
                            BottomWidgetType::Temp
                            | BottomWidgetType::Proc
                            | BottomWidgetType::ProcSort
                            | BottomWidgetType::Disk
                            | BottomWidgetType::Battery => {
                                if let Some(basic_table_widget_state) =
                                    &mut self.states.basic_table_widget_state
                                {
                                    basic_table_widget_state.currently_displayed_widget_id =
                                        self.current_widget.widget_id;
                                    basic_table_widget_state.currently_displayed_widget_type =
                                        self.current_widget.widget_type.clone();
                                }
                            }
                            _ => {}
                        }

                        failed_to_get = false;
                        break;
                    }
                }
            }
        }

        if failed_to_get {
            return;
        }

        // Now handle click propagation down to widget.
        if let (Some((_tlc_x, tlc_y)), Some((_brc_x, brc_y))) = (
            &self.current_widget.top_left_corner,
            &self.current_widget.bottom_right_corner,
        ) {
            let border_offset = u16::from(self.is_drawing_border());

            // This check ensures the click isn't actually just clicking on the bottom
            // border.
            if y < (brc_y - border_offset) {
                match &self.current_widget.widget_type {
                    BottomWidgetType::Proc
                    | BottomWidgetType::ProcSort
                    | BottomWidgetType::CpuLegend
                    | BottomWidgetType::Temp
                    | BottomWidgetType::Disk => {
                        // Get our index...
                        let clicked_entry = y - *tlc_y;
                        let header_offset = self.header_offset(&self.current_widget);
                        let offset = border_offset + header_offset;
                        if clicked_entry >= offset {
                            let offset_clicked_entry = clicked_entry - offset;
                            match &self.current_widget.widget_type {
                                BottomWidgetType::Proc => {
                                    if let Some(proc_widget_state) = self
                                        .states
                                        .proc_state
                                        .get_widget_state(self.current_widget.widget_id)
                                    {
                                        if let Some(visual_index) =
                                            proc_widget_state.table.ratatui_selected()
                                        {
                                            let is_tree_mode = matches!(
                                                proc_widget_state.mode,
                                                ProcWidgetMode::Tree { .. }
                                            );
                                            let change =
                                                offset_clicked_entry as i64 - visual_index as i64;

                                            self.change_process_position(change);

                                            // If in tree mode, also check to see if this click is
                                            // on
                                            // the same entry as the already selected one - if it
                                            // is,
                                            // then we minimize.
                                            if is_tree_mode && change == 0 {
                                                self.toggle_collapsing_process_branch();
                                            }
                                        }
                                    }
                                }
                                BottomWidgetType::ProcSort => {
                                    // TODO: [Feature] This could sort if you double click!
                                    if let Some(proc_widget_state) = self
                                        .states
                                        .proc_state
                                        .get_widget_state(self.current_widget.widget_id - 2)
                                    {
                                        if let Some(visual_index) =
                                            proc_widget_state.sort_table.ratatui_selected()
                                        {
                                            self.change_process_sort_position(
                                                offset_clicked_entry as i64 - visual_index as i64,
                                            );
                                        }
                                    }
                                }
                                BottomWidgetType::CpuLegend => {
                                    if let Some(cpu_widget_state) = self
                                        .states
                                        .cpu_state
                                        .get_widget_state(self.current_widget.widget_id - 1)
                                    {
                                        if let Some(visual_index) =
                                            cpu_widget_state.table.ratatui_selected()
                                        {
                                            self.change_cpu_legend_position(
                                                offset_clicked_entry as i64 - visual_index as i64,
                                            );
                                        }
                                    }
                                }
                                BottomWidgetType::Temp => {
                                    if let Some(temp_widget_state) = self
                                        .states
                                        .temp_state
                                        .get_widget_state(self.current_widget.widget_id)
                                    {
                                        if let Some(visual_index) =
                                            temp_widget_state.table.ratatui_selected()
                                        {
                                            self.change_temp_position(
                                                offset_clicked_entry as i64 - visual_index as i64,
                                            );
                                        }
                                    }
                                }
                                BottomWidgetType::Disk => {
                                    if let Some(disk_widget_state) = self
                                        .states
                                        .disk_state
                                        .get_widget_state(self.current_widget.widget_id)
                                    {
                                        if let Some(visual_index) =
                                            disk_widget_state.table.ratatui_selected()
                                        {
                                            self.change_disk_position(
                                                offset_clicked_entry as i64 - visual_index as i64,
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            // We might have clicked on a header!  Check if we only exceeded the
                            // table + border offset, and it's implied
                            // we exceeded the gap offset.
                            if clicked_entry == border_offset {
                                match &self.current_widget.widget_type {
                                    BottomWidgetType::Proc => {
                                        if let Some(state) = self
                                            .states
                                            .proc_state
                                            .get_mut_widget_state(self.current_widget.widget_id)
                                        {
                                            if state.table.try_select_location(x, y).is_some() {
                                                state.force_data_update();
                                            }
                                        }
                                    }
                                    BottomWidgetType::Temp => {
                                        if let Some(temp) = self
                                            .states
                                            .temp_state
                                            .get_mut_widget_state(self.current_widget.widget_id)
                                        {
                                            if temp.table.try_select_location(x, y).is_some() {
                                                temp.force_data_update();
                                            }
                                        }
                                    }
                                    BottomWidgetType::Disk => {
                                        if let Some(disk) = self
                                            .states
                                            .disk_state
                                            .get_mut_widget_state(self.current_widget.widget_id)
                                        {
                                            if disk.table.try_select_location(x, y).is_some() {
                                                disk.force_data_update();
                                            }
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                    BottomWidgetType::Battery => {
                        #[cfg(feature = "battery")]
                        if let Some(battery_widget_state) = self
                            .states
                            .battery_state
                            .get_mut_widget_state(self.current_widget.widget_id)
                        {
                            if let Some(tab_spacing) = &battery_widget_state.tab_click_locs {
                                for (itx, ((tlc_x, tlc_y), (brc_x, brc_y))) in
                                    tab_spacing.iter().enumerate()
                                {
                                    if (x >= *tlc_x && y >= *tlc_y) && (x <= *brc_x && y <= *brc_y)
                                    {
                                        let num_batteries =
                                            self.data_store.get_data().battery_harvest.len();
                                        if itx >= num_batteries {
                                            // range check to keep within current data
                                            battery_widget_state.currently_selected_battery_index =
                                                num_batteries - 1;
                                        } else {
                                            battery_widget_state.currently_selected_battery_index =
                                                itx;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn is_drawing_border(&self) -> bool {
        self.is_expanded || !self.app_config_fields.use_basic_mode
    }

    fn header_offset(&self, widget: &BottomWidget) -> u16 {
        if let (Some((_tlc_x, tlc_y)), Some((_brc_x, brc_y))) =
            (widget.top_left_corner, widget.bottom_right_corner)
        {
            let height_diff = brc_y - tlc_y;
            if height_diff >= constants::TABLE_GAP_HEIGHT_LIMIT {
                1 + self.app_config_fields.table_gap
            } else {
                let min_height_for_header = if self.is_drawing_border() { 3 } else { 1 };
                u16::from(height_diff > min_height_for_header)
            }
        } else {
            1 + self.app_config_fields.table_gap
        }
    }

    /// A quick and dirty way to handle paste events.
    pub fn handle_paste(&mut self, paste: String) {
        // Partially copy-pasted from the single-char variant; should probably clean up
        // this process in the future. In particular, encapsulate this entire
        // logic and add some tests to make it less potentially error-prone.
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .states
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            let num_runes = UnicodeSegmentation::graphemes(paste.as_str(), true).count();

            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                let left_bound = proc_widget_state.cursor_char_index();

                let curr_query = &mut proc_widget_state
                    .proc_search
                    .search_state
                    .current_search_query;
                let (left, right) = curr_query.split_at(left_bound);
                *curr_query = concat_string!(left, paste, right);

                proc_widget_state.proc_search.search_state.grapheme_cursor =
                    GraphemeCursor::new(left_bound, curr_query.len(), true);

                for _ in 0..num_runes {
                    proc_widget_state.search_walk_forward();
                }

                proc_widget_state.update_query();
                proc_widget_state.proc_search.search_state.cursor_direction =
                    CursorDirection::Right;
            }
        }
    }
}
