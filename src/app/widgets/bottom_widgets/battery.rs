use std::collections::HashMap;

use tui::{layout::Rect, widgets::Borders};

use crate::{
    app::{data_farmer::DataCollection, Component, Widget},
    data_conversion::{convert_battery_harvest, ConvertedBatteryData},
    options::layout_options::LayoutRule,
};

#[derive(Default)]
pub struct BatteryWidgetState {
    pub currently_selected_battery_index: usize,
    pub tab_click_locs: Option<Vec<((u16, u16), (u16, u16))>>,
}

#[derive(Default)]
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
    selected_index: usize,
    batteries: Vec<String>,
    battery_data: Vec<ConvertedBatteryData>,
    width: LayoutRule,
    height: LayoutRule,
    block_border: Borders,
}

impl Default for BatteryTable {
    fn default() -> Self {
        Self {
            batteries: vec![],
            bounds: Default::default(),
            selected_index: 0,
            battery_data: Default::default(),
            width: LayoutRule::default(),
            height: LayoutRule::default(),
            block_border: Borders::ALL,
        }
    }
}

impl BatteryTable {
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

    /// Returns the index of the currently selected battery.
    pub fn index(&self) -> usize {
        self.selected_index
    }

    /// Returns a reference to the battery names.
    pub fn batteries(&self) -> &[String] {
        &self.batteries
    }

    /// Sets the block border style.
    pub fn basic_mode(mut self, basic_mode: bool) -> Self {
        if basic_mode {
            self.block_border = *crate::constants::SIDE_BORDERS;
        }

        self
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

    fn update_data(&mut self, data_collection: &DataCollection) {
        self.battery_data = convert_battery_harvest(data_collection);
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }
}
