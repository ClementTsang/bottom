#[cfg(feature = "battery")]
pub mod battery;
pub mod cpu;
pub mod general;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod layout;
pub mod memory;
pub mod network;
pub mod process;
mod style;
pub mod temperature;

use std::{fs, io::Write, path::PathBuf};

use indoc::indoc;
use serde::Deserialize;
pub use style::*;

#[cfg(feature = "battery")]
use self::battery::BatteryConfig;

#[cfg(feature = "gpu")]
use self::gpu::GpuConfig;

use self::{
    colours::ColourConfig, cpu::CpuConfig, general::GeneralConfig, layout::Row,
    memory::MemoryConfig, network::NetworkConfig, process::ProcessConfig,
    temperature::TemperatureConfig,
};
use crate::{args::BottomArgs, error};

pub const DEFAULT_CONFIG_FILE_PATH: &str = "bottom/bottom.toml";

pub(crate) trait DefaultConfig {
    fn default_config() -> String;
}

#[derive(Debug, Default, Deserialize)]
pub struct ConfigV2 {
    #[serde(default)]
    pub(crate) general: GeneralConfig,
    #[serde(default)]
    pub(crate) process: ProcessConfig,
    #[serde(default)]
    pub(crate) temperature: TemperatureConfig,
    #[serde(default)]
    pub(crate) cpu: CpuConfig,
    #[serde(default)]
    pub(crate) memory: MemoryConfig,
    #[serde(default)]
    pub(crate) network: NetworkConfig,
    #[cfg(feature = "battery")]
    #[serde(default)]
    pub(crate) battery: BatteryConfig,
    #[cfg(feature = "gpu")]
    #[serde(default)]
    pub(crate) gpu: GpuConfig,
    #[serde(default)]
    pub(crate) style: StyleConfig,

    // TODO: Merge these into above...
    pub(crate) colors: Option<ColourConfig>,
    pub(crate) row: Option<Vec<Row>>,
    pub(crate) disk_filter: Option<IgnoreList>,
    pub(crate) mount_filter: Option<IgnoreList>,
    pub(crate) temp_filter: Option<IgnoreList>,
    pub(crate) net_filter: Option<IgnoreList>,
}

impl ConfigV2 {
    /// Merges a [`BottomArgs`] with the internal shared "args" of the config file.
    ///
    /// In general, we favour whatever is set in `args` if it is set, then fall
    /// back to config value if set.
    pub fn merge(&mut self, args: BottomArgs) {
        self.general.args.merge(&args.general);
        self.process.args.merge(&args.process);
        self.temperature.args.merge(&args.temperature);
        self.cpu.args.merge(&args.cpu);
        self.memory.args.merge(&args.memory);
        self.network.args.merge(&args.network);
        #[cfg(feature = "battery")]
        self.battery.args.merge(&args.battery);
        #[cfg(feature = "gpu")]
        self.gpu.args.merge(&args.gpu);
        self.style.args.merge(&args.style);
    }
}

impl DefaultConfig for ConfigV2 {
    fn default_config() -> String {
        let mut str = String::default();

        pub const CONFIG_TOP_HEAD: &str = indoc! {"
            # This is bottom's config file. All of the settings are commented
            # out by default; if you wish to change them, uncomment and modify the
            # setting. Make sure to also comment out the relevant section header!

        "};

        str.push_str(CONFIG_TOP_HEAD);
        str.push_str(&GeneralConfig::default_config());
        str.push_str(&ProcessConfig::default_config());
        str.push_str(&TemperatureConfig::default_config());
        str.push_str(&CpuConfig::default_config());
        str.push_str(&MemoryConfig::default_config());
        str.push_str(&NetworkConfig::default_config());
        #[cfg(feature = "battery")]
        str.push_str(&BatteryConfig::default_config());
        #[cfg(feature = "gpu")]
        str.push_str(&GpuConfig::default_config());
        str.push_str(&StyleConfig::default_config());

        str
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

/// Get either the specified config path string or return the default one, if it exists.
pub fn get_config_path(override_path: Option<&str>) -> error::Result<Option<PathBuf>> {
    let config_path = if let Some(conf_loc) = override_path {
        Some(PathBuf::from(conf_loc))
    } else if cfg!(target_os = "windows") {
        if let Some(home_path) = dirs::config_dir() {
            let mut path = home_path;
            path.push(DEFAULT_CONFIG_FILE_PATH);
            Some(path)
        } else {
            None
        }
    } else if let Some(home_path) = dirs::home_dir() {
        let mut path = home_path;
        path.push(".config/");
        path.push(DEFAULT_CONFIG_FILE_PATH);
        if path.exists() {
            // If it already exists, use the old one.
            Some(path)
        } else {
            // If it does not, use the new one!
            if let Some(config_path) = dirs::config_dir() {
                let mut path = config_path;
                path.push(DEFAULT_CONFIG_FILE_PATH);
                Some(path)
            } else {
                None
            }
        }
    } else {
        None
    };

    Ok(config_path)
}

/// Either get an existing config or create a new one, and parse it.
pub fn create_or_get_config(config_path: &Option<PathBuf>) -> error::Result<ConfigV2> {
    if let Some(path) = config_path {
        if let Ok(config_string) = fs::read_to_string(path) {
            Ok(toml_edit::de::from_str(config_string.as_str())?)
        } else {
            if let Some(parent_path) = path.parent() {
                fs::create_dir_all(parent_path)?;
            }

            // TODO: Should this only create if we are on the default path?
            fs::File::create(path)?.write_all(ConfigV2::default_config().as_bytes())?;
            Ok(ConfigV2::default())
        }
    } else {
        // Don't write anything, just assume the default.
        Ok(ConfigV2::default())
    }
}
