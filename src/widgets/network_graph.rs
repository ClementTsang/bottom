use std::time::Instant;

use crate::widgets::GraphHeightCache;

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    pub height_cache: GraphHeightCache,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            height_cache: GraphHeightCache::default(),
        }
    }
}
