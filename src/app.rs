use std::{collections::HashMap, time::Instant};

use unicode_segmentation::GraphemeCursor;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use typed_builder::*;

use data_farmer::*;
use data_harvester::{processes, temperature};
use layout_manager::*;
pub use states::*;

use crate::{
    canvas, constants,
    utils::error::{BottomError, Result},
};

pub mod data_farmer;
pub mod data_harvester;
pub mod layout_manager;
mod process_killer;
pub mod query;
pub mod states;

const MAX_SEARCH_LENGTH: usize = 200;

/// AppConfigFields is meant to cover basic fields that would normally be set
/// by config files or launch options.
pub struct AppConfigFields {
    pub update_rate_in_milliseconds: u64,
    pub temperature_type: temperature::TemperatureType,
    pub use_dot: bool,
    pub left_legend: bool,
    pub show_average_cpu: bool,
    pub use_current_cpu_total: bool,
    pub use_basic_mode: bool,
    pub default_time_value: u64,
    pub time_interval: u64,
    pub hide_time: bool,
    pub autohide_time: bool,
    pub use_old_network_legend: bool,
    pub table_gap: u16,
}

#[derive(TypedBuilder)]
pub struct App {
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

    #[builder(default, setter(skip))]
    pub data_collection: DataCollection,

    #[builder(default, setter(skip))]
    pub delete_dialog_state: AppDeleteDialogState,

    #[builder(default, setter(skip))]
    pub help_dialog_state: AppHelpDialogState,

    #[builder(default = false, setter(skip))]
    pub is_expanded: bool,

    #[builder(default = false, setter(skip))]
    pub is_force_redraw: bool,

    #[builder(default = false, setter(skip))]
    pub basic_mode_use_percent: bool,

    pub cpu_state: CpuState,
    pub mem_state: MemState,
    pub net_state: NetState,
    pub proc_state: ProcState,
    pub temp_state: TempState,
    pub disk_state: DiskState,
    pub battery_state: BatteryState,

    pub basic_table_widget_state: Option<BasicTableWidgetState>,

    pub app_config_fields: AppConfigFields,
    pub widget_map: HashMap<u64, BottomWidget>,
    pub current_widget: BottomWidget,
    pub used_widgets: UsedWidgets,
}

impl App {
    pub fn reset(&mut self) {
        // Reset multi
        self.reset_multi_tap_keys();

        // Reset dialog state
        self.help_dialog_state.is_showing_help = false;
        self.delete_dialog_state.is_showing_dd = false;

        // Close all searches and reset it
        self.proc_state
            .widget_states
            .values_mut()
            .for_each(|state| {
                state.process_search_state.search_state.reset();
            });
        self.proc_state.force_update_all = true;

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

    fn close_dd(&mut self) {
        self.delete_dialog_state.is_showing_dd = false;
        self.delete_dialog_state.is_on_yes = false;
        self.to_delete_process_list = None;
        self.dd_err = None;
    }

    pub fn on_esc(&mut self) {
        self.reset_multi_tap_keys();
        if self.is_in_dialog() {
            if self.help_dialog_state.is_showing_help {
                self.help_dialog_state.is_showing_help = false;
                self.help_dialog_state.scroll_state.current_scroll_index = 0;
            } else {
                self.close_dd();
            }

            self.is_force_redraw = true;
        } else {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(current_proc_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        if current_proc_state.is_search_enabled() || current_proc_state.is_sort_open
                        {
                            current_proc_state
                                .process_search_state
                                .search_state
                                .is_enabled = false;
                            current_proc_state.is_sort_open = false;
                            self.is_force_redraw = true;
                            return;
                        }
                    }
                }
                BottomWidgetType::ProcSearch => {
                    if let Some(current_proc_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if current_proc_state.is_search_enabled() {
                            current_proc_state
                                .process_search_state
                                .search_state
                                .is_enabled = false;
                            self.move_widget_selection(&WidgetDirection::Up);
                            return;
                        }
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(current_proc_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        if current_proc_state.is_sort_open {
                            current_proc_state.columns.current_scroll_position =
                                current_proc_state.columns.backup_prev_scroll_position;
                            current_proc_state.is_sort_open = false;
                            self.move_widget_selection(&WidgetDirection::Right);
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
        self.help_dialog_state.is_showing_help || self.delete_dialog_state.is_showing_dd
    }

    pub fn on_tab(&mut self) {
        // Disallow usage whilst in a dialog and only in processes

        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Cpu => {
                    if let Some(cpu_widget_state) = self
                        .cpu_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        cpu_widget_state.is_multi_graph_mode =
                            !cpu_widget_state.is_multi_graph_mode;
                    }
                }
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        // Toggles process widget grouping state
                        proc_widget_state.is_grouped = !(proc_widget_state.is_grouped);

                        proc_widget_state
                            .columns
                            .column_mapping
                            .get_mut(&processes::ProcessSorting::State)
                            .unwrap()
                            .enabled = !(proc_widget_state.is_grouped);

                        proc_widget_state
                            .columns
                            .toggle(&processes::ProcessSorting::Count);
                        proc_widget_state
                            .columns
                            .toggle(&processes::ProcessSorting::Pid);

                        self.proc_state.force_update = Some(self.current_widget.widget_id);
                    }
                }
                _ => {}
            }
        }
    }

    /// I don't like this, but removing it causes a bunch of breakage.
    /// Use ``proc_widget_state.is_grouped`` if possible!
    pub fn is_grouped(&self, widget_id: u64) -> bool {
        if let Some(proc_widget_state) = self.proc_state.widget_states.get(&widget_id) {
            proc_widget_state.is_grouped
        } else {
            false
        }
    }

    /// "On space" if we don't want to treat is as a character.
    pub fn on_space(&mut self) {}

    pub fn on_slash(&mut self) {
        if !self.is_in_dialog() {
            if let BottomWidgetType::Proc = self.current_widget.widget_type {
                // Toggle on
                if let Some(proc_widget_state) = self
                    .proc_state
                    .get_mut_widget_state(self.current_widget.widget_id)
                {
                    proc_widget_state
                        .process_search_state
                        .search_state
                        .is_enabled = true;
                    self.move_widget_selection(&WidgetDirection::Down);
                }
            }
        }
    }

    pub fn toggle_sort(&mut self) {
        match &self.current_widget.widget_type {
            widget_type @ BottomWidgetType::Proc | widget_type @ BottomWidgetType::ProcSort => {
                let widget_id = self.current_widget.widget_id
                    - match &widget_type {
                        BottomWidgetType::Proc => 0,
                        BottomWidgetType::ProcSort => 2,
                        _ => 0,
                    };

                if let Some(proc_widget_state) = self.proc_state.get_mut_widget_state(widget_id) {
                    // Open up sorting dialog for that specific proc widget.
                    // TODO: It might be a decent idea to allow sorting ALL?  I dunno.

                    proc_widget_state.is_sort_open = !proc_widget_state.is_sort_open;
                    if proc_widget_state.is_sort_open {
                        // If it just opened, move left
                        proc_widget_state
                            .columns
                            .set_to_sorted_index(&proc_widget_state.process_sorting_type);
                        self.move_widget_selection(&WidgetDirection::Left);
                    } else {
                        // Otherwise, move right
                        self.move_widget_selection(&WidgetDirection::Right);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn invert_sort(&mut self) {
        match &self.current_widget.widget_type {
            widget_type @ BottomWidgetType::Proc | widget_type @ BottomWidgetType::ProcSort => {
                let widget_id = self.current_widget.widget_id
                    - match &widget_type {
                        BottomWidgetType::Proc => 0,
                        BottomWidgetType::ProcSort => 2,
                        _ => 0,
                    };

                if let Some(proc_widget_state) = self.proc_state.get_mut_widget_state(widget_id) {
                    proc_widget_state.process_sorting_reverse =
                        !proc_widget_state.process_sorting_reverse;

                    self.proc_state.force_update = Some(widget_id);
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
                    .proc_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    proc_widget_state
                        .columns
                        .toggle(&processes::ProcessSorting::Mem);
                    if let Some(mem_percent_state) = proc_widget_state
                        .columns
                        .toggle(&processes::ProcessSorting::MemPercent)
                    {
                        if proc_widget_state.process_sorting_type
                            == processes::ProcessSorting::MemPercent
                            || proc_widget_state.process_sorting_type
                                == processes::ProcessSorting::Mem
                        {
                            if mem_percent_state {
                                proc_widget_state.process_sorting_type =
                                    processes::ProcessSorting::MemPercent;
                            } else {
                                proc_widget_state.process_sorting_type =
                                    processes::ProcessSorting::Mem;
                            }
                        }
                    }

                    self.proc_state.force_update = Some(self.current_widget.widget_id);
                }
            }
            _ => {}
        }
    }

    pub fn toggle_ignore_case(&mut self) {
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                proc_widget_state
                    .process_search_state
                    .search_toggle_ignore_case();
                proc_widget_state.update_query();
                self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
            }
        }
    }

    pub fn toggle_search_whole_word(&mut self) {
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                proc_widget_state
                    .process_search_state
                    .search_toggle_whole_word();
                proc_widget_state.update_query();
                self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
            }
        }
    }

    pub fn toggle_search_regex(&mut self) {
        let is_in_search_widget = self.is_in_search_widget();
        if let Some(proc_widget_state) = self
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            if is_in_search_widget && proc_widget_state.is_search_enabled() {
                proc_widget_state.process_search_state.search_toggle_regex();
                proc_widget_state.update_query();
                self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
            }
        }
    }

    /// One of two functions allowed to run while in a dialog...
    pub fn on_enter(&mut self) {
        if self.delete_dialog_state.is_showing_dd {
            if self.dd_err.is_some() {
                self.close_dd();
            } else if self.delete_dialog_state.is_on_yes {
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
        } else if let BottomWidgetType::ProcSort = self.current_widget.widget_type {
            if let Some(proc_widget_state) = self
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 2))
            {
                self.proc_state.force_update = Some(self.current_widget.widget_id - 2);
                proc_widget_state.update_sorting_with_columns();
                self.toggle_sort();
            }
        }
    }

    pub fn on_delete(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            let is_in_search_widget = self.is_in_search_widget();
            if let Some(proc_widget_state) = self
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                if is_in_search_widget {
                    if proc_widget_state
                        .process_search_state
                        .search_state
                        .is_enabled
                        && proc_widget_state.get_cursor_position()
                            < proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len()
                    {
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .current_search_query
                            .remove(proc_widget_state.get_cursor_position());

                        proc_widget_state
                            .process_search_state
                            .search_state
                            .grapheme_cursor = GraphemeCursor::new(
                            proc_widget_state.get_cursor_position(),
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len(),
                            true,
                        );

                        proc_widget_state.update_query();
                        self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
                    }
                } else {
                    self.start_dd()
                }
            }
        }
    }

    pub fn on_backspace(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            let is_in_search_widget = self.is_in_search_widget();
            if let Some(proc_widget_state) = self
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                if is_in_search_widget
                    && proc_widget_state
                        .process_search_state
                        .search_state
                        .is_enabled
                    && proc_widget_state.get_cursor_position() > 0
                {
                    proc_widget_state.search_walk_back(proc_widget_state.get_cursor_position());

                    let removed_char = proc_widget_state
                        .process_search_state
                        .search_state
                        .current_search_query
                        .remove(proc_widget_state.get_cursor_position());

                    proc_widget_state
                        .process_search_state
                        .search_state
                        .grapheme_cursor = GraphemeCursor::new(
                        proc_widget_state.get_cursor_position(),
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .current_search_query
                            .len(),
                        true,
                    );

                    proc_widget_state
                        .process_search_state
                        .search_state
                        .char_cursor_position -= UnicodeWidthChar::width(removed_char).unwrap_or(0);
                    proc_widget_state
                        .process_search_state
                        .search_state
                        .cursor_direction = CursorDirection::Left;

                    proc_widget_state.update_query();
                    self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
                }
            }
        }
    }

    pub fn get_process_filter(&self, widget_id: u64) -> &Option<query::Query> {
        if let Some(process_widget_state) = self.proc_state.widget_states.get(&widget_id) {
            &process_widget_state.process_search_state.search_state.query
        } else {
            &None
        }
    }

    pub fn on_up_key(&mut self) {
        if !self.is_in_dialog() {
            self.decrement_position_count();
        } else if self.help_dialog_state.is_showing_help {
            self.help_scroll_up();
        }
        self.reset_multi_tap_keys();
    }

    pub fn on_down_key(&mut self) {
        if !self.is_in_dialog() {
            self.increment_position_count();
        } else if self.help_dialog_state.is_showing_help {
            self.help_scroll_down();
        }
        self.reset_multi_tap_keys();
    }

    pub fn on_left_key(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    // if let Some(proc_widget_state) = self
                    //     .proc_state
                    //     .get_mut_widget_state(self.current_widget.widget_id)
                    // {
                    //     proc_widget_state.current_column_index =
                    //         proc_widget_state.current_column_index.saturating_sub(1);

                    //     debug!(
                    //         "Current column index <: {}",
                    //         proc_widget_state.current_column_index
                    //     );
                    // }
                }
                BottomWidgetType::ProcSearch => {
                    let is_in_search_widget = self.is_in_search_widget();
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if is_in_search_widget {
                            let prev_cursor = proc_widget_state.get_cursor_position();
                            proc_widget_state
                                .search_walk_back(proc_widget_state.get_cursor_position());
                            if proc_widget_state.get_cursor_position() < prev_cursor {
                                let str_slice = &proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .current_search_query
                                    [proc_widget_state.get_cursor_position()..prev_cursor];
                                proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .char_cursor_position -= UnicodeWidthStr::width(str_slice);
                                proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .cursor_direction = CursorDirection::Left;
                            }
                        }
                    }
                }
                BottomWidgetType::Battery => {
                    if !self.canvas_data.battery_data.is_empty() {
                        if let Some(battery_widget_state) = self
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
        } else if self.delete_dialog_state.is_showing_dd && !self.delete_dialog_state.is_on_yes {
            self.delete_dialog_state.is_on_yes = true;
        }
    }

    pub fn on_right_key(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    // if let Some(proc_widget_state) = self
                    //     .proc_state
                    //     .get_mut_widget_state(self.current_widget.widget_id)
                    // {
                    //     if proc_widget_state.current_column_index
                    //         < proc_widget_state.columns.get_enabled_columns()
                    //     {
                    //         proc_widget_state.current_column_index += 1;
                    //     }
                    //     debug!(
                    //         "Current column index >: {}",
                    //         proc_widget_state.current_column_index
                    //     );
                    // }
                }
                BottomWidgetType::ProcSearch => {
                    let is_in_search_widget = self.is_in_search_widget();
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if is_in_search_widget {
                            let prev_cursor = proc_widget_state.get_cursor_position();
                            proc_widget_state
                                .search_walk_forward(proc_widget_state.get_cursor_position());
                            if proc_widget_state.get_cursor_position() > prev_cursor {
                                let str_slice = &proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .current_search_query
                                    [prev_cursor..proc_widget_state.get_cursor_position()];
                                proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .char_cursor_position += UnicodeWidthStr::width(str_slice);
                                proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .cursor_direction = CursorDirection::Right;
                            }
                        }
                    }
                }
                BottomWidgetType::Battery => {
                    if !self.canvas_data.battery_data.is_empty() {
                        let battery_count = self.canvas_data.battery_data.len();
                        if let Some(battery_widget_state) = self
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
                _ => {}
            }
        } else if self.delete_dialog_state.is_showing_dd && self.delete_dialog_state.is_on_yes {
            self.delete_dialog_state.is_on_yes = false;
        }
    }

    pub fn skip_cursor_beginning(&mut self) {
        if !self.is_in_dialog() {
            if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget {
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .grapheme_cursor = GraphemeCursor::new(
                            0,
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len(),
                            true,
                        );
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .char_cursor_position = 0;
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .cursor_direction = CursorDirection::Left;
                    }
                }
            }
        }
    }

    pub fn skip_cursor_end(&mut self) {
        if !self.is_in_dialog() {
            if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget {
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .grapheme_cursor = GraphemeCursor::new(
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len(),
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len(),
                            true,
                        );
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .char_cursor_position = UnicodeWidthStr::width(
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .as_str(),
                        );
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .cursor_direction = CursorDirection::Right;
                    }
                }
            }
        }
    }

    pub fn clear_search(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            if let Some(proc_widget_state) = self
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                proc_widget_state.clear_search();
                self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
            }
        }
    }

    pub fn start_dd(&mut self) {
        self.reset_multi_tap_keys();

        if let Some(proc_widget_state) = self
            .proc_state
            .widget_states
            .get(&self.current_widget.widget_id)
        {
            if let Some(corresponding_filtered_process_list) = self
                .canvas_data
                .finalized_process_data_map
                .get(&self.current_widget.widget_id)
            {
                if proc_widget_state.scroll_state.current_scroll_position
                    < corresponding_filtered_process_list.len()
                {
                    let current_process: (String, Vec<u32>);
                    if self.is_grouped(self.current_widget.widget_id) {
                        if let Some(process) = &corresponding_filtered_process_list
                            .get(proc_widget_state.scroll_state.current_scroll_position)
                        {
                            current_process = (process.name.to_string(), process.group_pids.clone())
                        } else {
                            return;
                        }
                    } else {
                        let process = corresponding_filtered_process_list
                            [proc_widget_state.scroll_state.current_scroll_position]
                            .clone();
                        current_process = (process.name.clone(), vec![process.pid])
                    };

                    self.to_delete_process_list = Some(current_process);
                    self.delete_dialog_state.is_showing_dd = true;
                }
            }
        }
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

            if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
                let is_in_search_widget = self.is_in_search_widget();
                if let Some(proc_widget_state) = self
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 1))
                {
                    if is_in_search_widget
                        && proc_widget_state.is_search_enabled()
                        && UnicodeWidthStr::width(
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .as_str(),
                        ) <= MAX_SEARCH_LENGTH
                    {
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .current_search_query
                            .insert(proc_widget_state.get_cursor_position(), caught_char);

                        proc_widget_state
                            .process_search_state
                            .search_state
                            .grapheme_cursor = GraphemeCursor::new(
                            proc_widget_state.get_cursor_position(),
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len(),
                            true,
                        );
                        proc_widget_state
                            .search_walk_forward(proc_widget_state.get_cursor_position());

                        proc_widget_state
                            .process_search_state
                            .search_state
                            .char_cursor_position +=
                            UnicodeWidthChar::width(caught_char).unwrap_or(0);

                        proc_widget_state.update_query();
                        self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .cursor_direction = CursorDirection::Right;

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
                        if (potential_index as usize) < self.help_dialog_state.index_shortcuts.len()
                        {
                            self.help_scroll_to_or_max(
                                self.help_dialog_state.index_shortcuts[potential_index as usize],
                            );
                        }
                    }
                }
                'j' | 'k' | 'g' | 'G' => self.handle_char(caught_char),
                _ => {}
            }
        } else if self.delete_dialog_state.is_showing_dd {
            match caught_char {
                'h' | 'j' => self.on_left_key(),
                'k' | 'l' => self.on_right_key(),
                _ => {}
            }
        }
    }

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
            'k' => self.on_up_key(),
            'j' => self.on_down_key(),
            'f' => {
                self.is_frozen = !self.is_frozen;
                if self.is_frozen {
                    self.data_collection.set_frozen_time();
                }
            }
            'c' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        match proc_widget_state.process_sorting_type {
                            processes::ProcessSorting::CpuPercent => {
                                proc_widget_state.process_sorting_reverse =
                                    !proc_widget_state.process_sorting_reverse
                            }
                            _ => {
                                proc_widget_state.process_sorting_type =
                                    processes::ProcessSorting::CpuPercent;
                                proc_widget_state.process_sorting_reverse = true;
                            }
                        }
                        self.proc_state.force_update = Some(self.current_widget.widget_id);

                        self.skip_to_first();
                    }
                }
            }
            'm' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        match proc_widget_state.process_sorting_type {
                            processes::ProcessSorting::MemPercent => {
                                proc_widget_state.process_sorting_reverse =
                                    !proc_widget_state.process_sorting_reverse
                            }
                            _ => {
                                proc_widget_state.process_sorting_type =
                                    processes::ProcessSorting::MemPercent;
                                proc_widget_state.process_sorting_reverse = true;
                            }
                        }
                        self.proc_state.force_update = Some(self.current_widget.widget_id);
                        self.skip_to_first();
                    }
                }
            }
            'p' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        // Skip if grouped
                        if !proc_widget_state.is_grouped {
                            match proc_widget_state.process_sorting_type {
                                processes::ProcessSorting::Pid => {
                                    proc_widget_state.process_sorting_reverse =
                                        !proc_widget_state.process_sorting_reverse
                                }
                                _ => {
                                    proc_widget_state.process_sorting_type =
                                        processes::ProcessSorting::Pid;
                                    proc_widget_state.process_sorting_reverse = false;
                                }
                            }
                            self.proc_state.force_update = Some(self.current_widget.widget_id);
                            self.skip_to_first();
                        }
                    }
                }
            }
            'P' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.is_using_command = !proc_widget_state.is_using_command;
                        proc_widget_state
                            .toggle_command_and_name(proc_widget_state.is_using_command);

                        match &proc_widget_state.process_sorting_type {
                            processes::ProcessSorting::Command
                            | processes::ProcessSorting::ProcessName => {
                                if proc_widget_state.is_using_command {
                                    proc_widget_state.process_sorting_type =
                                        processes::ProcessSorting::Command;
                                } else {
                                    proc_widget_state.process_sorting_type =
                                        processes::ProcessSorting::ProcessName;
                                }
                            }
                            _ => {}
                        }

                        self.proc_state.force_update = Some(self.current_widget.widget_id);
                    }
                }
            }
            'n' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        match proc_widget_state.process_sorting_type {
                            processes::ProcessSorting::ProcessName
                            | processes::ProcessSorting::Command => {
                                proc_widget_state.process_sorting_reverse =
                                    !proc_widget_state.process_sorting_reverse
                            }
                            _ => {
                                proc_widget_state.process_sorting_type =
                                    if proc_widget_state.is_using_command {
                                        processes::ProcessSorting::Command
                                    } else {
                                        processes::ProcessSorting::ProcessName
                                    };
                                proc_widget_state.process_sorting_reverse = false;
                            }
                        }
                        self.proc_state.force_update = Some(self.current_widget.widget_id);
                        self.skip_to_first();
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
            ' ' => self.on_space(),
            '+' => self.zoom_in(),
            '-' => self.zoom_out(),
            '=' => self.reset_zoom(),
            'e' => self.toggle_expand_widget(),
            's' => self.toggle_sort(),
            'I' => self.invert_sort(),
            '%' => self.toggle_percentages(),
            _ => {}
        }

        if let Some(second_char) = self.second_char {
            if self.awaiting_second_char && caught_char != second_char {
                self.awaiting_second_char = false;
            }
        }
    }

    pub fn kill_highlighted_process(&mut self) -> Result<()> {
        if let BottomWidgetType::Proc = self.current_widget.widget_type {
            if let Some(current_selected_processes) = &self.to_delete_process_list {
                for pid in &current_selected_processes.1 {
                    process_killer::kill_process_given_pid(*pid)?;
                }
            }
            self.to_delete_process_list = None;
            Ok(())
        } else {
            Err(BottomError::GenericError(
                "Cannot kill processes if the current widget is not the Process widget!"
                    .to_string(),
            ))
        }
    }

    pub fn get_to_delete_processes(&self) -> Option<(String, Vec<u32>)> {
        self.to_delete_process_list.clone()
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
        if !self.is_in_dialog() && !self.app_config_fields.use_basic_mode {
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
        /*
            We follow these following steps:
            1. Send a movement signal in `direction`.
            2. Check if this new widget we've landed on is hidden.  If not, halt.
            3. If it hidden, loop and either send:
               - A signal equal to the current direction, if it is opposite of the reflection.
               - Reflection direction.
        */

        if !self.is_in_dialog() && !self.is_expanded {
            if let Some(new_widget_id) = &(match direction {
                WidgetDirection::Left => self.current_widget.left_neighbour,
                WidgetDirection::Right => self.current_widget.right_neighbour,
                WidgetDirection::Up => self.current_widget.up_neighbour,
                WidgetDirection::Down => self.current_widget.down_neighbour,
            }) {
                if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                    match &new_widget.widget_type {
                        BottomWidgetType::Temp
                        | BottomWidgetType::Proc
                        | BottomWidgetType::ProcSearch
                        | BottomWidgetType::ProcSort
                        | BottomWidgetType::Disk
                        | BottomWidgetType::Battery
                            if self.basic_table_widget_state.is_some()
                                && (*direction == WidgetDirection::Left
                                    || *direction == WidgetDirection::Right) =>
                        {
                            // Gotta do this for the sort widget
                            if let BottomWidgetType::ProcSort = new_widget.widget_type {
                                if let Some(proc_widget_state) =
                                    self.proc_state.widget_states.get(&(new_widget_id - 2))
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
                                &mut self.basic_table_widget_state
                            {
                                basic_table_widget_state.currently_displayed_widget_id =
                                    self.current_widget.widget_id;
                                basic_table_widget_state.currently_displayed_widget_type =
                                    self.current_widget.widget_type.clone();
                            }
                        }
                        BottomWidgetType::BasicTables => {
                            match &direction {
                                WidgetDirection::Up => {
                                    // Note this case would fail if it moved up into a hidden
                                    // widget, but it's for basic so whatever, it's all hard-coded
                                    // right now anyways.
                                    if let Some(next_new_widget_id) = new_widget.up_neighbour {
                                        if let Some(next_new_widget) =
                                            self.widget_map.get(&next_new_widget_id)
                                        {
                                            self.current_widget = next_new_widget.clone();
                                        }
                                    }
                                }
                                WidgetDirection::Down => {
                                    // This means we're in basic mode.  As such, then
                                    // we want to move DOWN to the currently shown widget
                                    if let Some(basic_table_widget_state) =
                                        &self.basic_table_widget_state
                                    {
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
                                    let next_neighbour_id = match &direction {
                                        WidgetDirection::Left => new_widget.left_neighbour,
                                        WidgetDirection::Right => new_widget.right_neighbour,
                                        WidgetDirection::Up => new_widget.up_neighbour,
                                        WidgetDirection::Down => new_widget.down_neighbour,
                                    }
                                    .unwrap_or(*new_widget_id);
                                    match &new_widget.widget_type {
                                        BottomWidgetType::CpuLegend => {
                                            if let Some(cpu_widget_state) = self
                                                .cpu_state
                                                .widget_states
                                                .get(&(new_widget_id - *offset))
                                            {
                                                if cpu_widget_state.is_legend_hidden {
                                                    if let Some(next_neighbour_widget) =
                                                        self.widget_map.get(&next_neighbour_id)
                                                    {
                                                        self.current_widget =
                                                            next_neighbour_widget.clone();
                                                    }
                                                } else {
                                                    self.current_widget = new_widget.clone();
                                                }
                                            }
                                        }
                                        BottomWidgetType::ProcSearch
                                        | BottomWidgetType::ProcSort => {
                                            if let Some(proc_widget_state) = self
                                                .proc_state
                                                .widget_states
                                                .get(&(new_widget_id - *offset))
                                            {
                                                match &new_widget.widget_type {
                                                    BottomWidgetType::ProcSearch => {
                                                        if !proc_widget_state.is_search_enabled() {
                                                            if let Some(next_neighbour_widget) =
                                                                self.widget_map
                                                                    .get(&next_neighbour_id)
                                                            {
                                                                self.current_widget =
                                                                    next_neighbour_widget.clone();
                                                            }
                                                        } else {
                                                            self.current_widget =
                                                                new_widget.clone();
                                                        }
                                                    }
                                                    BottomWidgetType::ProcSort => {
                                                        if !proc_widget_state.is_sort_open {
                                                            if let Some(next_neighbour_widget) =
                                                                self.widget_map
                                                                    .get(&next_neighbour_id)
                                                            {
                                                                self.current_widget =
                                                                    next_neighbour_widget.clone();
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
                        self.move_widget_selection(ref_dir);
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
                    proc_type @ BottomWidgetType::Proc | proc_type @ BottomWidgetType::ProcSort => {
                        let widget_id = self.current_widget.widget_id
                            - match proc_type {
                                BottomWidgetType::ProcSort => 2,
                                _ => 0,
                            };
                        if let Some(current_widget) = self.widget_map.get(&widget_id) {
                            if let Some(new_widget_id) = current_widget.down_neighbour {
                                if let Some(new_widget) = self.widget_map.get(&new_widget_id) {
                                    if let Some(proc_widget_state) =
                                        self.proc_state.get_widget_state(widget_id)
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

        self.reset_multi_tap_keys();
    }

    fn handle_left_expanded_movement(&mut self) {
        if let BottomWidgetType::Proc = self.current_widget.widget_type {
            if let Some(new_widget_id) = self.current_widget.left_neighbour {
                if let Some(proc_widget_state) = self
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
        } else if self.app_config_fields.left_legend {
            if let BottomWidgetType::Cpu = self.current_widget.widget_type {
                if let Some(current_widget) = self.widget_map.get(&self.current_widget.widget_id) {
                    if let Some(cpu_widget_state) = self
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
        } else if self.app_config_fields.left_legend {
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
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.scroll_state.current_scroll_position = 0;
                        proc_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        proc_widget_state.columns.current_scroll_position = 0;
                        proc_widget_state.columns.scroll_direction = ScrollDirection::Up;
                    }
                }
                BottomWidgetType::Temp => {
                    if let Some(temp_widget_state) = self
                        .temp_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        temp_widget_state.scroll_state.current_scroll_position = 0;
                        temp_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
                    }
                }
                BottomWidgetType::Disk => {
                    if let Some(disk_widget_state) = self
                        .disk_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        disk_widget_state.scroll_state.current_scroll_position = 0;
                        disk_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
                    }
                }
                BottomWidgetType::CpuLegend => {
                    if let Some(cpu_widget_state) = self
                        .cpu_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        cpu_widget_state.scroll_state.current_scroll_position = 0;
                        cpu_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
                    }
                }

                _ => {}
            }
            self.reset_multi_tap_keys();
        } else {
            self.help_dialog_state.scroll_state.current_scroll_index = 0;
        }
    }

    pub fn skip_to_last(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        if let Some(finalized_process_data) = self
                            .canvas_data
                            .finalized_process_data_map
                            .get(&self.current_widget.widget_id)
                        {
                            if !self.canvas_data.finalized_process_data_map.is_empty() {
                                proc_widget_state.scroll_state.current_scroll_position =
                                    finalized_process_data.len() - 1;
                                proc_widget_state.scroll_state.scroll_direction =
                                    ScrollDirection::Down;
                            }
                        }
                    }
                }
                BottomWidgetType::ProcSort => {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 2)
                    {
                        proc_widget_state.columns.current_scroll_position =
                            proc_widget_state.columns.get_enabled_columns_len() - 1;
                        proc_widget_state.columns.scroll_direction = ScrollDirection::Down;
                    }
                }
                BottomWidgetType::Temp => {
                    if let Some(temp_widget_state) = self
                        .temp_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        if !self.canvas_data.temp_sensor_data.is_empty() {
                            temp_widget_state.scroll_state.current_scroll_position =
                                self.canvas_data.temp_sensor_data.len() - 1;
                            temp_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
                        }
                    }
                }
                BottomWidgetType::Disk => {
                    if let Some(disk_widget_state) = self
                        .disk_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        if !self.canvas_data.disk_data.is_empty() {
                            disk_widget_state.scroll_state.current_scroll_position =
                                self.canvas_data.disk_data.len() - 1;
                            disk_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
                        }
                    }
                }
                BottomWidgetType::CpuLegend => {
                    if let Some(cpu_widget_state) = self
                        .cpu_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        let cap = self.canvas_data.cpu_data.len();
                        if cap > 0 {
                            cpu_widget_state.scroll_state.current_scroll_position = cap - 1;
                            cpu_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
                        }
                    }
                }
                _ => {}
            }
            self.reset_multi_tap_keys();
        } else {
            self.help_dialog_state.scroll_state.current_scroll_index = self
                .help_dialog_state
                .scroll_state
                .max_scroll_index
                .saturating_sub(1);
        }
    }

    pub fn decrement_position_count(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => self.change_process_position(-1),
                BottomWidgetType::ProcSort => self.change_process_sort_position(-1),
                BottomWidgetType::Temp => self.change_temp_position(-1),
                BottomWidgetType::Disk => self.change_disk_position(-1),
                BottomWidgetType::CpuLegend => self.change_cpu_table_position(-1),
                _ => {}
            }
        }
    }

    pub fn increment_position_count(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => self.change_process_position(1),
                BottomWidgetType::ProcSort => self.change_process_sort_position(1),
                BottomWidgetType::Temp => self.change_temp_position(1),
                BottomWidgetType::Disk => self.change_disk_position(1),
                BottomWidgetType::CpuLegend => self.change_cpu_table_position(1),
                _ => {}
            }
        }
    }

    fn change_process_sort_position(&mut self, num_to_change_by: i64) {
        if let Some(proc_widget_state) = self
            .proc_state
            .get_mut_widget_state(self.current_widget.widget_id - 2)
        {
            let current_posn = proc_widget_state.columns.current_scroll_position;
            let num_columns = proc_widget_state.columns.get_enabled_columns_len();

            if current_posn as i64 + num_to_change_by >= 0
                && current_posn as i64 + num_to_change_by < num_columns as i64
            {
                proc_widget_state.columns.current_scroll_position =
                    (current_posn as i64 + num_to_change_by) as usize;
            }

            if num_to_change_by < 0 {
                proc_widget_state.columns.scroll_direction = ScrollDirection::Up;
            } else {
                proc_widget_state.columns.scroll_direction = ScrollDirection::Down;
            }
        }
    }

    fn change_cpu_table_position(&mut self, num_to_change_by: i64) {
        if let Some(cpu_widget_state) = self
            .cpu_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id - 1))
        {
            let current_posn = cpu_widget_state.scroll_state.current_scroll_position;

            let cap = self.canvas_data.cpu_data.len();
            if current_posn as i64 + num_to_change_by >= 0
                && current_posn as i64 + num_to_change_by < cap as i64
            {
                cpu_widget_state.scroll_state.current_scroll_position =
                    (current_posn as i64 + num_to_change_by) as usize;
            }

            if num_to_change_by < 0 {
                cpu_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
            } else {
                cpu_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
            }
        }
    }

    fn change_process_position(&mut self, num_to_change_by: i64) {
        if let Some(proc_widget_state) = self
            .proc_state
            .get_mut_widget_state(self.current_widget.widget_id)
        {
            let current_posn = proc_widget_state.scroll_state.current_scroll_position;

            if let Some(finalized_process_data) = self
                .canvas_data
                .finalized_process_data_map
                .get(&self.current_widget.widget_id)
            {
                if current_posn as i64 + num_to_change_by >= 0
                    && current_posn as i64 + num_to_change_by < finalized_process_data.len() as i64
                {
                    proc_widget_state.scroll_state.current_scroll_position =
                        (current_posn as i64 + num_to_change_by) as usize;
                }
            }

            if num_to_change_by < 0 {
                proc_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
            } else {
                proc_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
            }
        }
    }

    fn change_temp_position(&mut self, num_to_change_by: i64) {
        if let Some(temp_widget_state) = self
            .temp_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            let current_posn = temp_widget_state.scroll_state.current_scroll_position;

            if current_posn as i64 + num_to_change_by >= 0
                && current_posn as i64 + num_to_change_by
                    < self.canvas_data.temp_sensor_data.len() as i64
            {
                temp_widget_state.scroll_state.current_scroll_position =
                    (current_posn as i64 + num_to_change_by) as usize;
            }

            if num_to_change_by < 0 {
                temp_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
            } else {
                temp_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
            }
        }
    }

    fn change_disk_position(&mut self, num_to_change_by: i64) {
        if let Some(disk_widget_state) = self
            .disk_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            let current_posn = disk_widget_state.scroll_state.current_scroll_position;

            if current_posn as i64 + num_to_change_by >= 0
                && current_posn as i64 + num_to_change_by < self.canvas_data.disk_data.len() as i64
            {
                disk_widget_state.scroll_state.current_scroll_position =
                    (current_posn as i64 + num_to_change_by) as usize;
            }

            if num_to_change_by < 0 {
                disk_widget_state.scroll_state.scroll_direction = ScrollDirection::Up;
            } else {
                disk_widget_state.scroll_state.scroll_direction = ScrollDirection::Down;
            }
        }
    }

    fn help_scroll_up(&mut self) {
        if self.help_dialog_state.scroll_state.current_scroll_index > 0 {
            self.help_dialog_state.scroll_state.current_scroll_index -= 1;
        }
    }

    fn help_scroll_down(&mut self) {
        if self.help_dialog_state.scroll_state.current_scroll_index + 1
            < self.help_dialog_state.scroll_state.max_scroll_index
        {
            self.help_dialog_state.scroll_state.current_scroll_index += 1;
        }
    }

    fn help_scroll_to_or_max(&mut self, new_position: u16) {
        if new_position < self.help_dialog_state.scroll_state.max_scroll_index {
            self.help_dialog_state.scroll_state.current_scroll_index = new_position;
        } else {
            self.help_dialog_state.scroll_state.current_scroll_index =
                self.help_dialog_state.scroll_state.max_scroll_index - 1;
        }
    }

    pub fn handle_scroll_up(&mut self) {
        if self.help_dialog_state.is_showing_help {
            self.help_scroll_up();
        } else if self.current_widget.widget_type.is_widget_graph() {
            self.zoom_in();
        } else if self.current_widget.widget_type.is_widget_table() {
            self.decrement_position_count();
        }
    }

    pub fn handle_scroll_down(&mut self) {
        if self.help_dialog_state.is_showing_help {
            self.help_scroll_down();
        } else if self.current_widget.widget_type.is_widget_graph() {
            self.zoom_out();
        } else if self.current_widget.widget_type.is_widget_table() {
            self.increment_position_count();
        }
    }

    fn zoom_out(&mut self) {
        match self.current_widget.widget_type {
            BottomWidgetType::Cpu => {
                if let Some(cpu_widget_state) = self
                    .cpu_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = cpu_widget_state.current_display_time
                        + self.app_config_fields.time_interval;
                    if new_time <= constants::STALE_MAX_MILLISECONDS {
                        cpu_widget_state.current_display_time = new_time;
                        self.cpu_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if cpu_widget_state.current_display_time
                        != constants::STALE_MAX_MILLISECONDS
                    {
                        cpu_widget_state.current_display_time = constants::STALE_MAX_MILLISECONDS;
                        self.cpu_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Mem => {
                if let Some(mem_widget_state) = self
                    .mem_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = mem_widget_state.current_display_time
                        + self.app_config_fields.time_interval;
                    if new_time <= constants::STALE_MAX_MILLISECONDS {
                        mem_widget_state.current_display_time = new_time;
                        self.mem_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if mem_widget_state.current_display_time
                        != constants::STALE_MAX_MILLISECONDS
                    {
                        mem_widget_state.current_display_time = constants::STALE_MAX_MILLISECONDS;
                        self.mem_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Net => {
                if let Some(net_widget_state) = self
                    .net_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = net_widget_state.current_display_time
                        + self.app_config_fields.time_interval;
                    if new_time <= constants::STALE_MAX_MILLISECONDS {
                        net_widget_state.current_display_time = new_time;
                        self.net_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if net_widget_state.current_display_time
                        != constants::STALE_MAX_MILLISECONDS
                    {
                        net_widget_state.current_display_time = constants::STALE_MAX_MILLISECONDS;
                        self.net_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
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
                    .cpu_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = cpu_widget_state.current_display_time
                        - self.app_config_fields.time_interval;
                    if new_time >= constants::STALE_MIN_MILLISECONDS {
                        cpu_widget_state.current_display_time = new_time;
                        self.cpu_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if cpu_widget_state.current_display_time
                        != constants::STALE_MIN_MILLISECONDS
                    {
                        cpu_widget_state.current_display_time = constants::STALE_MIN_MILLISECONDS;
                        self.cpu_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            cpu_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Mem => {
                if let Some(mem_widget_state) = self
                    .mem_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = mem_widget_state.current_display_time
                        - self.app_config_fields.time_interval;
                    if new_time >= constants::STALE_MIN_MILLISECONDS {
                        mem_widget_state.current_display_time = new_time;
                        self.mem_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if mem_widget_state.current_display_time
                        != constants::STALE_MIN_MILLISECONDS
                    {
                        mem_widget_state.current_display_time = constants::STALE_MIN_MILLISECONDS;
                        self.mem_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            mem_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            BottomWidgetType::Net => {
                if let Some(net_widget_state) = self
                    .net_state
                    .widget_states
                    .get_mut(&self.current_widget.widget_id)
                {
                    let new_time = net_widget_state.current_display_time
                        - self.app_config_fields.time_interval;
                    if new_time >= constants::STALE_MIN_MILLISECONDS {
                        net_widget_state.current_display_time = new_time;
                        self.net_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    } else if net_widget_state.current_display_time
                        != constants::STALE_MIN_MILLISECONDS
                    {
                        net_widget_state.current_display_time = constants::STALE_MIN_MILLISECONDS;
                        self.net_state.force_update = Some(self.current_widget.widget_id);
                        if self.app_config_fields.autohide_time {
                            net_widget_state.autohide_timer = Some(Instant::now());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn reset_cpu_zoom(&mut self) {
        if let Some(cpu_widget_state) = self
            .cpu_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            cpu_widget_state.current_display_time = self.app_config_fields.default_time_value;
            self.cpu_state.force_update = Some(self.current_widget.widget_id);
            if self.app_config_fields.autohide_time {
                cpu_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    fn reset_mem_zoom(&mut self) {
        if let Some(mem_widget_state) = self
            .mem_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            mem_widget_state.current_display_time = self.app_config_fields.default_time_value;
            self.mem_state.force_update = Some(self.current_widget.widget_id);
            if self.app_config_fields.autohide_time {
                mem_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    fn reset_net_zoom(&mut self) {
        if let Some(net_widget_state) = self
            .net_state
            .widget_states
            .get_mut(&self.current_widget.widget_id)
        {
            net_widget_state.current_display_time = self.app_config_fields.default_time_value;
            self.net_state.force_update = Some(self.current_widget.widget_id);
            if self.app_config_fields.autohide_time {
                net_widget_state.autohide_timer = Some(Instant::now());
            }
        }
    }

    fn reset_zoom(&mut self) {
        match self.current_widget.widget_type {
            BottomWidgetType::Cpu => self.reset_cpu_zoom(),
            BottomWidgetType::Mem => self.reset_mem_zoom(),
            BottomWidgetType::Net => self.reset_net_zoom(),
            _ => {}
        }
    }
}
