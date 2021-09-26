pub mod data_farmer;
pub mod data_harvester;
pub mod event;
pub mod filter;
pub mod layout_manager;
mod process_killer;
pub mod query;
pub mod widgets;

use std::{collections::HashMap, time::Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use fxhash::FxHashMap;
use indextree::{Arena, NodeId};
use unicode_width::UnicodeWidthStr;

pub use data_farmer::*;
use data_harvester::{processes, temperature};
pub use filter::*;
use layout_manager::*;
pub use widgets::*;

use crate::{
    canvas, constants,
    units::data_units::DataUnit,
    utils::error::{BottomError, Result},
    BottomEvent, Pid,
};

use self::event::{ComponentEventResult, EventResult, ReturnSignal};

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
}
