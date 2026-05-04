use std::time::Instant;

use crate::widgets::TimeseriesState;

pub struct MemWidgetState {
    pub time_series_state: TimeseriesState,
}

impl MemWidgetState {
    pub fn init(starting_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            time_series_state: TimeseriesState::new(starting_time)
                .with_autohide_timer(autohide_timer),
        }
    }
}
