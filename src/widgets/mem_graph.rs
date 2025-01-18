use std::time::Instant;

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,

    pub ram_points_cache: Vec<(f64, f64)>, // TODO: Cache this, probably in graph widget
}

impl MemWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            current_display_time,
            autohide_timer,
            ram_points_cache: vec![],
        }
    }
}
