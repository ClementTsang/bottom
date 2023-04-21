use std::{
    borrow::Cow,
    convert::TryInto,
    str::FromStr,
    time::{Duration, Instant},
};

use clap::ArgMatches;
use hashbrown::{HashMap, HashSet};
use layout_options::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use typed_builder::*;

#[cfg(feature = "battery")]
use starship_battery::Manager;

use crate::{
    app::{filter::Filter, layout_manager::*, *},
    canvas::{canvas_styling::CanvasColours, ColourScheme},
    constants::*,
    units::data_units::DataUnit,
    utils::error::{self, BottomError},
    widgets::{
        BatteryWidgetState, CpuWidgetState, DiskTableWidget, MemWidgetState, NetWidgetState,
        ProcWidgetMode, ProcWidgetState, TempWidgetState,
    },
};

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

#[derive(Clone, Debug, Default, Deserialize, Serialize, TypedBuilder)]
pub struct ConfigFlags {
    pub hide_avg_cpu: Option<bool>,
    pub dot_marker: Option<bool>,
    pub temperature_type: Option<String>,
    pub rate: Option<u64>,
    pub left_legend: Option<bool>,
    pub current_usage: Option<bool>,
    pub unnormalized_cpu: Option<bool>,
    pub group_processes: Option<bool>,
    pub case_sensitive: Option<bool>,
    pub whole_word: Option<bool>,
    pub regex: Option<bool>,
    pub basic: Option<bool>,
    pub default_time_value: Option<u64>,
    pub time_delta: Option<u64>,
    pub autohide_time: Option<bool>,
    pub hide_time: Option<bool>,
    pub default_widget_type: Option<String>,
    pub default_widget_count: Option<u64>,
    pub expanded_on_startup: Option<bool>,
    pub use_old_network_legend: Option<bool>,
    pub hide_table_gap: Option<bool>,
    pub battery: Option<bool>,
    pub disable_click: Option<bool>,
    pub no_write: Option<bool>,
    /// For built-in colour palettes.
    pub color: Option<String>,
    pub mem_as_value: Option<bool>,
    pub tree: Option<bool>,
    show_table_scroll_position: Option<bool>,
    pub process_command: Option<bool>,
    pub disable_advanced_kill: Option<bool>,
    pub network_use_bytes: Option<bool>,
    pub network_use_log: Option<bool>,
    pub network_use_binary_prefix: Option<bool>,
    pub enable_gpu_memory: Option<bool>,
    pub enable_cache_memory: Option<bool>,
    #[serde(with = "humantime_serde")]
    #[serde(default)]
    pub retention: Option<Duration>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigColours {
    pub table_header_color: Option<Cow<'static, str>>,
    pub all_cpu_color: Option<Cow<'static, str>>,
    pub avg_cpu_color: Option<Cow<'static, str>>,
    pub cpu_core_colors: Option<Vec<Cow<'static, str>>>,
    pub ram_color: Option<Cow<'static, str>>,
    #[cfg(not(target_os = "windows"))]
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

impl ConfigColours {
    /// Returns `true` if there is a [`ConfigColours`] that is empty or there isn't one at all.
    pub fn is_empty(&self) -> bool {
        if let Ok(serialized_string) = toml_edit::ser::to_string(self) {
            return serialized_string.is_empty();
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
    // TODO: Deprecate and/or rename, current name sounds awful.
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

macro_rules! is_flag_enabled {
    ($flag_name:ident, $matches:expr, $config:expr) => {
        if $matches.get_flag(stringify!($flag_name)) {
            true
        } else if let Some(flags) = &$config.flags {
            flags.$flag_name.unwrap_or(false)
        } else {
            false
        }
    };

    ($cmd_flag:literal, $cfg_flag:ident, $matches:expr, $config:expr) => {
        if $matches.get_flag($cmd_flag) {
            true
        } else if let Some(flags) = &$config.flags {
            flags.$cfg_flag.unwrap_or(false)
        } else {
            false
        }
    };
}

pub fn build_app(
    matches: &ArgMatches, config: &mut Config, widget_layout: &BottomLayout,
    default_widget_id: u64, default_widget_type_option: &Option<BottomWidgetType>,
    colours: &CanvasColours,
) -> Result<App> {
    use BottomWidgetType::*;

    let retention_ms =
        get_retention_ms(matches, config).context("Update `retention` in your config file.")?;
    let autohide_time = is_flag_enabled!(autohide_time, matches, config);
    let default_time_value = get_default_time_value(matches, config, retention_ms)
        .context("Update 'default_time_value' in your config file.")?;

    let use_basic_mode = is_flag_enabled!(basic, matches, config);
    let expanded_upon_startup = is_flag_enabled!(expanded_on_startup, matches, config);

    // For processes
    let is_grouped = is_flag_enabled!("group", group_processes, matches, config);
    let is_case_sensitive = is_flag_enabled!(case_sensitive, matches, config);
    let is_match_whole_word = is_flag_enabled!(whole_word, matches, config);
    let is_use_regex = is_flag_enabled!(regex, matches, config);

    let mut widget_map = HashMap::new();
    let mut cpu_state_map: HashMap<u64, CpuWidgetState> = HashMap::new();
    let mut mem_state_map: HashMap<u64, MemWidgetState> = HashMap::new();
    let mut net_state_map: HashMap<u64, NetWidgetState> = HashMap::new();
    let mut proc_state_map: HashMap<u64, ProcWidgetState> = HashMap::new();
    let mut temp_state_map: HashMap<u64, TempWidgetState> = HashMap::new();
    let mut disk_state_map: HashMap<u64, DiskTableWidget> = HashMap::new();
    let mut battery_state_map: HashMap<u64, BatteryWidgetState> = HashMap::new();

    let autohide_timer = if autohide_time {
        Some(Instant::now())
    } else {
        None
    };

    let mut initial_widget_id: u64 = default_widget_id;
    let mut initial_widget_type = Proc;
    let is_custom_layout = config.row.is_some();
    let mut used_widget_set = HashSet::new();

    let show_memory_as_values = is_flag_enabled!(mem_as_value, matches, config);
    let is_default_tree = is_flag_enabled!(tree, matches, config);
    let is_default_command = is_flag_enabled!(process_command, matches, config);
    let is_advanced_kill = !(is_flag_enabled!(disable_advanced_kill, matches, config));

    let network_unit_type = get_network_unit_type(matches, config);
    let network_scale_type = get_network_scale_type(matches, config);
    let network_use_binary_prefix = is_flag_enabled!(network_use_binary_prefix, matches, config);

    let app_config_fields = AppConfigFields {
        update_rate_in_milliseconds: get_update_rate_in_milliseconds(matches, config)
            .context("Update 'rate' in your config file.")?,
        temperature_type: get_temperature(matches, config)
            .context("Update 'temperature_type' in your config file.")?,
        show_average_cpu: get_show_average_cpu(matches, config),
        use_dot: is_flag_enabled!(dot_marker, matches, config),
        left_legend: is_flag_enabled!(left_legend, matches, config),
        use_current_cpu_total: is_flag_enabled!(current_usage, matches, config),
        unnormalized_cpu: is_flag_enabled!(unnormalized_cpu, matches, config),
        use_basic_mode,
        default_time_value,
        time_interval: get_time_interval(matches, config, retention_ms)
            .context("Update 'time_delta' in your config file.")?,
        hide_time: is_flag_enabled!(hide_time, matches, config),
        autohide_time,
        use_old_network_legend: is_flag_enabled!(use_old_network_legend, matches, config),
        table_gap: u16::from(!(is_flag_enabled!(hide_table_gap, matches, config))),
        disable_click: is_flag_enabled!(disable_click, matches, config),
        enable_gpu_memory: get_enable_gpu_memory(matches, config),
        enable_cache_memory: get_enable_cache_memory(matches, config),
        show_table_scroll_position: is_flag_enabled!(show_table_scroll_position, matches, config),
        is_advanced_kill,
        network_scale_type,
        network_unit_type,
        network_use_binary_prefix,
        retention_ms,
    };

    for row in &widget_layout.rows {
        for col in &row.children {
            for col_row in &col.children {
                for widget in &col_row.children {
                    widget_map.insert(widget.widget_id, widget.clone());
                    if let Some(default_widget_type) = &default_widget_type_option {
                        if !is_custom_layout || use_basic_mode {
                            match widget.widget_type {
                                BasicCpu => {
                                    if let Cpu = *default_widget_type {
                                        initial_widget_id = widget.widget_id;
                                        initial_widget_type = Cpu;
                                    }
                                }
                                BasicMem => {
                                    if let Mem = *default_widget_type {
                                        initial_widget_id = widget.widget_id;
                                        initial_widget_type = Cpu;
                                    }
                                }
                                BasicNet => {
                                    if let Net = *default_widget_type {
                                        initial_widget_id = widget.widget_id;
                                        initial_widget_type = Cpu;
                                    }
                                }
                                _ => {
                                    if *default_widget_type == widget.widget_type {
                                        initial_widget_id = widget.widget_id;
                                        initial_widget_type = widget.widget_type.clone();
                                    }
                                }
                            }
                        }
                    }

                    used_widget_set.insert(widget.widget_type.clone());

                    match widget.widget_type {
                        Cpu => {
                            cpu_state_map.insert(
                                widget.widget_id,
                                CpuWidgetState::new(
                                    &app_config_fields,
                                    default_time_value,
                                    autohide_timer,
                                    colours,
                                ),
                            );
                        }
                        Mem => {
                            mem_state_map.insert(
                                widget.widget_id,
                                MemWidgetState::init(default_time_value, autohide_timer),
                            );
                        }
                        Net => {
                            net_state_map.insert(
                                widget.widget_id,
                                NetWidgetState::init(default_time_value, autohide_timer),
                            );
                        }
                        Proc => {
                            let mode = if is_grouped {
                                ProcWidgetMode::Grouped
                            } else if is_default_tree {
                                ProcWidgetMode::Tree {
                                    collapsed_pids: Default::default(),
                                }
                            } else {
                                ProcWidgetMode::Normal
                            };

                            proc_state_map.insert(
                                widget.widget_id,
                                ProcWidgetState::new(
                                    &app_config_fields,
                                    mode,
                                    is_case_sensitive,
                                    is_match_whole_word,
                                    is_use_regex,
                                    show_memory_as_values,
                                    is_default_command,
                                    colours,
                                ),
                            );
                        }
                        Disk => {
                            disk_state_map.insert(
                                widget.widget_id,
                                DiskTableWidget::new(&app_config_fields, colours),
                            );
                        }
                        Temp => {
                            temp_state_map.insert(
                                widget.widget_id,
                                TempWidgetState::new(&app_config_fields, colours),
                            );
                        }
                        Battery => {
                            battery_state_map
                                .insert(widget.widget_id, BatteryWidgetState::default());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let basic_table_widget_state = if use_basic_mode {
        Some(match initial_widget_type {
            Proc | Disk | Temp => BasicTableWidgetState {
                currently_displayed_widget_type: initial_widget_type,
                currently_displayed_widget_id: initial_widget_id,
                widget_id: 100,
                left_tlc: None,
                left_brc: None,
                right_tlc: None,
                right_brc: None,
            },
            _ => BasicTableWidgetState {
                currently_displayed_widget_type: Proc,
                currently_displayed_widget_id: DEFAULT_WIDGET_ID,
                widget_id: 100,
                left_tlc: None,
                left_brc: None,
                right_tlc: None,
                right_brc: None,
            },
        })
    } else {
        None
    };

    let use_mem = used_widget_set.get(&Mem).is_some() || used_widget_set.get(&BasicMem).is_some();
    let used_widgets = UsedWidgets {
        use_cpu: used_widget_set.get(&Cpu).is_some() || used_widget_set.get(&BasicCpu).is_some(),
        use_mem,
        use_cache: use_mem && get_enable_cache_memory(matches, config),
        use_gpu: use_mem && get_enable_gpu_memory(matches, config),
        use_net: used_widget_set.get(&Net).is_some() || used_widget_set.get(&BasicNet).is_some(),
        use_proc: used_widget_set.get(&Proc).is_some(),
        use_disk: used_widget_set.get(&Disk).is_some(),
        use_temp: used_widget_set.get(&Temp).is_some(),
        use_battery: used_widget_set.get(&Battery).is_some(),
    };

    let disk_filter =
        get_ignore_list(&config.disk_filter).context("Update 'disk_filter' in your config file")?;
    let mount_filter = get_ignore_list(&config.mount_filter)
        .context("Update 'mount_filter' in your config file")?;
    let temp_filter =
        get_ignore_list(&config.temp_filter).context("Update 'temp_filter' in your config file")?;
    let net_filter =
        get_ignore_list(&config.net_filter).context("Update 'net_filter' in your config file")?;

    Ok(App::builder()
        .app_config_fields(app_config_fields)
        .cpu_state(CpuState::init(cpu_state_map))
        .mem_state(MemState::init(mem_state_map))
        .net_state(NetState::init(net_state_map))
        .proc_state(ProcState::init(proc_state_map))
        .disk_state(DiskState::init(disk_state_map))
        .temp_state(TempState::init(temp_state_map))
        .battery_state(BatteryState::init(battery_state_map))
        .basic_table_widget_state(basic_table_widget_state)
        .current_widget(widget_map.get(&initial_widget_id).unwrap().clone()) // TODO: [UNWRAP] - many of the unwraps are fine (like this one) but do a once-over and/or switch to expect?
        .widget_map(widget_map)
        .used_widgets(used_widgets)
        .is_expanded(expanded_upon_startup && !use_basic_mode)
        .filters(DataFilters {
            disk_filter,
            mount_filter,
            temp_filter,
            net_filter,
        })
        .build())
}

pub fn get_widget_layout(
    matches: &ArgMatches, config: &Config,
) -> error::Result<(BottomLayout, u64, Option<BottomWidgetType>)> {
    let left_legend = is_flag_enabled!(left_legend, matches, config);

    let (default_widget_type, mut default_widget_count) =
        get_default_widget_and_count(matches, config)?;
    let mut default_widget_id = 1;

    let bottom_layout = if is_flag_enabled!(basic, matches, config) {
        default_widget_id = DEFAULT_WIDGET_ID;

        BottomLayout::init_basic_default(get_use_battery(matches, config))
    } else {
        let ref_row: Vec<Row>; // Required to handle reference
        let rows = match &config.row {
            Some(r) => r,
            None => {
                // This cannot (like it really shouldn't) fail!
                ref_row = toml_edit::de::from_str::<Config>(if get_use_battery(matches, config) {
                    DEFAULT_BATTERY_LAYOUT
                } else {
                    DEFAULT_LAYOUT
                })?
                .row
                .unwrap();
                &ref_row
            }
        };

        let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
        let mut total_height_ratio = 0;

        let mut ret_bottom_layout = BottomLayout {
            rows: rows
                .iter()
                .map(|row| {
                    row.convert_row_to_bottom_row(
                        &mut iter_id,
                        &mut total_height_ratio,
                        &mut default_widget_id,
                        &default_widget_type,
                        &mut default_widget_count,
                        left_legend,
                    )
                })
                .collect::<error::Result<Vec<_>>>()?,
            total_row_height_ratio: total_height_ratio,
        };

        // Confirm that we have at least ONE widget left - if not, error out!
        if iter_id > 0 {
            ret_bottom_layout.get_movement_mappings();
            // debug!("Bottom layout: {:#?}", ret_bottom_layout);

            ret_bottom_layout
        } else {
            return Err(error::BottomError::ConfigError(
                "please have at least one widget under the '[[row]]' section.".to_string(),
            ));
        }
    };

    Ok((bottom_layout, default_widget_id, default_widget_type))
}

fn get_update_rate_in_milliseconds(matches: &ArgMatches, config: &Config) -> error::Result<u64> {
    let update_rate_in_milliseconds = if let Some(update_rate) = matches.get_one::<String>("rate") {
        update_rate.parse::<u64>().map_err(|_| {
            BottomError::ConfigError(
                "could not parse as a valid 64-bit unsigned integer".to_string(),
            )
        })?
    } else if let Some(flags) = &config.flags {
        if let Some(rate) = flags.rate {
            rate
        } else {
            DEFAULT_REFRESH_RATE_IN_MILLISECONDS
        }
    } else {
        DEFAULT_REFRESH_RATE_IN_MILLISECONDS
    };

    if update_rate_in_milliseconds < 250 {
        return Err(BottomError::ConfigError(
            "set your update rate to be at least 250 milliseconds.".to_string(),
        ));
    }

    Ok(update_rate_in_milliseconds)
}

fn get_temperature(
    matches: &ArgMatches, config: &Config,
) -> error::Result<data_harvester::temperature::TemperatureType> {
    if matches.get_flag("fahrenheit") {
        return Ok(data_harvester::temperature::TemperatureType::Fahrenheit);
    } else if matches.get_flag("kelvin") {
        return Ok(data_harvester::temperature::TemperatureType::Kelvin);
    } else if matches.get_flag("celsius") {
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
fn get_show_average_cpu(matches: &ArgMatches, config: &Config) -> bool {
    if matches.get_flag("hide_avg_cpu") {
        return false;
    } else if let Some(flags) = &config.flags {
        if let Some(avg_cpu) = flags.hide_avg_cpu {
            return !avg_cpu;
        }
    }

    true
}

/// FIXME: Let this accept human times.
fn get_default_time_value(
    matches: &ArgMatches, config: &Config, retention_ms: u64,
) -> error::Result<u64> {
    let default_time =
        if let Some(default_time_value) = matches.get_one::<String>("default_time_value") {
            default_time_value.parse::<u64>().map_err(|_| {
                BottomError::ConfigError(
                    "could not parse as a valid 64-bit unsigned integer".to_string(),
                )
            })?
        } else if let Some(flags) = &config.flags {
            if let Some(default_time_value) = flags.default_time_value {
                default_time_value
            } else {
                DEFAULT_TIME_MILLISECONDS
            }
        } else {
            DEFAULT_TIME_MILLISECONDS
        };

    if default_time < 30000 {
        return Err(BottomError::ConfigError(
            "set your default value to be at least 30000 milliseconds.".to_string(),
        ));
    } else if default_time > retention_ms {
        return Err(BottomError::ConfigError(format!(
            "set your default value to be at most {} milliseconds.",
            retention_ms
        )));
    }

    Ok(default_time)
}

fn get_time_interval(
    matches: &ArgMatches, config: &Config, retention_ms: u64,
) -> error::Result<u64> {
    let time_interval = if let Some(time_interval) = matches.get_one::<String>("time_delta") {
        time_interval.parse::<u64>().map_err(|_| {
            BottomError::ConfigError(
                "could not parse as a valid 64-bit unsigned integer".to_string(),
            )
        })?
    } else if let Some(flags) = &config.flags {
        if let Some(time_interval) = flags.time_delta {
            time_interval
        } else {
            TIME_CHANGE_MILLISECONDS
        }
    } else {
        TIME_CHANGE_MILLISECONDS
    };

    if time_interval < 1000 {
        return Err(BottomError::ConfigError(
            "set your time delta to be at least 1000 milliseconds.".to_string(),
        ));
    } else if time_interval > retention_ms {
        return Err(BottomError::ConfigError(format!(
            "set your time delta to be at most {} milliseconds.",
            retention_ms
        )));
    }

    Ok(time_interval)
}

fn get_default_widget_and_count(
    matches: &ArgMatches, config: &Config,
) -> error::Result<(Option<BottomWidgetType>, u64)> {
    let widget_type = if let Some(widget_type) = matches.get_one::<String>("default_widget_type") {
        let parsed_widget = widget_type.parse::<BottomWidgetType>()?;
        if let BottomWidgetType::Empty = parsed_widget {
            None
        } else {
            Some(parsed_widget)
        }
    } else if let Some(flags) = &config.flags {
        if let Some(widget_type) = &flags.default_widget_type {
            let parsed_widget = widget_type.parse::<BottomWidgetType>()?;
            if let BottomWidgetType::Empty = parsed_widget {
                None
            } else {
                Some(parsed_widget)
            }
        } else {
            None
        }
    } else {
        None
    };

    let widget_count = if let Some(widget_count) = matches.get_one::<String>("default_widget_count")
    {
        Some(widget_count.parse::<u128>()?)
    } else if let Some(flags) = &config.flags {
        flags
            .default_widget_count
            .map(|widget_count| widget_count.into())
    } else {
        None
    };

    match (widget_type, widget_count) {
        (Some(widget_type), Some(widget_count)) => {
            let widget_count = widget_count.try_into().map_err(|_| BottomError::ConfigError(
                "set your widget count to be at most unsigned INT_MAX.".to_string()
            ))?;
            Ok((Some(widget_type), widget_count))
        }
        (Some(widget_type), None) => Ok((Some(widget_type), 1)),
        (None, Some(_widget_count)) =>  Err(BottomError::ConfigError(
            "cannot set 'default_widget_count' by itself, it must be used with 'default_widget_type'.".to_string(),
        )),
        (None, None) => Ok((None, 1))
    }
}

#[allow(unused_variables)]
fn get_use_battery(matches: &ArgMatches, config: &Config) -> bool {
    #[cfg(feature = "battery")]
    {
        if let Ok(battery_manager) = Manager::new() {
            if let Ok(batteries) = battery_manager.batteries() {
                if batteries.count() == 0 {
                    return false;
                }
            }
        }

        if matches.get_flag("battery") {
            return true;
        } else if let Some(flags) = &config.flags {
            if let Some(battery) = flags.battery {
                return battery;
            }
        }
    }

    false
}

#[allow(unused_variables)]
fn get_enable_gpu_memory(matches: &ArgMatches, config: &Config) -> bool {
    #[cfg(feature = "gpu")]
    {
        if matches.get_flag("enable_gpu_memory") {
            return true;
        } else if let Some(flags) = &config.flags {
            if let Some(enable_gpu_memory) = flags.enable_gpu_memory {
                return enable_gpu_memory;
            }
        }
    }

    false
}

#[allow(unused_variables)]
fn get_enable_cache_memory(matches: &ArgMatches, config: &Config) -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        if matches.get_flag("enable_cache_memory") {
            return true;
        } else if let Some(flags) = &config.flags {
            if let Some(enable_cache_memory) = flags.enable_cache_memory {
                return enable_cache_memory;
            }
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

pub fn get_color_scheme(matches: &ArgMatches, config: &Config) -> error::Result<ColourScheme> {
    if let Some(color) = matches.get_one::<String>("color") {
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

fn get_network_unit_type(matches: &ArgMatches, config: &Config) -> DataUnit {
    if matches.get_flag("network_use_bytes") {
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

fn get_network_scale_type(matches: &ArgMatches, config: &Config) -> AxisScaling {
    if matches.get_flag("network_use_log") {
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

fn get_retention_ms(matches: &ArgMatches, config: &Config) -> error::Result<u64> {
    const DEFAULT_RETENTION_MS: u64 = 600 * 1000; // Keep 10 minutes of data.

    if let Some(retention) = matches.get_one::<String>("retention") {
        humantime::parse_duration(retention)
            .map(|dur| dur.as_millis() as u64)
            .map_err(|err| BottomError::ConfigError(format!("invalid retention duration: {err:?}")))
    } else if let Some(flags) = &config.flags {
        if let Some(retention) = flags.retention {
            Ok(retention.as_millis() as u64)
        } else {
            Ok(DEFAULT_RETENTION_MS)
        }
    } else {
        Ok(DEFAULT_RETENTION_MS)
    }
}

#[cfg(test)]
mod test {

    use clap::ArgMatches;

    use super::{get_color_scheme, get_widget_layout, Config};
    use crate::{app::App, canvas::canvas_styling::CanvasColours};

    fn create_app(mut config: Config, matches: ArgMatches) -> App {
        let (layout, id, ty) = get_widget_layout(&matches, &config).unwrap();
        let colours =
            CanvasColours::new(get_color_scheme(&matches, &config).unwrap(), &config).unwrap();

        super::build_app(&matches, &mut config, &layout, id, &ty, &colours).unwrap()
    }

    // TODO: There's probably a better way to create clap options AND unify together to avoid the possibility of
    // typos/mixing up. Use proc macros to unify on one struct?
    #[test]
    fn verify_cli_options_build() {
        let app = crate::clap::build_app();

        let default_app = {
            let app = app.clone();
            let config = Config::default();
            let matches = app.get_matches_from([""]);

            create_app(config, matches)
        };

        // Skip battery since it's tricky to test depending on the platform/features we're testing with.
        let skip = ["help", "version", "celsius", "battery"];

        for arg in app.get_arguments().collect::<Vec<_>>() {
            let arg_name = arg
                .get_long_and_visible_aliases()
                .unwrap()
                .first()
                .unwrap()
                .to_owned();

            if !arg.get_action().takes_values() && !skip.contains(&arg_name) {
                let arg = format!("--{arg_name}");

                let arguments = vec!["btm", &arg];
                let app = app.clone();
                let config = Config::default();
                let matches = app.get_matches_from(arguments);

                let testing_app = create_app(config, matches);

                if (default_app.app_config_fields == testing_app.app_config_fields)
                    && default_app.is_expanded == testing_app.is_expanded
                    && default_app
                        .proc_state
                        .widget_states
                        .iter()
                        .zip(testing_app.proc_state.widget_states.iter())
                        .all(|(a, b)| (a.1.test_equality(b.1)))
                {
                    panic!("failed on {arg_name}");
                }
            }
        }
    }
}
