use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    app::{layout_manager::*, *},
    canvas::{ColourScheme, Painter},
    constants::*,
    units::data_units::DataUnit,
    utils::error::{self, BottomError},
};

use layout_options::*;

pub mod layout_options;

use anyhow::{Context, Result};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub flags: Option<ConfigFlags>,
    pub colors: Option<ConfigColours>,
    pub row: Option<Vec<Row>>,
    pub disk_filter: Option<IgnoreList>,
    pub mount_filter: Option<IgnoreList>,
    pub temp_filter: Option<IgnoreList>,
    pub net_filter: Option<IgnoreList>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigFlags {
    pub hide_avg_cpu: Option<bool>,

    pub dot_marker: Option<bool>,

    pub temperature_type: Option<String>,

    pub rate: Option<u64>,

    pub left_legend: Option<bool>,

    pub current_usage: Option<bool>,

    pub group_processes: Option<bool>,

    pub case_sensitive: Option<bool>,

    pub whole_word: Option<bool>,

    pub regex: Option<bool>,

    pub basic: Option<bool>,

    pub default_time_value: Option<u64>,

    pub time_delta: Option<u64>,

    pub autohide_time: Option<bool>,

    pub hide_time: Option<bool>,

    pub use_old_network_legend: Option<bool>,

    pub hide_table_gap: Option<bool>,

    pub battery: Option<bool>,

    pub disable_click: Option<bool>,

    pub no_write: Option<bool>,

    // For built-in colour palettes.
    pub color: Option<String>,

    pub search_case_enabled_widgets: Option<Vec<WidgetIdEnabled>>,

    pub search_whole_word_enabled_widgets: Option<Vec<WidgetIdEnabled>>,

    pub search_regex_enabled_widgets: Option<Vec<WidgetIdEnabled>>,

    pub mem_as_value: Option<bool>,

    pub tree: Option<bool>,

    show_table_scroll_position: Option<bool>,

    pub process_command: Option<bool>,

    pub disable_advanced_kill: Option<bool>,

    pub network_use_bytes: Option<bool>,

    pub network_use_log: Option<bool>,

    pub network_use_binary_prefix: Option<bool>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct WidgetIdEnabled {
    id: u64,
    enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigColours {
    pub table_header_color: Option<String>,
    pub all_cpu_color: Option<String>,
    pub avg_cpu_color: Option<String>,
    pub cpu_core_colors: Option<Vec<String>>,
    pub ram_color: Option<String>,
    pub swap_color: Option<String>,
    pub rx_color: Option<String>,
    pub tx_color: Option<String>,
    pub rx_total_color: Option<String>, // These only affect basic mode.
    pub tx_total_color: Option<String>, // These only affect basic mode.
    pub border_color: Option<String>,
    pub highlighted_border_color: Option<String>,
    pub disabled_text_color: Option<String>,
    pub text_color: Option<String>,
    pub selected_text_color: Option<String>,
    pub selected_bg_color: Option<String>,
    pub widget_title_color: Option<String>,
    pub graph_color: Option<String>,
    pub high_battery_color: Option<String>,
    pub medium_battery_color: Option<String>,
    pub low_battery_color: Option<String>,
}

impl ConfigColours {
    pub fn is_empty(&self) -> bool {
        if let Ok(serialized_string) = toml::to_string(self) {
            if !serialized_string.is_empty() {
                return false;
            }
        }

        true
    }
}

/// Workaround as per https://github.com/serde-rs/serde/issues/1030
fn default_as_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IgnoreList {
    #[serde(default = "default_as_true")]
    // TODO: [Config] Deprecate and/or rename, current name sounds awful.
    // Maybe to something like "deny_entries"?  Currently it defaults to a denylist anyways, so maybe "allow_entries"?
    pub is_list_ignored: bool,
    pub list: Vec<String>,
    #[serde(default = "bool::default")]
    pub regex: bool,
    #[serde(default = "bool::default")]
    pub case_sensitive: bool,
    #[serde(default = "bool::default")]
    pub whole_word: bool,
}

/// Represents the default states of all process widgets.
pub struct ProcessDefaults {
    pub is_grouped: bool,
    pub is_case_sensitive: bool,
    pub is_match_whole_word: bool,
    pub is_use_regex: bool,
    pub is_show_mem_as_values: bool,
    pub is_tree: bool,
    pub is_command: bool,
}

pub fn build_app(matches: &clap::ArgMatches<'static>, config: &mut Config) -> Result<AppState> {
    // Process defaults
    let process_defaults = ProcessDefaults {
        is_grouped: get_process_grouping(matches, config),
        is_case_sensitive: get_case_sensitive(matches, config),
        is_match_whole_word: get_match_whole_word(matches, config),
        is_use_regex: get_use_regex(matches, config),
        is_show_mem_as_values: get_mem_as_value(matches, config),
        is_tree: get_is_default_tree(matches, config),
        is_command: get_is_default_process_command(matches, config),
    };

    // App config fields
    let autohide_time = get_autohide_time(matches, config);
    let default_time_value = get_default_time_value(matches, config)
        .context("Update 'default_time_value' in your config file.")?;
    let use_basic_mode = get_use_basic_mode(matches, config);
    let is_advanced_kill = !get_is_advanced_kill_disabled(matches, config);
    let network_unit_type = get_network_unit_type(matches, config);
    let network_scale_type = get_network_scale_type(matches, config);
    let network_use_binary_prefix = get_network_use_binary_prefix(matches, config);

    let app_config_fields = AppConfigFields {
        update_rate_in_milliseconds: get_update_rate_in_milliseconds(matches, config)
            .context("Update 'rate' in your config file.")?,
        temperature_type: get_temperature(matches, config)
            .context("Update 'temperature_type' in your config file.")?,
        show_average_cpu: get_show_average_cpu(matches, config),
        use_dot: get_use_dot(matches, config),
        left_legend: get_use_left_legend(matches, config),
        use_current_cpu_total: get_use_current_cpu_total(matches, config),
        use_basic_mode,
        default_time_value,
        time_interval: get_time_interval(matches, config)
            .context("Update 'time_delta' in your config file.")?,
        hide_time: get_hide_time(matches, config),
        autohide_time,
        use_old_network_legend: get_use_old_network_legend(matches, config),
        table_gap: !get_hide_table_gap(matches, config),
        disable_click: get_disable_click(matches, config),
        // no_write: get_no_write(matches, config),
        no_write: false,
        show_table_scroll_position: get_show_table_scroll_position(matches, config),
        is_advanced_kill,
        network_scale_type,
        network_unit_type,
        network_use_binary_prefix,
    };

    let layout_tree_output = if let Some(row) = &config.row {
        create_layout_tree(row, process_defaults, &app_config_fields)?
    } else if get_use_basic_mode(matches, config) {
        if get_use_battery(matches, config) {
            let rows = toml::from_str::<Config>(DEFAULT_BASIC_BATTERY_LAYOUT)?
                .row
                .unwrap();
            create_layout_tree(&rows, process_defaults, &app_config_fields)?
        } else {
            let rows = toml::from_str::<Config>(DEFAULT_BASIC_LAYOUT)?.row.unwrap();
            create_layout_tree(&rows, process_defaults, &app_config_fields)?
        }
    } else if get_use_battery(matches, config) {
        let rows = toml::from_str::<Config>(DEFAULT_BATTERY_LAYOUT)?
            .row
            .unwrap();
        create_layout_tree(&rows, process_defaults, &app_config_fields)?
    } else {
        let rows = toml::from_str::<Config>(DEFAULT_LAYOUT)?.row.unwrap();
        create_layout_tree(&rows, process_defaults, &app_config_fields)?
    };

    let disk_filter =
        get_ignore_list(&config.disk_filter).context("Update 'disk_filter' in your config file")?;
    let mount_filter = get_ignore_list(&config.mount_filter)
        .context("Update 'mount_filter' in your config file")?;
    let temp_filter =
        get_ignore_list(&config.temp_filter).context("Update 'temp_filter' in your config file")?;
    let net_filter =
        get_ignore_list(&config.net_filter).context("Update 'net_filter' in your config file")?;
    let data_filter = DataFilters {
        disk_filter,
        mount_filter,
        temp_filter,
        net_filter,
    };

    let painter = Painter::init(&config, get_color_scheme(&matches, &config)?)?;
    AppState::new(app_config_fields, data_filter, layout_tree_output, painter)
}

fn get_update_rate_in_milliseconds(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<u64> {
    let update_rate_in_milliseconds = if let Some(update_rate) = matches.value_of("rate") {
        update_rate.parse::<u128>()?
    } else if let Some(flags) = &config.flags {
        if let Some(rate) = flags.rate {
            rate as u128
        } else {
            DEFAULT_REFRESH_RATE_IN_MILLISECONDS as u128
        }
    } else {
        DEFAULT_REFRESH_RATE_IN_MILLISECONDS as u128
    };

    if update_rate_in_milliseconds < 250 {
        return Err(BottomError::ConfigError(
            "set your update rate to be at least 250 milliseconds.".to_string(),
        ));
    } else if update_rate_in_milliseconds as u128 > std::u64::MAX as u128 {
        return Err(BottomError::ConfigError(
            "set your update rate to be at most unsigned INT_MAX.".to_string(),
        ));
    }

    Ok(update_rate_in_milliseconds as u64)
}

fn get_temperature(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<data_harvester::temperature::TemperatureType> {
    if matches.is_present("fahrenheit") {
        return Ok(data_harvester::temperature::TemperatureType::Fahrenheit);
    } else if matches.is_present("kelvin") {
        return Ok(data_harvester::temperature::TemperatureType::Kelvin);
    } else if matches.is_present("celsius") {
        return Ok(data_harvester::temperature::TemperatureType::Celsius);
    } else if let Some(flags) = &config.flags {
        if let Some(temp_type) = &flags.temperature_type {
            // Give lowest priority to config.
            return match temp_type.as_str() {
                "fahrenheit" | "f" => Ok(data_harvester::temperature::TemperatureType::Fahrenheit),
                "kelvin" | "k" => Ok(data_harvester::temperature::TemperatureType::Kelvin),
                "celsius" | "c" => Ok(data_harvester::temperature::TemperatureType::Celsius),
                _ => Err(BottomError::ConfigError(format!(
                    "\"{}\" is an invalid temperature type, use \"<kelvin|k|celsius|c|fahrenheit|f>\".",
                    temp_type
                ))),
            };
        }
    }
    Ok(data_harvester::temperature::TemperatureType::Celsius)
}

/// Yes, this function gets whether to show average CPU (true) or not (false)
fn get_show_average_cpu(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("hide_avg_cpu") {
        return false;
    } else if let Some(flags) = &config.flags {
        if let Some(avg_cpu) = flags.hide_avg_cpu {
            return !avg_cpu;
        }
    }

    true
}

fn get_use_dot(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("dot_marker") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(dot_marker) = flags.dot_marker {
            return dot_marker;
        }
    }
    false
}

fn get_use_left_legend(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("left_legend") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(left_legend) = flags.left_legend {
            return left_legend;
        }
    }

    false
}

fn get_use_current_cpu_total(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("current_usage") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(current_usage) = flags.current_usage {
            return current_usage;
        }
    }

    false
}

fn get_use_basic_mode(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("basic") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(basic) = flags.basic {
            return basic;
        }
    }

    false
}

fn get_default_time_value(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<u64> {
    let default_time = if let Some(default_time_value) = matches.value_of("default_time_value") {
        default_time_value.parse::<u128>()?
    } else if let Some(flags) = &config.flags {
        if let Some(default_time_value) = flags.default_time_value {
            default_time_value as u128
        } else {
            DEFAULT_TIME_MILLISECONDS as u128
        }
    } else {
        DEFAULT_TIME_MILLISECONDS as u128
    };

    if default_time < 30000 {
        return Err(BottomError::ConfigError(
            "set your default value to be at least 30000 milliseconds.".to_string(),
        ));
    } else if default_time as u128 > STALE_MAX_MILLISECONDS as u128 {
        return Err(BottomError::ConfigError(format!(
            "set your default value to be at most {} milliseconds.",
            STALE_MAX_MILLISECONDS
        )));
    }

    Ok(default_time as u64)
}

fn get_time_interval(matches: &clap::ArgMatches<'static>, config: &Config) -> error::Result<u64> {
    let time_interval = if let Some(time_interval) = matches.value_of("time_delta") {
        time_interval.parse::<u128>()?
    } else if let Some(flags) = &config.flags {
        if let Some(time_interval) = flags.time_delta {
            time_interval as u128
        } else {
            TIME_CHANGE_MILLISECONDS as u128
        }
    } else {
        TIME_CHANGE_MILLISECONDS as u128
    };

    if time_interval < 1000 {
        return Err(BottomError::ConfigError(
            "set your time delta to be at least 1000 milliseconds.".to_string(),
        ));
    } else if time_interval > STALE_MAX_MILLISECONDS as u128 {
        return Err(BottomError::ConfigError(format!(
            "set your time delta to be at most {} milliseconds.",
            STALE_MAX_MILLISECONDS
        )));
    }

    Ok(time_interval as u64)
}

pub fn get_process_grouping(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("group") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(grouping) = flags.group_processes {
            return grouping;
        }
    }
    false
}

pub fn get_case_sensitive(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("case_sensitive") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(case_sensitive) = flags.case_sensitive {
            return case_sensitive;
        }
    }
    false
}

pub fn get_match_whole_word(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("whole_word") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(whole_word) = flags.whole_word {
            return whole_word;
        }
    }
    false
}

pub fn get_use_regex(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("regex") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(regex) = flags.regex {
            return regex;
        }
    }
    false
}

fn get_hide_time(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("hide_time") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(hide_time) = flags.hide_time {
            return hide_time;
        }
    }
    false
}

fn get_autohide_time(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("autohide_time") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(autohide_time) = flags.autohide_time {
            return autohide_time;
        }
    }

    false
}

fn get_disable_click(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("disable_click") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(disable_click) = flags.disable_click {
            return disable_click;
        }
    }
    false
}

fn get_use_old_network_legend(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("use_old_network_legend") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(use_old_network_legend) = flags.use_old_network_legend {
            return use_old_network_legend;
        }
    }
    false
}

fn get_hide_table_gap(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("hide_table_gap") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(hide_table_gap) = flags.hide_table_gap {
            return hide_table_gap;
        }
    }
    false
}

fn get_use_battery(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("battery") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(battery) = flags.battery {
            return battery;
        }
    }
    false
}

fn get_ignore_list(ignore_list: &Option<IgnoreList>) -> error::Result<Option<Filter>> {
    if let Some(ignore_list) = ignore_list {
        let list: Result<Vec<_>, _> = ignore_list
            .list
            .iter()
            .map(|name| {
                let escaped_string: String;
                let res = format!(
                    "{}{}{}{}",
                    if ignore_list.whole_word { "^" } else { "" },
                    if ignore_list.case_sensitive {
                        ""
                    } else {
                        "(?i)"
                    },
                    if ignore_list.regex {
                        name
                    } else {
                        escaped_string = regex::escape(name);
                        &escaped_string
                    },
                    if ignore_list.whole_word { "$" } else { "" },
                );

                Regex::new(&res)
            })
            .collect();

        Ok(Some(Filter {
            list: list?,
            is_list_ignored: ignore_list.is_list_ignored,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_color_scheme(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<ColourScheme> {
    if let Some(color) = matches.value_of("color") {
        // Highest priority is always command line flags...
        return ColourScheme::from_str(color);
    } else if let Some(colors) = &config.colors {
        if !colors.is_empty() {
            // Then, give priority to custom colours...
            return Ok(ColourScheme::Custom);
        } else if let Some(flags) = &config.flags {
            // Last priority is config file flags...
            if let Some(color) = &flags.color {
                return ColourScheme::from_str(color);
            }
        }
    } else if let Some(flags) = &config.flags {
        // Last priority is config file flags...
        if let Some(color) = &flags.color {
            return ColourScheme::from_str(color);
        }
    }

    // And lastly, the final case is just "default".
    Ok(ColourScheme::Default)
}

fn get_mem_as_value(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("mem_as_value") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(mem_as_value) = flags.mem_as_value {
            return mem_as_value;
        }
    }
    false
}

fn get_is_default_tree(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("tree") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(tree) = flags.tree {
            return tree;
        }
    }
    false
}

fn get_show_table_scroll_position(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("show_table_scroll_position") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(show_table_scroll_position) = flags.show_table_scroll_position {
            return show_table_scroll_position;
        }
    }
    false
}

fn get_is_default_process_command(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("process_command") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(process_command) = flags.process_command {
            return process_command;
        }
    }
    false
}

fn get_is_advanced_kill_disabled(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("disable_advanced_kill") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(disable_advanced_kill) = flags.disable_advanced_kill {
            return disable_advanced_kill;
        }
    }
    false
}

fn get_network_unit_type(matches: &clap::ArgMatches<'static>, config: &Config) -> DataUnit {
    if matches.is_present("network_use_bytes") {
        return DataUnit::Byte;
    } else if let Some(flags) = &config.flags {
        if let Some(network_use_bytes) = flags.network_use_bytes {
            if network_use_bytes {
                return DataUnit::Byte;
            }
        }
    }

    DataUnit::Bit
}

fn get_network_scale_type(matches: &clap::ArgMatches<'static>, config: &Config) -> AxisScaling {
    if matches.is_present("network_use_log") {
        return AxisScaling::Log;
    } else if let Some(flags) = &config.flags {
        if let Some(network_use_log) = flags.network_use_log {
            if network_use_log {
                return AxisScaling::Log;
            }
        }
    }

    AxisScaling::Linear
}

fn get_network_use_binary_prefix(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("network_use_binary_prefix") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(network_use_binary_prefix) = flags.network_use_binary_prefix {
            return network_use_binary_prefix;
        }
    }
    false
}
