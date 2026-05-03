//! Code around a temperature graph widget.

use std::time::Instant;

use crate::widgets::GraphHeightCache;

/// A timeseries graph widget displaying temperature usage over time.
pub struct TempGraphWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    pub height_cache: GraphHeightCache,
    pub max_temp: Option<f32>,
}

impl TempGraphWidgetState {
    pub fn new(
        current_display_time: u64, autohide_timer: Option<Instant>, max_temp: Option<f32>,
    ) -> Self {
        TempGraphWidgetState {
            current_display_time,
            autohide_timer,
            height_cache: GraphHeightCache::default(),
            max_temp,
        }
    }
}
