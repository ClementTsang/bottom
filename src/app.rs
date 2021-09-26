pub mod data_farmer;
pub mod data_harvester;
pub mod event;
pub mod filter;
pub mod layout_manager;
mod process_killer;
pub mod query;
pub mod widgets;

use std::{
    cmp::{max, min},
    collections::HashMap,
    time::Instant,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use fxhash::FxHashMap;
use indextree::{Arena, NodeId};
use unicode_segmentation::GraphemeCursor;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub use data_farmer::*;
use data_harvester::{processes, temperature};
pub use filter::*;
use layout_manager::*;
pub use widgets::*;

use crate::{
    canvas,
    constants::{self, MAX_SIGNAL},
    units::data_units::DataUnit,
    utils::error::{BottomError, Result},
    BottomEvent, Pid,
};

use self::event::{ComponentEventResult, EventResult, ReturnSignal};

const MAX_SEARCH_LENGTH: usize = 200;

#[derive(Debug, Clone)]
pub enum AxisScaling {
    Log,
    Linear,
}

#[derive(Clone, Default, Debug)]
pub struct UsedWidgets {
    pub use_cpu: bool,
    pub use_mem: bool,
    pub use_net: bool,
    pub use_proc: bool,
    pub use_disk: bool,
    pub use_temp: bool,
    pub use_battery: bool,
}

impl UsedWidgets {
    pub fn add(&mut self, widget_type: &BottomWidgetType) {
        match widget_type {
            BottomWidgetType::Cpu | BottomWidgetType::BasicCpu => {
                self.use_cpu = true;
            }
            BottomWidgetType::Mem | BottomWidgetType::BasicMem => {
                self.use_mem = true;
            }
            BottomWidgetType::Net | BottomWidgetType::BasicNet => {
                self.use_net = true;
            }
            BottomWidgetType::Proc => {
                self.use_proc = true;
            }
            BottomWidgetType::Temp => {
                self.use_temp = true;
            }
            BottomWidgetType::Disk => {
                self.use_disk = true;
            }
            BottomWidgetType::Battery => {
                self.use_battery = true;
            }
            _ => {}
        }
    }
}

/// AppConfigFields is meant to cover basic fields that would normally be set
/// by config files or launch options.
#[derive(Debug)]
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
    pub table_gap: u16, // TODO: Just make this a bool...
    pub disable_click: bool,
    pub no_write: bool,
    pub show_table_scroll_position: bool,
    pub is_advanced_kill: bool,
    pub network_unit_type: DataUnit,
    pub network_scale_type: AxisScaling,
    pub network_use_binary_prefix: bool,
}

/// The [`FrozenState`] indicates whether the application state should be frozen; if it is, save a snapshot of
/// the data collected at that instant.
pub enum FrozenState {
    NotFrozen,
    Frozen(DataCollection),
}

impl Default for FrozenState {
    fn default() -> Self {
        Self::NotFrozen
    }
}

pub struct AppState {
    pub dd_err: Option<String>,

    to_delete_process_list: Option<(String, Vec<Pid>)>,

    pub canvas_data: canvas::DisplayableData,

    pub data_collection: DataCollection,

    pub is_expanded: bool,

    pub used_widgets: UsedWidgets,
    pub filters: DataFilters,
    pub app_config_fields: AppConfigFields,

    // --- Eventually delete/rewrite ---
    pub delete_dialog_state: AppDeleteDialogState,

    // --- TO DELETE ---
    pub cpu_state: CpuState,
    pub mem_state: MemState,
    pub net_state: NetState,
    pub proc_state: ProcState,
    pub temp_state: TempState,
    pub disk_state: DiskState,
    pub battery_state: BatteryState,
    pub basic_table_widget_state: Option<BasicTableWidgetState>,
    pub widget_map: HashMap<u64, BottomWidget>,
    pub current_widget: BottomWidget,

    last_key_press: Instant,

    awaiting_second_char: bool,

    second_char: Option<char>,

    pub basic_mode_use_percent: bool,

    pub is_force_redraw: bool,

    pub is_determining_widget_boundary: bool,

    // --- NEW STUFF ---
    pub selected_widget: NodeId,
    pub widget_lookup_map: FxHashMap<NodeId, TmpBottomWidget>,
    pub layout_tree: Arena<LayoutNode>,
    pub layout_tree_root: NodeId,
    frozen_state: FrozenState,

    pub help_dialog: DialogState<HelpDialog>,
}

impl AppState {
    /// Creates a new [`AppState`].
    pub fn new(
        app_config_fields: AppConfigFields, filters: DataFilters,
        layout_tree_output: LayoutCreationOutput,
    ) -> Self {
        let LayoutCreationOutput {
            layout_tree,
            root: layout_tree_root,
            widget_lookup_map,
            selected: selected_widget,
            used_widgets,
        } = layout_tree_output;

        Self {
            app_config_fields,
            filters,
            used_widgets,
            selected_widget,
            widget_lookup_map,
            layout_tree,
            layout_tree_root,

            // Use defaults.
            dd_err: Default::default(),
            to_delete_process_list: Default::default(),
            canvas_data: Default::default(),
            data_collection: Default::default(),
            is_expanded: Default::default(),
            delete_dialog_state: Default::default(),
            cpu_state: Default::default(),
            mem_state: Default::default(),
            net_state: Default::default(),
            proc_state: Default::default(),
            temp_state: Default::default(),
            disk_state: Default::default(),
            battery_state: Default::default(),
            basic_table_widget_state: Default::default(),
            widget_map: Default::default(),
            current_widget: Default::default(),
            last_key_press: Instant::now(),
            awaiting_second_char: Default::default(),
            second_char: Default::default(),
            basic_mode_use_percent: Default::default(),
            is_force_redraw: Default::default(),
            is_determining_widget_boundary: Default::default(),
            frozen_state: Default::default(),
            help_dialog: Default::default(),
        }
    }

    pub fn is_frozen(&self) -> bool {
        matches!(self.frozen_state, FrozenState::Frozen(_))
    }

    pub fn toggle_freeze(&mut self) {
        if matches!(self.frozen_state, FrozenState::Frozen(_)) {
            self.frozen_state = FrozenState::NotFrozen;
        } else {
            self.frozen_state = FrozenState::Frozen(self.data_collection.clone());
        }
    }

    pub fn reset(&mut self) {
        // Call reset on all widgets.
        self.widget_lookup_map
            .values_mut()
            .for_each(|widget| widget.reset());

        // Unfreeze.
        self.frozen_state = FrozenState::NotFrozen;

        // Reset data
        self.data_collection.reset();
    }

    pub fn should_get_widget_bounds(&self) -> bool {
        self.is_force_redraw || self.is_determining_widget_boundary
    }

    fn close_dd(&mut self) {
        self.delete_dialog_state.is_showing_dd = false;
        self.delete_dialog_state.selected_signal = KillSignal::default();
        self.delete_dialog_state.scroll_pos = 0;
        self.to_delete_process_list = None;
        self.dd_err = None;
    }

    /// Handles a global event involving a char.
    fn handle_global_char(&mut self, c: char) -> EventResult {
        if c.is_ascii_control() {
            EventResult::NoRedraw
        } else {
            // Check for case-sensitive bindings first.
            match c {
                'H' | 'A' => self.move_to_widget(MovementDirection::Left),
                'L' | 'D' => self.move_to_widget(MovementDirection::Right),
                'K' | 'W' => self.move_to_widget(MovementDirection::Up),
                'J' | 'S' => self.move_to_widget(MovementDirection::Down),
                _ => {
                    let c = c.to_ascii_lowercase();
                    match c {
                        'q' => EventResult::Quit,
                        'e' if !self.help_dialog.is_showing() => {
                            if self.app_config_fields.use_basic_mode {
                                EventResult::NoRedraw
                            } else {
                                self.is_expanded = !self.is_expanded;
                                EventResult::Redraw
                            }
                        }
                        '?' if !self.help_dialog.is_showing() => {
                            self.help_dialog.show();
                            EventResult::Redraw
                        }
                        'f' if !self.help_dialog.is_showing() => {
                            self.toggle_freeze();
                            if !self.is_frozen() {
                                let data_collection = &self.data_collection;
                                self.widget_lookup_map
                                    .iter_mut()
                                    .for_each(|(_id, widget)| widget.update_data(data_collection));
                            }
                            EventResult::Redraw
                        }
                        _ => EventResult::NoRedraw,
                    }
                }
            }
        }
    }

    /// Moves to a widget.
    fn move_to_widget(&mut self, direction: MovementDirection) -> EventResult {
        match if self.is_expanded {
            move_expanded_widget_selection(
                &mut self.widget_lookup_map,
                self.selected_widget,
                direction,
            )
        } else {
            let layout_tree = &mut self.layout_tree;

            move_widget_selection(
                layout_tree,
                &mut self.widget_lookup_map,
                self.selected_widget,
                direction,
            )
        } {
            MoveWidgetResult::ForceRedraw(new_widget_id) => {
                self.selected_widget = new_widget_id;
                EventResult::Redraw
            }
            MoveWidgetResult::NodeId(new_widget_id) => {
                let previous_selected = self.selected_widget;
                self.selected_widget = new_widget_id;

                if previous_selected != self.selected_widget {
                    EventResult::Redraw
                } else {
                    EventResult::NoRedraw
                }
            }
        }
    }

    /// Quick and dirty handler to convert [`ComponentEventResult`]s to [`EventResult`]s, and handle [`ReturnSignal`]s.
    fn convert_widget_event_result(&mut self, w: ComponentEventResult) -> EventResult {
        match w {
            ComponentEventResult::Unhandled => EventResult::NoRedraw,
            ComponentEventResult::Redraw => EventResult::Redraw,
            ComponentEventResult::NoRedraw => EventResult::NoRedraw,
            ComponentEventResult::Signal(signal) => match signal {
                ReturnSignal::KillProcess => {
                    todo!()
                }
                ReturnSignal::Update => {
                    if let Some(widget) = self.widget_lookup_map.get_mut(&self.selected_widget) {
                        match &self.frozen_state {
                            FrozenState::NotFrozen => {
                                widget.update_data(&self.data_collection);
                            }
                            FrozenState::Frozen(frozen_data) => {
                                widget.update_data(frozen_data);
                            }
                        }
                    }
                    EventResult::Redraw
                }
            },
        }
    }

    /// Handles a [`KeyEvent`], and returns an [`EventResult`].
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        let result = if let DialogState::Shown(help_dialog) = &mut self.help_dialog {
            help_dialog.handle_key_event(event)
        } else if let Some(widget) = self.widget_lookup_map.get_mut(&self.selected_widget) {
            widget.handle_key_event(event)
        } else {
            ComponentEventResult::Unhandled
        };

        match result {
            ComponentEventResult::Unhandled => self.handle_global_key_event(event),
            _ => self.convert_widget_event_result(result),
        }
    }

    /// Handles a global [`KeyEvent`], and returns an [`EventResult`].
    fn handle_global_key_event(&mut self, event: KeyEvent) -> EventResult {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Esc => {
                    if self.is_expanded {
                        self.is_expanded = false;
                        EventResult::Redraw
                    } else if self.help_dialog.is_showing() {
                        self.help_dialog.hide();
                        EventResult::Redraw
                    } else if self.delete_dialog_state.is_showing_dd {
                        self.close_dd();
                        EventResult::Redraw
                    } else {
                        EventResult::NoRedraw
                    }
                }
                _ => {
                    if let KeyCode::Char(c) = event.code {
                        self.handle_global_char(c)
                    } else {
                        EventResult::NoRedraw
                    }
                }
            }
        } else if let KeyModifiers::CONTROL = event.modifiers {
            match event.code {
                KeyCode::Char('c') | KeyCode::Char('C') => EventResult::Quit,
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.reset();
                    let data_collection = &self.data_collection;
                    self.widget_lookup_map
                        .iter_mut()
                        .for_each(|(_id, widget)| widget.update_data(data_collection));
                    EventResult::Redraw
                }
                KeyCode::Left => self.move_to_widget(MovementDirection::Left),
                KeyCode::Right => self.move_to_widget(MovementDirection::Right),
                KeyCode::Up => self.move_to_widget(MovementDirection::Up),
                KeyCode::Down => self.move_to_widget(MovementDirection::Down),
                _ => EventResult::NoRedraw,
            }
        } else if let KeyModifiers::SHIFT = event.modifiers {
            match event.code {
                KeyCode::Left => self.move_to_widget(MovementDirection::Left),
                KeyCode::Right => self.move_to_widget(MovementDirection::Right),
                KeyCode::Up => self.move_to_widget(MovementDirection::Up),
                KeyCode::Down => self.move_to_widget(MovementDirection::Down),
                KeyCode::Char(c) => self.handle_global_char(c),
                _ => EventResult::NoRedraw,
            }
        } else {
            EventResult::NoRedraw
        }
    }

    /// Handles a [`MouseEvent`].
    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        if let DialogState::Shown(help_dialog) = &mut self.help_dialog {
            let result = help_dialog.handle_mouse_event(event);
            self.convert_widget_event_result(result)
        } else if self.is_expanded {
            if let Some(widget) = self.widget_lookup_map.get_mut(&self.selected_widget) {
                let result = widget.handle_mouse_event(event);
                self.convert_widget_event_result(result)
            } else {
                EventResult::NoRedraw
            }
        } else {
            let mut returned_result = EventResult::NoRedraw;
            for (id, widget) in self.widget_lookup_map.iter_mut() {
                if widget.does_border_intersect_mouse(&event) {
                    let result = widget.handle_mouse_event(event);
                    match widget.selectable_type() {
                        SelectableType::Selectable => {
                            let was_id_already_selected = self.selected_widget == *id;
                            self.selected_widget = *id;

                            if was_id_already_selected {
                                returned_result = self.convert_widget_event_result(result);
                                break;
                            } else {
                                // If the weren't equal, *force* a redraw, and correct the layout tree.
                                correct_layout_last_selections(
                                    &mut self.layout_tree,
                                    self.selected_widget,
                                );
                                let _ = self.convert_widget_event_result(result);
                                returned_result = EventResult::Redraw;
                                break;
                            }
                        }
                        SelectableType::Unselectable => {
                            let result = widget.handle_mouse_event(event);
                            return self.convert_widget_event_result(result);
                        }
                    }
                }
            }

            returned_result
        }
    }

    /// Handles a [`BottomEvent`] and updates the [`AppState`] if needed. Returns an [`EventResult`] indicating
    /// whether the app now requires a redraw.
    pub fn handle_event(&mut self, event: BottomEvent) -> EventResult {
        match event {
            BottomEvent::KeyInput(event) => self.handle_key_event(event),
            BottomEvent::MouseInput(event) => {
                // Not great, but basically a blind lookup through the table for anything that clips the click location.
                self.handle_mouse_event(event)
            }
            BottomEvent::Update(new_data) => {
                self.data_collection.eat_data(new_data);

                // TODO: Optimization for dialogs; don't redraw here.

                if !self.is_frozen() {
                    let data_collection = &self.data_collection;
                    self.widget_lookup_map
                        .iter_mut()
                        .for_each(|(_id, widget)| widget.update_data(data_collection));

                    EventResult::Redraw
                } else {
                    EventResult::NoRedraw
                }
            }
            BottomEvent::Resize {
                width: _,
                height: _,
            } => EventResult::Redraw,
            BottomEvent::Clean => {
                self.data_collection
                    .clean_data(constants::STALE_MAX_MILLISECONDS);
                EventResult::NoRedraw
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
        self.delete_dialog_state.is_showing_dd
    }

    fn ignore_normal_keybinds(&self) -> bool {
        self.is_in_dialog()
    }

    pub fn on_tab(&mut self) {
        // Allow usage whilst only in processes

        if !self.ignore_normal_keybinds() {
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
                        // Do NOT allow when in tree mode!
                        if !proc_widget_state.is_tree_mode {
                            // Toggles process widget grouping state
                            proc_widget_state.is_grouped = !(proc_widget_state.is_grouped);

                            // Forcefully switch off column if we were on it...
                            if (proc_widget_state.is_grouped
                                && (proc_widget_state.process_sorting_type
                                    == processes::ProcessSorting::Pid
                                    || proc_widget_state.process_sorting_type
                                        == processes::ProcessSorting::User
                                    || proc_widget_state.process_sorting_type
                                        == processes::ProcessSorting::State))
                                || (!proc_widget_state.is_grouped
                                    && proc_widget_state.process_sorting_type
                                        == processes::ProcessSorting::Count)
                            {
                                proc_widget_state.process_sorting_type =
                                    processes::ProcessSorting::CpuPercent; // Go back to default, negate PID for group
                                proc_widget_state.is_process_sort_descending = true;
                            }

                            proc_widget_state.columns.set_to_sorted_index_from_type(
                                &proc_widget_state.process_sorting_type,
                            );

                            proc_widget_state.columns.try_set(
                                &processes::ProcessSorting::State,
                                !(proc_widget_state.is_grouped),
                            );

                            #[cfg(target_family = "unix")]
                            proc_widget_state.columns.try_set(
                                &processes::ProcessSorting::User,
                                !(proc_widget_state.is_grouped),
                            );

                            proc_widget_state
                                .columns
                                .toggle(&processes::ProcessSorting::Count);
                            proc_widget_state
                                .columns
                                .toggle(&processes::ProcessSorting::Pid);

                            proc_widget_state.requires_redraw = true;
                            self.proc_state.force_update = Some(self.current_widget.widget_id);
                        }
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

    pub fn on_slash(&mut self) {
        if !self.ignore_normal_keybinds() {
            match &self.current_widget.widget_type {
                BottomWidgetType::Proc | BottomWidgetType::ProcSort => {
                    // Toggle on
                    if let Some(proc_widget_state) = self.proc_state.get_mut_widget_state(
                        self.current_widget.widget_id
                            - match &self.current_widget.widget_type {
                                BottomWidgetType::ProcSort => 2,
                                _ => 0,
                            },
                    ) {
                        proc_widget_state
                            .process_search_state
                            .search_state
                            .is_enabled = true;
                        self.move_widget_selection(&WidgetDirection::Down);
                        self.is_force_redraw = true;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn toggle_sort(&mut self) {
        match &self.current_widget.widget_type {
            BottomWidgetType::Proc | BottomWidgetType::ProcSort => {
                let widget_id = self.current_widget.widget_id
                    - match &self.current_widget.widget_type {
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
                            .set_to_sorted_index_from_type(&proc_widget_state.process_sorting_type);
                        self.move_widget_selection(&WidgetDirection::Left);
                    } else {
                        // Otherwise, move right if currently on the sort widget
                        if let BottomWidgetType::ProcSort = self.current_widget.widget_type {
                            self.move_widget_selection(&WidgetDirection::Right);
                        }
                    }
                }

                self.is_force_redraw = true;
            }
            _ => {}
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

                if let Some(proc_widget_state) = self.proc_state.get_mut_widget_state(widget_id) {
                    proc_widget_state.is_process_sort_descending =
                        !proc_widget_state.is_process_sort_descending;

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

                    proc_widget_state.requires_redraw = true;
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

    pub fn toggle_tree_mode(&mut self) {
        if let Some(proc_widget_state) = self
            .proc_state
            .widget_states
            .get_mut(&(self.current_widget.widget_id))
        {
            proc_widget_state.is_tree_mode = !proc_widget_state.is_tree_mode;

            // FIXME: For consistency, either disable tree mode if grouped, or allow grouped mode if in tree mode.
            if proc_widget_state.is_tree_mode {
                // Disable grouping if so!
                proc_widget_state.is_grouped = false;

                proc_widget_state
                    .columns
                    .try_enable(&processes::ProcessSorting::State);

                #[cfg(target_family = "unix")]
                proc_widget_state
                    .columns
                    .try_enable(&processes::ProcessSorting::User);

                proc_widget_state
                    .columns
                    .try_disable(&processes::ProcessSorting::Count);

                proc_widget_state
                    .columns
                    .try_enable(&processes::ProcessSorting::Pid);

                // We enabled... set PID sort type to ascending.
                proc_widget_state.process_sorting_type = processes::ProcessSorting::Pid;
                proc_widget_state.is_process_sort_descending = false;
            }

            self.proc_state.force_update = Some(self.current_widget.widget_id);
            proc_widget_state.requires_redraw = true;
        }
    }

    /// One of two functions allowed to run while in a dialog...
    pub fn on_enter(&mut self) {
        if self.delete_dialog_state.is_showing_dd {
            if self.dd_err.is_some() {
                self.close_dd();
            } else if self.delete_dialog_state.selected_signal != KillSignal::Cancel {
                // If within dd...
                if self.dd_err.is_none() {
                    // Also ensure that we didn't just fail a dd...
                    let dd_result = self.kill_highlighted_process();
                    self.delete_dialog_state.scroll_pos = 0;
                    self.delete_dialog_state.selected_signal = KillSignal::default();

                    // Check if there was an issue... if so, inform the user.
                    if let Err(dd_err) = dd_result {
                        self.dd_err = Some(dd_err.to_string());
                    } else {
                        self.delete_dialog_state.is_showing_dd = false;
                    }
                }
            } else {
                self.delete_dialog_state.scroll_pos = 0;
                self.delete_dialog_state.selected_signal = KillSignal::default();
                self.delete_dialog_state.is_showing_dd = false;
            }
            self.is_force_redraw = true;
        } else if !self.is_in_dialog() {
            if let BottomWidgetType::ProcSort = self.current_widget.widget_type {
                if let Some(proc_widget_state) = self
                    .proc_state
                    .widget_states
                    .get_mut(&(self.current_widget.widget_id - 2))
                {
                    proc_widget_state.update_sorting_with_columns();
                    self.proc_state.force_update = Some(self.current_widget.widget_id - 2);
                    self.toggle_sort();
                }
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
                        && proc_widget_state.get_search_cursor_position()
                            < proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len()
                    {
                        let current_cursor = proc_widget_state.get_search_cursor_position();
                        proc_widget_state
                            .search_walk_forward(proc_widget_state.get_search_cursor_position());

                        let _removed_chars: String = proc_widget_state
                            .process_search_state
                            .search_state
                            .current_search_query
                            .drain(current_cursor..proc_widget_state.get_search_cursor_position())
                            .collect();

                        proc_widget_state
                            .process_search_state
                            .search_state
                            .grapheme_cursor = GraphemeCursor::new(
                            current_cursor,
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
                    self.start_killing_process()
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
                    && proc_widget_state.get_search_cursor_position() > 0
                {
                    let current_cursor = proc_widget_state.get_search_cursor_position();
                    proc_widget_state
                        .search_walk_back(proc_widget_state.get_search_cursor_position());

                    let removed_chars: String = proc_widget_state
                        .process_search_state
                        .search_state
                        .current_search_query
                        .drain(proc_widget_state.get_search_cursor_position()..current_cursor)
                        .collect();

                    proc_widget_state
                        .process_search_state
                        .search_state
                        .grapheme_cursor = GraphemeCursor::new(
                        proc_widget_state.get_search_cursor_position(),
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
                        .char_cursor_position -= UnicodeWidthStr::width(removed_chars.as_str());

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

    #[cfg(target_family = "unix")]
    pub fn on_number(&mut self, number_char: char) {
        if self.delete_dialog_state.is_showing_dd {
            if self
                .delete_dialog_state
                .last_number_press
                .map_or(100, |ins| ins.elapsed().as_millis())
                >= 400
            {
                self.delete_dialog_state.keyboard_signal_select = 0;
            }
            let mut kbd_signal = self.delete_dialog_state.keyboard_signal_select * 10;
            kbd_signal += number_char.to_digit(10).unwrap() as usize;
            if kbd_signal > 64 {
                kbd_signal %= 100;
            }
            #[cfg(target_os = "linux")]
            if kbd_signal > 64 || kbd_signal == 32 || kbd_signal == 33 {
                kbd_signal %= 10;
            }
            #[cfg(target_os = "macos")]
            if kbd_signal > 31 {
                kbd_signal %= 10;
            }
            self.delete_dialog_state.selected_signal = KillSignal::Kill(kbd_signal);
            if kbd_signal < 10 {
                self.delete_dialog_state.keyboard_signal_select = kbd_signal;
            } else {
                self.delete_dialog_state.keyboard_signal_select = 0;
            }
            self.delete_dialog_state.last_number_press = Some(Instant::now());
        }
    }

    pub fn on_up_key(&mut self) {
        if !self.is_in_dialog() {
            self.decrement_position_count();
        } else if self.help_dialog.is_showing() {
            self.help_scroll_up();
        } else if self.delete_dialog_state.is_showing_dd {
            #[cfg(target_os = "windows")]
            self.on_right_key();
            #[cfg(target_family = "unix")]
            {
                if self.app_config_fields.is_advanced_kill {
                    self.on_left_key();
                } else {
                    self.on_right_key();
                }
            }
            return;
        }
        self.reset_multi_tap_keys();
    }

    pub fn on_down_key(&mut self) {
        if !self.is_in_dialog() {
            self.increment_position_count();
        } else if self.help_dialog.is_showing() {
            self.help_scroll_down();
        } else if self.delete_dialog_state.is_showing_dd {
            #[cfg(target_os = "windows")]
            self.on_left_key();
            #[cfg(target_family = "unix")]
            {
                if self.app_config_fields.is_advanced_kill {
                    self.on_right_key();
                } else {
                    self.on_left_key();
                }
            }
            return;
        }
        self.reset_multi_tap_keys();
    }

    pub fn on_left_key(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::ProcSearch => {
                    let is_in_search_widget = self.is_in_search_widget();
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if is_in_search_widget {
                            let prev_cursor = proc_widget_state.get_search_cursor_position();
                            proc_widget_state
                                .search_walk_back(proc_widget_state.get_search_cursor_position());
                            if proc_widget_state.get_search_cursor_position() < prev_cursor {
                                let str_slice = &proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .current_search_query
                                    [proc_widget_state.get_search_cursor_position()..prev_cursor];
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
        } else if self.delete_dialog_state.is_showing_dd {
            #[cfg(target_family = "unix")]
            {
                if self.app_config_fields.is_advanced_kill {
                    match self.delete_dialog_state.selected_signal {
                        KillSignal::Kill(prev_signal) => {
                            self.delete_dialog_state.selected_signal = match prev_signal - 1 {
                                0 => KillSignal::Cancel,
                                // 32+33 are skipped
                                33 => KillSignal::Kill(31),
                                signal => KillSignal::Kill(signal),
                            };
                        }
                        KillSignal::Cancel => {}
                    };
                } else {
                    self.delete_dialog_state.selected_signal = KillSignal::default();
                }
            }
            #[cfg(target_os = "windows")]
            {
                self.delete_dialog_state.selected_signal = KillSignal::Kill(1);
            }
        }
    }

    pub fn on_right_key(&mut self) {
        if !self.is_in_dialog() {
            match self.current_widget.widget_type {
                BottomWidgetType::ProcSearch => {
                    let is_in_search_widget = self.is_in_search_widget();
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id - 1)
                    {
                        if is_in_search_widget {
                            let prev_cursor = proc_widget_state.get_search_cursor_position();
                            proc_widget_state.search_walk_forward(
                                proc_widget_state.get_search_cursor_position(),
                            );
                            if proc_widget_state.get_search_cursor_position() > prev_cursor {
                                let str_slice = &proc_widget_state
                                    .process_search_state
                                    .search_state
                                    .current_search_query
                                    [prev_cursor..proc_widget_state.get_search_cursor_position()];
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
        } else if self.delete_dialog_state.is_showing_dd {
            #[cfg(target_family = "unix")]
            {
                if self.app_config_fields.is_advanced_kill {
                    let new_signal = match self.delete_dialog_state.selected_signal {
                        KillSignal::Cancel => 1,
                        // 32+33 are skipped
                        #[cfg(target_os = "linux")]
                        KillSignal::Kill(31) => 34,
                        #[cfg(target_os = "macos")]
                        KillSignal::Kill(31) => 31,
                        KillSignal::Kill(64) => 64,
                        KillSignal::Kill(signal) => signal + 1,
                    };
                    self.delete_dialog_state.selected_signal = KillSignal::Kill(new_signal);
                } else {
                    self.delete_dialog_state.selected_signal = KillSignal::Cancel;
                }
            }
            #[cfg(target_os = "windows")]
            {
                self.delete_dialog_state.selected_signal = KillSignal::Cancel;
            }
        }
    }

    pub fn on_page_up(&mut self) {
        if self.delete_dialog_state.is_showing_dd {
            let mut new_signal = match self.delete_dialog_state.selected_signal {
                KillSignal::Cancel => 0,
                KillSignal::Kill(signal) => max(signal, 8) - 8,
            };
            if new_signal > 23 && new_signal < 33 {
                new_signal -= 2;
            }
            self.delete_dialog_state.selected_signal = match new_signal {
                0 => KillSignal::Cancel,
                sig => KillSignal::Kill(sig),
            };
        }
    }

    pub fn on_page_down(&mut self) {
        if self.delete_dialog_state.is_showing_dd {
            let mut new_signal = match self.delete_dialog_state.selected_signal {
                KillSignal::Cancel => 8,
                KillSignal::Kill(signal) => min(signal + 8, MAX_SIGNAL),
            };
            if new_signal > 31 && new_signal < 42 {
                new_signal += 2;
            }
            self.delete_dialog_state.selected_signal = KillSignal::Kill(new_signal);
        }
    }

    pub fn skip_cursor_beginning(&mut self) {
        if !self.ignore_normal_keybinds() {
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
        if !self.ignore_normal_keybinds() {
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

    pub fn clear_previous_word(&mut self) {
        if let BottomWidgetType::ProcSearch = self.current_widget.widget_type {
            if let Some(proc_widget_state) = self
                .proc_state
                .widget_states
                .get_mut(&(self.current_widget.widget_id - 1))
            {
                // Traverse backwards from the current cursor location until you hit non-whitespace characters,
                // then continue to traverse (and delete) backwards until you hit a whitespace character.  Halt.

                // So... first, let's get our current cursor position using graphemes...
                let end_index = proc_widget_state.get_char_cursor_position();

                // Then, let's crawl backwards until we hit our location, and store the "head"...
                let query = proc_widget_state.get_current_search_query();
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

                let removed_chars: String = proc_widget_state
                    .process_search_state
                    .search_state
                    .current_search_query
                    .drain(start_index..end_index)
                    .collect();

                proc_widget_state
                    .process_search_state
                    .search_state
                    .grapheme_cursor = GraphemeCursor::new(
                    start_index,
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
                    .char_cursor_position -= UnicodeWidthStr::width(removed_chars.as_str());

                proc_widget_state
                    .process_search_state
                    .search_state
                    .cursor_direction = CursorDirection::Left;

                proc_widget_state.update_query();
                self.proc_state.force_update = Some(self.current_widget.widget_id - 1);

                // Now, convert this range into a String-friendly range and remove it all at once!

                // Now make sure to also update our current cursor positions...

                self.proc_state.force_update = Some(self.current_widget.widget_id - 1);
            }
        }
    }

    pub fn start_killing_process(&mut self) {
        todo!()

        // if let Some(proc_widget_state) = self
        //     .proc_state
        //     .widget_states
        //     .get(&self.current_widget.widget_id)
        // {
        //     if let Some(corresponding_filtered_process_list) = self
        //         .canvas_data
        //         .finalized_process_data_map
        //         .get(&self.current_widget.widget_id)
        //     {
        //         if proc_widget_state.scroll_state.current_scroll_position
        //             < corresponding_filtered_process_list.len()
        //         {
        //             let current_process: (String, Vec<Pid>);
        //             if self.is_grouped(self.current_widget.widget_id) {
        //                 if let Some(process) = &corresponding_filtered_process_list
        //                     .get(proc_widget_state.scroll_state.current_scroll_position)
        //                 {
        //                     current_process = (process.name.to_string(), process.group_pids.clone())
        //                 } else {
        //                     return;
        //                 }
        //             } else {
        //                 let process = corresponding_filtered_process_list
        //                     [proc_widget_state.scroll_state.current_scroll_position]
        //                     .clone();
        //                 current_process = (process.name.clone(), vec![process.pid])
        //             };

        //             self.to_delete_process_list = Some(current_process);
        //             self.delete_dialog_state.is_showing_dd = true;
        //             self.is_determining_widget_boundary = true;
        //         }
        //     }
        // }
    }

    pub fn on_char_key(&mut self, caught_char: char) {
        // Skip control code chars
        if caught_char.is_control() {
            return;
        }

        // Forbid any char key presses when showing a dialog box...
        if !self.ignore_normal_keybinds() {
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
                            .insert(proc_widget_state.get_search_cursor_position(), caught_char);

                        proc_widget_state
                            .process_search_state
                            .search_state
                            .grapheme_cursor = GraphemeCursor::new(
                            proc_widget_state.get_search_cursor_position(),
                            proc_widget_state
                                .process_search_state
                                .search_state
                                .current_search_query
                                .len(),
                            true,
                        );
                        proc_widget_state
                            .search_walk_forward(proc_widget_state.get_search_cursor_position());

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
        } else if self.help_dialog.is_showing() {
            match caught_char {
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {}
                'j' | 'k' | 'g' | 'G' => self.handle_char(caught_char),
                _ => {}
            }
        } else if self.delete_dialog_state.is_showing_dd {
            match caught_char {
                'h' => self.on_left_key(),
                'j' => self.on_down_key(),
                'k' => self.on_up_key(),
                'l' => self.on_right_key(),
                #[cfg(target_family = "unix")]
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    self.on_number(caught_char)
                }
                'u' => self.on_page_up(),
                'd' => self.on_page_down(),
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

                            self.start_killing_process();
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
            'f' => {}
            'C' => {
                // self.open_config(),
            }
            'c' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state
                            .columns
                            .set_to_sorted_index_from_type(&processes::ProcessSorting::CpuPercent);
                        proc_widget_state.update_sorting_with_columns();
                        self.proc_state.force_update = Some(self.current_widget.widget_id);
                    }
                }
            }
            'm' => {
                if let BottomWidgetType::Proc = self.current_widget.widget_type {
                    if let Some(proc_widget_state) = self
                        .proc_state
                        .get_mut_widget_state(self.current_widget.widget_id)
                    {
                        proc_widget_state.columns.set_to_sorted_index_from_type(
                            &(if proc_widget_state
                                .columns
                                .is_enabled(&processes::ProcessSorting::MemPercent)
                            {
                                processes::ProcessSorting::MemPercent
                            } else {
                                processes::ProcessSorting::Mem
                            }),
                        );
                        proc_widget_state.update_sorting_with_columns();
                        self.proc_state.force_update = Some(self.current_widget.widget_id);
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
                            proc_widget_state
                                .columns
                                .set_to_sorted_index_from_type(&processes::ProcessSorting::Pid);
                            proc_widget_state.update_sorting_with_columns();
                            self.proc_state.force_update = Some(self.current_widget.widget_id);
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
                        proc_widget_state.requires_redraw = true;
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
                        proc_widget_state.columns.set_to_sorted_index_from_type(
                            &(if proc_widget_state.is_using_command {
                                processes::ProcessSorting::Command
                            } else {
                                processes::ProcessSorting::ProcessName
                            }),
                        );
                        proc_widget_state.update_sorting_with_columns();
                        self.proc_state.force_update = Some(self.current_widget.widget_id);
                    }
                }
            }
            '?' => {
                self.help_dialog.show();
                self.is_force_redraw = true;
            }
            'H' | 'A' => self.move_widget_selection(&WidgetDirection::Left),
            'L' | 'D' => self.move_widget_selection(&WidgetDirection::Right),
            'K' | 'W' => self.move_widget_selection(&WidgetDirection::Up),
            'J' | 'S' => self.move_widget_selection(&WidgetDirection::Down),
            't' => self.toggle_tree_mode(),
            '+' => self.on_plus(),
            '-' => self.on_minus(),
            '=' => self.reset_zoom(),
            'e' => self.toggle_expand_widget(),
            's' => self.toggle_sort(),
            'I' => self.invert_sort(),
            '%' => self.toggle_percentages(),
            ' ' => self.on_space(),
            _ => {}
        }

        if let Some(second_char) = self.second_char {
            if self.awaiting_second_char && caught_char != second_char {
                self.awaiting_second_char = false;
            }
        }
    }

    pub fn on_space(&mut self) {}

    pub fn kill_highlighted_process(&mut self) -> Result<()> {
        if let BottomWidgetType::Proc = self.current_widget.widget_type {
            if let Some(current_selected_processes) = &self.to_delete_process_list {
                #[cfg(target_family = "unix")]
                let signal = match self.delete_dialog_state.selected_signal {
                    KillSignal::Kill(sig) => sig,
                    KillSignal::Cancel => 15, // should never happen, so just TERM
                };
                for pid in &current_selected_processes.1 {
                    #[cfg(target_family = "unix")]
                    {
                        process_killer::kill_process_given_pid(*pid, signal)?;
                    }
                    #[cfg(target_os = "windows")]
                    {
                        process_killer::kill_process_given_pid(*pid)?;
                    }
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

    pub fn get_to_delete_processes(&self) -> Option<(String, Vec<Pid>)> {
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
        /*
            The actual logic for widget movement.

            We follow these following steps:
            1. Send a movement signal in `direction`.
            2. Check if this new widget we've landed on is hidden.  If not, halt.
            3. If it hidden, loop and either send:
               - A signal equal to the current direction, if it is opposite of the reflection.
               - Reflection direction.
        */

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
                                        &mut self.basic_table_widget_state
                                    {
                                        // We also want to move towards Proc if we had set it to ProcSort.
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
                            self.current_widget = proc_sort_widget.clone();
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
        if !self.ignore_normal_keybinds() {
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
        } else if self.help_dialog.is_showing() {
            // self.help_dialog_state.scroll_state.current_scroll_index = 0;
        } else if self.delete_dialog_state.is_showing_dd {
            self.delete_dialog_state.selected_signal = KillSignal::Cancel;
        }
    }

    pub fn skip_to_last(&mut self) {}

    pub fn decrement_position_count(&mut self) {
        if !self.ignore_normal_keybinds() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    self.increment_process_position(-1);
                }
                BottomWidgetType::ProcSort => self.increment_process_sort_position(-1),
                BottomWidgetType::Temp => self.increment_temp_position(-1),
                BottomWidgetType::Disk => self.increment_disk_position(-1),
                BottomWidgetType::CpuLegend => self.increment_cpu_legend_position(-1),
                _ => {}
            }
        }
    }

    pub fn increment_position_count(&mut self) {
        if !self.ignore_normal_keybinds() {
            match self.current_widget.widget_type {
                BottomWidgetType::Proc => {
                    self.increment_process_position(1);
                }
                BottomWidgetType::ProcSort => self.increment_process_sort_position(1),
                BottomWidgetType::Temp => self.increment_temp_position(1),
                BottomWidgetType::Disk => self.increment_disk_position(1),
                BottomWidgetType::CpuLegend => self.increment_cpu_legend_position(1),
                _ => {}
            }
        }
    }

    fn increment_process_sort_position(&mut self, num_to_change_by: i64) {
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

    fn increment_cpu_legend_position(&mut self, num_to_change_by: i64) {
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

    /// Returns the new position.
    fn increment_process_position(&mut self, _num_to_change_by: i64) -> Option<usize> {
        None
    }

    fn increment_temp_position(&mut self, num_to_change_by: i64) {
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

    fn increment_disk_position(&mut self, num_to_change_by: i64) {
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
        // if self.help_dialog_state.scroll_state.current_scroll_index > 0 {
        //     self.help_dialog_state.scroll_state.current_scroll_index -= 1;
        // }
    }

    fn help_scroll_down(&mut self) {
        // if self.help_dialog_state.scroll_state.current_scroll_index + 1
        //     < self.help_dialog_state.scroll_state.max_scroll_index
        // {
        //     self.help_dialog_state.scroll_state.current_scroll_index += 1;
        // }
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

    fn toggle_collapsing_process_branch(&mut self) {}

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
