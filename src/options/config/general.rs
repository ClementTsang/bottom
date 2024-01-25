use clap::ArgMatches;
use serde::Deserialize;

use super::StringOrNum;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GeneralConfig {
    #[serde(default)]
    pub(crate) autohide_time: bool,
    #[serde(default)]
    pub(crate) basic: bool,
    pub(crate) default_time_value: Option<StringOrNum>,
    pub(crate) default_widget_type: Option<String>,
    #[serde(default)]
    pub(crate) disable_click: bool,
    #[serde(default)]
    pub(crate) dot_marker: bool, // TODO: Support other markers!
    #[serde(default)]
    pub(crate) expanded: bool,
    #[serde(default)]
    pub(crate) hide_table_gap: bool,
    #[serde(default)]
    pub(crate) hide_time: bool, // TODO: Combine with autohide_time
    #[serde(default)]
    pub(crate) left_legend: bool,
    pub(crate) rate: Option<StringOrNum>,
    pub(crate) retention: Option<StringOrNum>,
    #[serde(default)]
    pub(crate) show_table_scroll_position: bool,
    pub(crate) time_delta: Option<StringOrNum>,
}

impl GeneralConfig {
    pub(crate) fn merge_with_args(&mut self, args: &ArgMatches) {}
}
