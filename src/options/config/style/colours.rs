use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ColourConfig {
    pub table_header_color: Option<Cow<'static, str>>,
    pub all_cpu_color: Option<Cow<'static, str>>,
    pub avg_cpu_color: Option<Cow<'static, str>>,
    pub cpu_core_colors: Option<Vec<Cow<'static, str>>>,
    pub ram_color: Option<Cow<'static, str>>,
    pub cache_color: Option<Cow<'static, str>>,
    pub swap_color: Option<Cow<'static, str>>,
    pub arc_color: Option<Cow<'static, str>>,
    pub gpu_core_colors: Option<Vec<Cow<'static, str>>>,
    pub rx_color: Option<Cow<'static, str>>,
    pub tx_color: Option<Cow<'static, str>>,
    pub rx_total_color: Option<Cow<'static, str>>, // These only affect basic mode.
    pub tx_total_color: Option<Cow<'static, str>>, // These only affect basic mode.
    pub border_color: Option<Cow<'static, str>>,
    pub highlighted_border_color: Option<Cow<'static, str>>,
    pub disabled_text_color: Option<Cow<'static, str>>,
    pub text_color: Option<Cow<'static, str>>,
    pub selected_text_color: Option<Cow<'static, str>>,
    pub selected_bg_color: Option<Cow<'static, str>>,
    pub widget_title_color: Option<Cow<'static, str>>,
    pub graph_color: Option<Cow<'static, str>>,
    pub high_battery_color: Option<Cow<'static, str>>,
    pub medium_battery_color: Option<Cow<'static, str>>,
    pub low_battery_color: Option<Cow<'static, str>>,
}

impl ColourConfig {
    /// Returns `true` if there is a [`ColourConfig`] that is empty or there isn't one at all.
    pub fn is_empty(&self) -> bool {
        if let Ok(serialized_string) = toml_edit::ser::to_string(self) {
            return serialized_string.is_empty();
        }

        true
    }
}
