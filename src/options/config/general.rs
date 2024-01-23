use serde::Deserialize;

use super::StringOrNum;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GeneralConfig {
    pub(crate) autohide_time: Option<bool>,
    pub(crate) basic: Option<bool>,
    pub(crate) default_time_value: Option<StringOrNum>,
    pub(crate) default_widget_type: Option<String>,
    pub(crate) disable_click: Option<bool>,
    pub(crate) dot_marker: Option<bool>, // TODO: Support other markers!
    pub(crate) expanded: Option<bool>,
    pub(crate) hide_table_gap: Option<bool>,
    pub(crate) hide_time: Option<bool>, // TODO: Combine with autohide_time
    pub(crate) left_legend: Option<bool>,
    pub(crate) rate: Option<StringOrNum>,
    pub(crate) retention: Option<StringOrNum>,
    pub(crate) show_table_scroll_position: Option<bool>,
    pub(crate) time_delta: Option<StringOrNum>,
}
