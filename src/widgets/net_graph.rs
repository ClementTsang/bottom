use std::time::Instant;

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}
