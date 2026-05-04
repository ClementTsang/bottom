use std::time::Instant;

use crate::widgets::{GraphHeightCache, TimeseriesState};

pub struct NetWidgetState {
    pub timeseries_state: TimeseriesState,
    pub height_cache: GraphHeightCache,
}

impl NetWidgetState {
    pub fn init(starting_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            timeseries_state: TimeseriesState::new(starting_time)
                .with_autohide_timer(autohide_timer),
            height_cache: GraphHeightCache::default(),
        }
    }
}
