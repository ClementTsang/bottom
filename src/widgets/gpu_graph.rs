use crate::app::AppConfigFields;

pub struct GpuWidgetState {
    pub current_display_time: u64,
}

impl GpuWidgetState {
    pub fn new(_config: &AppConfigFields, current_display_time: u64) -> Self {
        GpuWidgetState {
            current_display_time,
        }
    }
}
