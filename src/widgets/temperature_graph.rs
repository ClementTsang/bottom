//! Code around a temperature graph widget.

use std::time::Instant;

use crate::widgets::{GraphHeightCache, TimeseriesConfig, TimeseriesState};

/// A time_series graph widget displaying temperature usage over time.
pub struct TempGraphWidgetState {
    pub time_series_state: TimeseriesState,
    pub height_cache: GraphHeightCache,
    pub max_temp: Option<f32>,
}

impl TempGraphWidgetState {
    pub fn new(
        ts_config: TimeseriesConfig, autohide_timer: Option<Instant>, max_temp: Option<f32>,
    ) -> Self {
        TempGraphWidgetState {
            time_series_state: TimeseriesState::new(ts_config, autohide_timer),
            height_cache: GraphHeightCache::default(),
            max_temp,
        }
    }
}
