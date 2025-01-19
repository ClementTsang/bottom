use std::time::Instant;

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,

    // FIXME: REMOVE THESE
    pub ram_points_cache: Vec<(f64, f64)>,
    pub swap_points_cache: Vec<(f64, f64)>,
    #[cfg(not(target_os = "windows"))]
    pub cache_points_cache: Vec<(f64, f64)>,
}

impl MemWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            current_display_time,
            autohide_timer,
            ram_points_cache: vec![],
            swap_points_cache: vec![],
            cache_points_cache: vec![],
        }
    }
}
