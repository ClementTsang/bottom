use std::time::Instant;

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,

    // FIXME: (points_rework_v1) REMOVE THIS
    pub rx_cache: Vec<(f64, f64)>,
    pub tx_cache: Vec<(f64, f64)>,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            rx_cache: vec![],
            tx_cache: vec![],
        }
    }
}
