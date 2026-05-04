//! Code around a temperature graph widget.

use std::time::Instant;

use crate::widgets::{GraphHeightCache, TimeseriesState};

/// A timeseries graph widget displaying temperature usage over time.
pub struct TempGraphWidgetState {
    pub timeseries_state: TimeseriesState,
    pub height_cache: GraphHeightCache,
    pub max_temp: Option<f32>,
}

impl TempGraphWidgetState {
    pub fn new(starting_time: u64, autohide_timer: Option<Instant>, max_temp: Option<f32>) -> Self {
        TempGraphWidgetState {
            timeseries_state: TimeseriesState::new(starting_time)
                .with_autohide_timer(autohide_timer),
            height_cache: GraphHeightCache::default(),
            max_temp,
        }
    }
}
