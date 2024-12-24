use serde::{Deserialize, Serialize};

use super::StringOrNum;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct FlagConfig {
    pub(crate) hide_avg_cpu: Option<bool>,
    pub(crate) dot_marker: Option<bool>,
    pub(crate) temperature_type: Option<String>,
    pub(crate) rate: Option<StringOrNum>,
    pub(crate) cpu_left_legend: Option<bool>,
    pub(crate) current_usage: Option<bool>,
    pub(crate) unnormalized_cpu: Option<bool>,
    pub(crate) group_processes: Option<bool>,
    pub(crate) case_sensitive: Option<bool>,
    pub(crate) whole_word: Option<bool>,
    pub(crate) regex: Option<bool>,
    pub(crate) basic: Option<bool>,
    pub(crate) default_time_value: Option<StringOrNum>,
    pub(crate) time_delta: Option<StringOrNum>,
    pub(crate) autohide_time: Option<bool>,
    pub(crate) hide_time: Option<bool>,
    pub(crate) default_widget_type: Option<String>,
    pub(crate) default_widget_count: Option<u64>,
    pub(crate) expanded: Option<bool>,
    pub(crate) use_old_network_legend: Option<bool>,
    pub(crate) hide_table_gap: Option<bool>,
    pub(crate) battery: Option<bool>,
    pub(crate) disable_click: Option<bool>,
    pub(crate) no_write: Option<bool>,
    pub(crate) network_legend: Option<String>,
    pub(crate) memory_legend: Option<String>,
    pub(crate) process_memory_as_value: Option<bool>,
    pub(crate) tree: Option<bool>,
    pub(crate) show_table_scroll_position: Option<bool>,
    pub(crate) process_command: Option<bool>,
    pub(crate) disable_advanced_kill: Option<bool>,
    pub(crate) network_use_bytes: Option<bool>,
    pub(crate) network_use_log: Option<bool>,
    pub(crate) network_use_binary_prefix: Option<bool>,
    pub(crate) disable_gpu: Option<bool>,
    pub(crate) enable_cache_memory: Option<bool>,
    pub(crate) retention: Option<StringOrNum>,
    pub(crate) average_cpu_row: Option<bool>,
    pub(crate) tree_collapse: Option<bool>,
}
