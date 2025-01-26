use std::time::Instant;

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl MemWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}
