use std::time::Instant;

use crate::widgets::TimeseriesState;

pub struct MemWidgetState {
    pub timeseries_state: TimeseriesState,
}

impl MemWidgetState {
    pub fn init(starting_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            timeseries_state: TimeseriesState::new(starting_time).autohide_timer(autohide_timer),
        }
    }
}
