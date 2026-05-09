use rustc_hash::FxHashMap as HashMap;

use crate::{
    app::layout_manager::BottomWidgetType,
    constants,
    utils::input::InputFieldState,
    widgets::{
        BatteryWidgetState, CpuWidgetState, DiskTableWidget, MemWidgetState, NetWidgetState,
        ProcWidgetState, TempGraphWidgetState, TempWidgetState, query::ProcessQuery,
    },
};

pub struct AppWidgetStates {
    pub cpu_state: CpuState,
    pub mem_state: MemState,
    pub net_state: NetState,
    pub proc_state: ProcState,
    pub temp_state: TempState,
    pub temp_graph_state: TempGraphStates,
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
    is_searching: bool,
    pub search_input_state: InputFieldState,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
            height: 0,
            scroll_state: ParagraphScrollState::default(),
            index_shortcuts: vec![0; constants::HELP_TEXT.len()],
            is_searching: false,
            search_input_state: InputFieldState::default(),
        }
    }
}

impl AppHelpDialogState {
    /// Get whether the search state is active.
    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    /// Enable search state.
    pub fn enable_search(&mut self) {
        self.is_searching = true;
    }

    /// Close the search.
    pub fn disable_search(&mut self) {
        self.is_searching = false;
    }
}

/// AppSearchState deals with generic searching (I might do this in the future).
#[derive(Default)]
pub struct AppSearchState {
    pub is_enabled: bool,
    pub is_invalid_search: bool,
    pub input_field_state: InputFieldState,

    /// The query. TODO: Merge this as one enum.
    pub query: Option<ProcessQuery>,
    pub error_message: Option<String>,
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
        self.input_field_state.current_query().is_empty() || self.is_invalid_search
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut NetWidgetState> {
        self.widget_states.get_mut(&widget_id)
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

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut MemWidgetState> {
        self.widget_states.get_mut(&widget_id)
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

pub struct TempGraphStates {
    pub widget_states: HashMap<u64, TempGraphWidgetState>,
}

impl TempGraphStates {
    pub fn init(widget_states: HashMap<u64, TempGraphWidgetState>) -> Self {
        TempGraphStates { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut TempGraphWidgetState> {
        self.widget_states.get_mut(&widget_id)
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
