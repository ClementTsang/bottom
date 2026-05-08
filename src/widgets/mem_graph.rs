use std::time::Instant;

use crate::widgets::{TimeseriesConfig, TimeseriesState};

pub struct MemWidgetState {
    pub time_series_state: TimeseriesState,
}

impl MemWidgetState {
    pub fn init(ts_config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            time_series_state: TimeseriesState::new(ts_config, autohide_timer),
        }
    }
}
