use std::time::Instant;

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    pub cached_height_adjustment_range: Option<(Instant, Instant)>,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            cached_height_adjustment_range: None,
        }
    }
}
