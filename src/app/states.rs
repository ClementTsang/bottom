use std::{collections::HashMap, time::Instant};

use unicode_segmentation::GraphemeCursor;

use crate::{
    app::{layout_manager::BottomWidgetType, query::*},
    components::text_table::{CellContent, TableComponentColumn, TableComponentState, WidthBounds},
    constants,
};

use super::widgets::{DiskWidgetState, ProcWidget, TempWidgetState};

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

/// Meant for canvas operations involving table column widths.
#[derive(Default)]
pub struct CanvasTableWidthState {
    pub desired_column_widths: Vec<u16>,
    pub calculated_column_widths: Vec<u16>,
}

#[derive(PartialEq, Eq)]
pub enum KillSignal {
    Cancel,
    Kill(usize),
}

impl Default for KillSignal {
    #[cfg(target_family = "unix")]
    fn default() -> Self {
        KillSignal::Kill(15)
    }
    #[cfg(target_os = "windows")]
    fn default() -> Self {
        KillSignal::Kill(1)
    }
}

#[derive(Default)]
pub struct AppDeleteDialogState {
    pub is_showing_dd: bool,
    pub selected_signal: KillSignal,
    /// tl x, tl y, br x, br y, index/signal
    pub button_positions: Vec<(u16, u16, u16, u16, usize)>,
    pub keyboard_signal_select: usize,
    pub last_number_press: Option<Instant>,
    pub scroll_pos: usize,
}

pub struct AppHelpDialogState {
    pub is_showing_help: bool,
    pub height: u16,
    pub scroll_state: ParagraphScrollState,
    pub index_shortcuts: Vec<u16>,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
            height: 0,
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

pub struct ProcState {
    pub widget_states: HashMap<u64, ProcWidget>,
}

impl ProcState {
    pub fn init(widget_states: HashMap<u64, ProcWidget>) -> Self {
        ProcState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut ProcWidget> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&ProcWidget> {
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
    pub table_state: TableComponentState,
    pub is_multi_graph_mode: bool,
}

impl CpuWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
        const WIDTHS: [WidthBounds; CPU_LEGEND_HEADER.len()] = [
            WidthBounds::soft_from_str("CPU", Some(0.5)),
            WidthBounds::soft_from_str("Use%", Some(0.5)),
        ];

        let table_state = TableComponentState::new(
            CPU_LEGEND_HEADER
                .iter()
                .zip(WIDTHS)
                .map(|(c, width)| {
                    TableComponentColumn::new_custom(CellContent::new(*c, None), width)
                })
                .collect(),
        );

        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            autohide_timer,
            table_state,
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
    pub left_tlc: Option<(u16, u16)>,
    pub left_brc: Option<(u16, u16)>,
    pub right_tlc: Option<(u16, u16)>,
    pub right_brc: Option<(u16, u16)>,
}

#[derive(Default)]
pub struct BatteryWidgetState {
    pub currently_selected_battery_index: usize,
    pub tab_click_locs: Option<Vec<((u16, u16), (u16, u16))>>,
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
