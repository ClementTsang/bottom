pub mod battery;
pub mod cpu;
pub mod general;
pub mod gpu;
pub mod layout;
pub mod memory;
pub mod network;
pub mod process;
mod style;
pub mod temperature;

use serde::Deserialize;
pub use style::*;

use self::{colours::ColourConfig, cpu::CpuConfig, layout::Row, process::ProcessConfig};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    // #[serde(default)]
    // pub(crate) general_options: GeneralOptions,
    // #[serde(default)]
    // pub(crate) process_options: ProcessOptions,
    // #[serde(default)]
    // pub(crate) temperature_options: TemperatureOptions,
    // #[serde(default)]
    // pub(crate) cpu_options: CpuOptions,
    // #[serde(default)]
    // pub(crate) memory_options: MemoryOptions,
    // #[serde(default)]
    // pub(crate) network_options: NetworkOptions,
    // #[serde(default)]
    // pub(crate) battery_options: BatteryOptions,
    // #[serde(default)]
    // pub(crate) gpu_options: GpuOptions,
    // #[serde(default)]
    // pub(crate) style_options: StyleOptions,
    pub flags: Option<ConfigFlags>,

    // TODO: Merge these into above!
    pub(crate) colors: Option<ColourConfig>,
    pub(crate) row: Option<Vec<Row>>,
    pub(crate) disk_filter: Option<IgnoreList>,
    pub(crate) mount_filter: Option<IgnoreList>,
    pub(crate) temp_filter: Option<IgnoreList>,
    pub(crate) net_filter: Option<IgnoreList>,
    #[serde(default)]
    pub(crate) processes: ProcessConfig,
    #[serde(default)]
    pub(crate) cpu: CpuConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum StringOrNum {
    String(String),
    Num(u64),
}

impl From<String> for StringOrNum {
    fn from(value: String) -> Self {
        StringOrNum::String(value)
    }
}

impl From<u64> for StringOrNum {
    fn from(value: u64) -> Self {
        StringOrNum::Num(value)
    }
}

/// Workaround as per https://github.com/serde-rs/serde/issues/1030
fn default_as_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IgnoreList {
    #[serde(default = "default_as_true")]
    // TODO: Deprecate and/or rename, current name sounds awful.
    // Maybe to something like "deny_entries"?  Currently it defaults to a denylist anyways, so maybe "allow_entries"?
    pub is_list_ignored: bool,
    pub list: Vec<String>,
    #[serde(default)]
    pub regex: bool,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub whole_word: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConfigFlags {
    pub(crate) hide_avg_cpu: Option<bool>,
    pub(crate) dot_marker: Option<bool>,
    pub(crate) temperature_type: Option<String>,
    pub(crate) rate: Option<StringOrNum>,
    pub(crate) left_legend: Option<bool>,
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
    pub(crate) expanded_on_startup: Option<bool>,
    pub(crate) use_old_network_legend: Option<bool>,
    pub(crate) hide_table_gap: Option<bool>,
    pub(crate) battery: Option<bool>,
    pub(crate) disable_click: Option<bool>,
    pub(crate) color: Option<String>,
    pub(crate) mem_as_value: Option<bool>,
    pub(crate) tree: Option<bool>,
    pub(crate) show_table_scroll_position: Option<bool>,
    pub(crate) process_command: Option<bool>,
    pub(crate) disable_advanced_kill: Option<bool>,
    pub(crate) network_use_bytes: Option<bool>,
    pub(crate) network_use_log: Option<bool>,
    pub(crate) network_use_binary_prefix: Option<bool>,
    pub(crate) enable_gpu: Option<bool>,
    pub(crate) enable_cache_memory: Option<bool>,
    pub(crate) retention: Option<StringOrNum>,
}
