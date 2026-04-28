//! Code around a temperature graph widget.

use std::time::Instant;

/// A timeseries graph widget displaying temperature usage over time.
pub struct TemperatureGraphState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl TemperatureGraphState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        TemperatureGraphState {
            current_display_time,
            autohide_timer,
        }
    }
}
