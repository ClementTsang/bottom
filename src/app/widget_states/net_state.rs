use std::{collections::HashMap, time::Instant};

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    // pub draw_max_range_cache: f64,
    // pub draw_labels_cache: Vec<String>,
    // pub draw_time_start_cache: f64,
    // TODO: Re-enable these when we move net details state-side!
    // pub unit_type: DataUnitTypes,
    // pub scale_type: AxisScaling,
}

impl NetWidgetState {
    pub fn init(
        current_display_time: u64,
        autohide_timer: Option<Instant>,
        // unit_type: DataUnitTypes,
        // scale_type: AxisScaling,
    ) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            // draw_max_range_cache: 0.0,
            // draw_labels_cache: vec![],
            // draw_time_start_cache: 0.0,
            // unit_type,
            // scale_type,
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
