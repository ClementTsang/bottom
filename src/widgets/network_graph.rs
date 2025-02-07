use std::time::Instant;

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    pub height_cache: Option<NetWidgetHeightCache>,
}

pub struct NetWidgetHeightCache {
    pub best_point: (Instant, f64),
    pub right_edge: Instant,
    pub period: u64,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            height_cache: None,
        }
    }
}
