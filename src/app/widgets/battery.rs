use std::collections::HashMap;

use tui::layout::Rect;

use super::{Component, Widget};

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

// TODO: Implement battery widget.
/// A table displaying battery information on a per-battery basis.
pub struct BatteryTable {
    bounds: Rect,
}

impl BatteryTable {
    /// Creates a new [`BatteryTable`].
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
        }
    }
}

impl Component for BatteryTable {
    fn bounds(&self) -> tui::layout::Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: tui::layout::Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for BatteryTable {
    fn get_pretty_name(&self) -> &'static str {
        "Battery"
    }
}
