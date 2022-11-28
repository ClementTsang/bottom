#[derive(Default)]
pub struct BatteryWidgetState {
    pub currently_selected_battery_index: usize,
    pub tab_click_locs: Option<Vec<((u16, u16), (u16, u16))>>,
}
