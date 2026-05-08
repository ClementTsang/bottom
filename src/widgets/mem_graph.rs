use std::time::Instant;

use crate::widgets::{TimeseriesConfig, TimeseriesState};

pub struct MemWidgetState {
    pub time_series_state: TimeseriesState,
}

impl MemWidgetState {
    pub fn init(config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            time_series_state: TimeseriesState::new(config).with_autohide_timer(autohide_timer),
        }
    }
}
