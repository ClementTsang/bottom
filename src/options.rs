//! How to handle config files and arguments.

// TODO: Break this apart or do something a bit smarter.

pub mod args;
pub mod config;
mod error;

use std::{
    convert::TryInto,
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use config::style::ColourPalette;
pub use config::Config;
pub(crate) use error::{OptionError, OptionResult};
use hashbrown::{HashMap, HashSet};
use indexmap::IndexSet;
use regex::Regex;
#[cfg(feature = "battery")]
use starship_battery::Manager;

use self::{
    args::BottomArgs,
    config::{layout::Row, IgnoreList, StringOrNum},
};
use crate::{
    app::{filter::Filter, layout_manager::*, *},
    canvas::components::time_chart::LegendPosition,
    constants::*,
    data_collection::temperature::TemperatureType,
    utils::data_units::DataUnit,
    widgets::*,
};

macro_rules! is_flag_enabled {
    ($flag_name:ident, $arg:expr, $config:expr) => {
        if $arg.$flag_name {
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

/// The default config file sub-path.
const DEFAULT_CONFIG_FILE_LOCATION: &str = "bottom/bottom.toml";

/// Returns the config path to use. If `override_config_path` is specified, then
/// we will use that. If not, then return the "default" config path, which is:
///
/// - If a path already exists at `<HOME>/bottom/bottom.toml`, then use that for
///   legacy reasons.
/// - Otherwise, use `<SYSTEM_CONFIG_FOLDER>/bottom/bottom.toml`.
///
/// For more details on this, see [dirs](https://docs.rs/dirs/latest/dirs/fn.config_dir.html)'
/// documentation.
///
/// XXX: For macOS, we additionally will manually check `$XDG_CONFIG_HOME` as well first
/// before falling back to `dirs`.
fn get_config_path(override_config_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(conf_loc) = override_config_path {
        return Some(conf_loc.to_path_buf());
    } else if let Some(home_path) = dirs::home_dir() {
        let mut old_home_path = home_path;
        old_home_path.push(".config/");
        old_home_path.push(DEFAULT_CONFIG_FILE_LOCATION);
        if let Ok(res) = old_home_path.try_exists() {
            if res {
                // We used to create it at `<HOME>/DEFAULT_CONFIG_FILE_PATH`, but changed it
                // to be more correct later. However, for legacy reasons, if it already exists,
                // use the old one.
                return Some(old_home_path);
            }
        }
    }

    let config_path = dirs::config_dir().map(|mut path| {
        path.push(DEFAULT_CONFIG_FILE_LOCATION);
        path
    });

    if cfg!(target_os = "macos") {
        if let Ok(xdg_config_path) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg_config_path.is_empty() {
                // If XDG_CONFIG_HOME exists and is non-empty, _but_ we previously used the Library-based path
                // for a config and it exists, then use that instead for backwards-compatibility.
                if let Some(old_macos_path) = &config_path {
                    if let Ok(res) = old_macos_path.try_exists() {
                        if res {
                            return config_path;
                        }
                    }
                }

                // Otherwise, try and use the XDG_CONFIG_HOME-based path.
                let mut cfg_path = PathBuf::new();
                cfg_path.push(xdg_config_path);
                cfg_path.push(DEFAULT_CONFIG_FILE_LOCATION);

                return Some(cfg_path);
            }
        }
    }

    config_path
}

fn create_config_at_path(path: &Path) -> anyhow::Result<Config> {
    if let Some(parent_path) = path.parent() {
        fs::create_dir_all(parent_path)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(CONFIG_TEXT.as_bytes())?;

    Ok(Config::default())
}

/// Get the config at `config_path`. If there is no config file at the specified
/// path, it will try to create a new file with the default settings, and return
/// the default config.
///
/// We're going to use the following behaviour on when we'll return an error rather
/// than just "silently" continuing on:
/// - If the user passed in a path explicitly, then we will be loud and error out.
/// - If the user does NOT pass in a path explicitly, then just show a warning,
///   but continue. This is in case they do not want to write a default config file at
///   the XDG locations, for example.
pub fn get_or_create_config(config_path: Option<&Path>) -> anyhow::Result<Config> {
    let adjusted_config_path = get_config_path(config_path);

    match &adjusted_config_path {
        Some(path) => {
            if let Ok(config_string) = fs::read_to_string(path) {
                Ok(toml_edit::de::from_str(&config_string)?)
            } else {
                match create_config_at_path(path) {
                    Ok(cfg) => Ok(cfg),
                    Err(err) => {
                        if config_path.is_some() {
                            Err(err.context(format!(
                                "bottom could not create a new config file at '{}'.",
                                path.display()
                            )))
                        } else {
                            indoc::eprintdoc!(
                                "Note: bottom couldn't create a default config file at '{}', and the \
                                application has fallen back to the default configuration.
                                    
                                Caused by:
                                    {err}
                                ",
                                path.display()
                            );

                            Ok(Config::default())
                        }
                    }
                }
            }
        }
        None => {
            // If we somehow don't have any config path, then just assume the default config
            // but don't write to any file.
            //
            // TODO: For now, just print a message to stderr indicating this. In the future,
            // probably show in-app (too).

            eprintln!(
                "Note: bottom couldn't find a location to create or read a config file, so \
                the application has fallen back to the default configuration. \
                This could be for a variety of reasons, such as issues with file permissions."
            );

            Ok(Config::default())
        }
    }
}

pub(crate) fn init_app(
    args: BottomArgs, config: Config,
) -> Result<(App, BottomLayout, ColourPalette)> {
    use BottomWidgetType::*;

    // Since everything takes a reference, but we want to take ownership here to
    // drop matches/config later...
    let args = &args;
    let config = &config;

    let styling = ColourPalette::new(args, config)?;

    let (widget_layout, default_widget_id, default_widget_type_option) =
        get_widget_layout(args, config)
            .context("Found an issue while trying to build the widget layout.")?;

    let retention_ms = get_retention(args, config)?;
    let autohide_time = is_flag_enabled!(autohide_time, args.general, config);
    let default_time_value = get_default_time_value(args, config, retention_ms)?;

    let use_basic_mode = is_flag_enabled!(basic, args.general, config);
    let expanded = is_flag_enabled!(expanded, args.general, config);

    // For processes
    let is_grouped = is_flag_enabled!(group_processes, args.process, config);
    let is_case_sensitive = is_flag_enabled!(case_sensitive, args.process, config);
    let is_match_whole_word = is_flag_enabled!(whole_word, args.process, config);
    let is_use_regex = is_flag_enabled!(regex, args.process, config);
    let is_default_tree = is_flag_enabled!(tree, args.process, config);
    let is_default_command = is_flag_enabled!(process_command, args.process, config);
    let is_advanced_kill = !(is_flag_enabled!(disable_advanced_kill, args.process, config));
    let process_memory_as_value = is_flag_enabled!(process_memory_as_value, args.process, config);

    // For CPU
    let default_cpu_selection = get_default_cpu_selection(args, config);

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

    let network_unit_type = get_network_unit_type(args, config);
    let network_scale_type = get_network_scale_type(args, config);
    let network_use_binary_prefix =
        is_flag_enabled!(network_use_binary_prefix, args.network, config);

    let proc_columns: Option<IndexSet<ProcWidgetColumn>> = {
        config.processes.as_ref().and_then(|cfg| {
            if cfg.columns.is_empty() {
                None
            } else {
                Some(IndexSet::from_iter(
                    cfg.columns.iter().map(ProcWidgetColumn::from),
                ))
            }
        })
    };

    let network_legend_position = get_network_legend_position(args, config)?;
    let memory_legend_position = get_memory_legend_position(args, config)?;

    // TODO: Can probably just reuse the options struct.
    let app_config_fields = AppConfigFields {
        update_rate: get_update_rate(args, config)?,
        temperature_type: get_temperature(args, config)
            .context("Update 'temperature_type' in your config file.")?,
        show_average_cpu: get_show_average_cpu(args, config),
        use_dot: is_flag_enabled!(dot_marker, args.general, config),
        cpu_left_legend: is_flag_enabled!(cpu_left_legend, args.cpu, config),
        use_current_cpu_total: is_flag_enabled!(current_usage, args.process, config),
        unnormalized_cpu: is_flag_enabled!(unnormalized_cpu, args.process, config),
        use_basic_mode,
        default_time_value,
        time_interval: get_time_interval(args, config, retention_ms)?,
        hide_time: is_flag_enabled!(hide_time, args.general, config),
        autohide_time,
        use_old_network_legend: is_flag_enabled!(use_old_network_legend, args.network, config),
        table_gap: u16::from(!(is_flag_enabled!(hide_table_gap, args.general, config))),
        disable_click: is_flag_enabled!(disable_click, args.general, config),
        enable_gpu: get_enable_gpu(args, config),
        enable_cache_memory: get_enable_cache_memory(args, config),
        show_table_scroll_position: is_flag_enabled!(
            show_table_scroll_position,
            args.general,
            config
        ),
        is_advanced_kill,
        memory_legend_position,
        network_legend_position,
        network_scale_type,
        network_unit_type,
        network_use_binary_prefix,
        retention_ms,
        dedicated_average_row: get_dedicated_avg_row(config),
    };

    let table_config = ProcTableConfig {
        is_case_sensitive,
        is_match_whole_word,
        is_use_regex,
        show_memory_as_values: process_memory_as_value,
        is_command: is_default_command,
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
                                    default_cpu_selection,
                                    default_time_value,
                                    autohide_timer,
                                    &styling,
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
                                    table_config,
                                    &styling,
                                    &proc_columns,
                                ),
                            );
                        }
                        Disk => {
                            disk_state_map.insert(
                                widget.widget_id,
                                DiskTableWidget::new(&app_config_fields, &styling),
                            );
                        }
                        Temp => {
                            temp_state_map.insert(
                                widget.widget_id,
                                TempWidgetState::new(&app_config_fields, &styling),
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
        use_cache: use_mem && get_enable_cache_memory(args, config),
        use_gpu: get_enable_gpu(args, config),
        use_net: used_widget_set.get(&Net).is_some() || used_widget_set.get(&BasicNet).is_some(),
        use_proc: used_widget_set.get(&Proc).is_some(),
        use_disk: used_widget_set.get(&Disk).is_some(),
        use_temp: used_widget_set.get(&Temp).is_some(),
        use_battery: used_widget_set.get(&Battery).is_some(),
    };

    let (disk_name_filter, disk_mount_filter) = {
        match &config.disk {
            Some(cfg) => {
                let df = get_ignore_list(&cfg.name_filter)
                    .context("Update 'disk.name_filter' in your config file")?;
                let mf = get_ignore_list(&cfg.mount_filter)
                    .context("Update 'disk.mount_filter' in your config file")?;

                (df, mf)
            }
            None => (None, None),
        }
    };
    let temp_sensor_filter = match &config.temperature {
        Some(cfg) => get_ignore_list(&cfg.sensor_filter)
            .context("Update 'temperature.sensor_filter' in your config file")?,
        None => None,
    };
    let net_interface_filter = match &config.network {
        Some(cfg) => get_ignore_list(&cfg.interface_filter)
            .context("Update 'network.interface_filter' in your config file")?,
        None => None,
    };

    let states = AppWidgetStates {
        cpu_state: CpuState::init(cpu_state_map),
        mem_state: MemState::init(mem_state_map),
        net_state: NetState::init(net_state_map),
        proc_state: ProcState::init(proc_state_map),
        temp_state: TempState::init(temp_state_map),
        disk_state: DiskState::init(disk_state_map),
        battery_state: BatteryState::init(battery_state_map),
        basic_table_widget_state,
    };

    let current_widget = widget_map.get(&initial_widget_id).unwrap().clone();
    let filters = DataFilters {
        disk_filter: disk_name_filter,
        mount_filter: disk_mount_filter,
        temp_filter: temp_sensor_filter,
        net_filter: net_interface_filter,
    };
    let is_expanded = expanded && !use_basic_mode;

    Ok((
        App::new(
            app_config_fields,
            states,
            widget_map,
            current_widget,
            used_widgets,
            filters,
            is_expanded,
        ),
        widget_layout,
        styling,
    ))
}

pub fn get_widget_layout(
    args: &BottomArgs, config: &Config,
) -> OptionResult<(BottomLayout, u64, Option<BottomWidgetType>)> {
    let cpu_left_legend = is_flag_enabled!(cpu_left_legend, args.cpu, config);

    let (default_widget_type, mut default_widget_count) =
        get_default_widget_and_count(args, config)?;
    let mut default_widget_id = 1;

    let bottom_layout = if is_flag_enabled!(basic, args.general, config) {
        default_widget_id = DEFAULT_WIDGET_ID;

        BottomLayout::init_basic_default(get_use_battery(args, config))
    } else {
        let ref_row: Vec<Row>; // Required to handle reference
        let rows = match &config.row {
            Some(r) => r,
            None => {
                // This cannot (like it really shouldn't) fail!
                ref_row = toml_edit::de::from_str::<Config>(if get_use_battery(args, config) {
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
                        cpu_left_legend,
                    )
                    .map_err(|err| OptionError::config(err.to_string()))
                })
                .collect::<OptionResult<Vec<_>>>()?,
            total_row_height_ratio: total_height_ratio,
        };

        // Confirm that we have at least ONE widget left - if not, error out!
        if iter_id > 0 {
            ret_bottom_layout.get_movement_mappings();
            ret_bottom_layout
        } else {
            return Err(OptionError::config(
                "have at least one widget under the '[[row]]' section.",
            ));
        }
    };

    Ok((bottom_layout, default_widget_id, default_widget_type))
}

#[inline]
fn try_parse_ms(s: &str) -> Result<u64, ()> {
    Ok(if let Ok(val) = humantime::parse_duration(s) {
        val.as_millis().try_into().map_err(|_| ())?
    } else if let Ok(val) = s.parse::<u64>() {
        val
    } else {
        return Err(());
    })
}

macro_rules! parse_arg_value {
    ($to_try:expr, $flag:literal) => {
        $to_try.map_err(|_| OptionError::invalid_arg_value($flag))
    };
}

macro_rules! parse_config_value {
    ($to_try:expr, $setting:literal) => {
        $to_try.map_err(|_| OptionError::invalid_config_value($setting))
    };
}

macro_rules! parse_ms_option {
    ($arg_expr:expr, $config_expr:expr, $default_value:expr, $setting:literal, $low:expr, $high:expr $(,)?) => {{
        use humantime::format_duration;

        if let Some(to_parse) = $arg_expr {
            let value = parse_arg_value!(try_parse_ms(to_parse), $setting)?;

            if let Some(limit) = $low {
                if value < limit {
                    return Err(OptionError::arg(format!(
                        "'--{}' must be greater than {}",
                        $setting,
                        format_duration(Duration::from_millis(limit))
                    )));
                }
            }

            if let Some(limit) = $high {
                if value > limit {
                    return Err(OptionError::arg(format!(
                        "'--{}' must be less than {}",
                        $setting,
                        format_duration(Duration::from_millis(limit))
                    )));
                }
            }

            Ok(value)
        } else if let Some(to_parse) = $config_expr {
            let value = match to_parse {
                StringOrNum::String(s) => parse_config_value!(try_parse_ms(s), $setting)?,
                StringOrNum::Num(n) => *n,
            };

            if let Some(limit) = $low {
                if value < limit {
                    return Err(OptionError::arg(format!(
                        "'{}' must be greater than {}",
                        $setting,
                        format_duration(Duration::from_millis(limit))
                    )));
                }
            }

            if let Some(limit) = $high {
                if value > limit {
                    return Err(OptionError::arg(format!(
                        "'{}' must be less than {}",
                        $setting,
                        format_duration(Duration::from_millis(limit))
                    )));
                }
            }

            Ok(value)
        } else {
            Ok($default_value)
        }
    }};
}

#[inline]
fn get_update_rate(args: &BottomArgs, config: &Config) -> OptionResult<u64> {
    parse_ms_option!(
        &args.general.rate,
        config.flags.as_ref().and_then(|flags| flags.rate.as_ref()),
        DEFAULT_REFRESH_RATE_IN_MILLISECONDS,
        "rate",
        Some(250),
        None,
    )
}

fn get_temperature(args: &BottomArgs, config: &Config) -> OptionResult<TemperatureType> {
    if args.temperature.fahrenheit {
        return Ok(TemperatureType::Fahrenheit);
    } else if args.temperature.kelvin {
        return Ok(TemperatureType::Kelvin);
    } else if args.temperature.celsius {
        return Ok(TemperatureType::Celsius);
    } else if let Some(flags) = &config.flags {
        if let Some(temp_type) = &flags.temperature_type {
            return parse_config_value!(TemperatureType::from_str(temp_type), "temperature_type");
        }
    }
    Ok(TemperatureType::Celsius)
}

/// Yes, this function gets whether to show average CPU (true) or not (false).
fn get_show_average_cpu(args: &BottomArgs, config: &Config) -> bool {
    if args.cpu.hide_avg_cpu {
        return false;
    } else if let Some(flags) = &config.flags {
        if let Some(avg_cpu) = flags.hide_avg_cpu {
            return !avg_cpu;
        }
    }

    true
}

// I hate this too.
fn get_default_cpu_selection(args: &BottomArgs, config: &Config) -> config::cpu::CpuDefault {
    match &args.cpu.default_cpu_entry {
        Some(default) => match default {
            args::CpuDefault::All => config::cpu::CpuDefault::All,
            args::CpuDefault::Average => config::cpu::CpuDefault::Average,
        },
        None => config.cpu.as_ref().map(|c| c.default).unwrap_or_default(),
    }
}

fn get_dedicated_avg_row(config: &Config) -> bool {
    let conf = config
        .flags
        .as_ref()
        .and_then(|flags| flags.average_cpu_row)
        .unwrap_or(false);

    conf
}

#[inline]
fn get_default_time_value(
    args: &BottomArgs, config: &Config, retention_ms: u64,
) -> OptionResult<u64> {
    parse_ms_option!(
        &args.general.default_time_value,
        config
            .flags
            .as_ref()
            .and_then(|flags| flags.default_time_value.as_ref()),
        DEFAULT_TIME_MILLISECONDS,
        "default_time_value",
        Some(30000),
        Some(retention_ms),
    )
}

#[inline]
fn get_time_interval(args: &BottomArgs, config: &Config, retention_ms: u64) -> OptionResult<u64> {
    parse_ms_option!(
        &args.general.time_delta,
        config
            .flags
            .as_ref()
            .and_then(|flags| flags.time_delta.as_ref()),
        TIME_CHANGE_MILLISECONDS,
        "time_delta",
        Some(1000),
        Some(retention_ms),
    )
}

fn get_default_widget_and_count(
    args: &BottomArgs, config: &Config,
) -> OptionResult<(Option<BottomWidgetType>, u64)> {
    let widget_type = if let Some(widget_type) = &args.general.default_widget_type {
        let parsed_widget = parse_arg_value!(widget_type.parse(), "default_widget_type")?;
        if let BottomWidgetType::Empty = parsed_widget {
            None
        } else {
            Some(parsed_widget)
        }
    } else if let Some(flags) = &config.flags {
        if let Some(widget_type) = &flags.default_widget_type {
            let parsed_widget = parse_config_value!(widget_type.parse(), "default_widget_type")?;
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

    let widget_count: Option<u128> = if let Some(widget_count) = args.general.default_widget_count {
        Some(widget_count.into())
    } else {
        config.flags.as_ref().and_then(|flags| {
            flags
                .default_widget_count
                .map(|widget_count| widget_count.into())
        })
    };

    match (widget_type, widget_count) {
        (Some(widget_type), Some(widget_count)) => {
            let widget_count = widget_count.try_into().map_err(|_| OptionError::other(
                "set your widget count to be at most 18446744073709551615.".to_string()
            ))?;
            Ok((Some(widget_type), widget_count))
        }
        (Some(widget_type), None) => Ok((Some(widget_type), 1)),
        (None, Some(_widget_count)) =>  Err(OptionError::other(
            "cannot set 'default_widget_count' by itself, it must be used with 'default_widget_type'.".to_string(),
        )),
        (None, None) => Ok((None, 1))
    }
}

#[allow(unused_variables)]
fn get_use_battery(args: &BottomArgs, config: &Config) -> bool {
    #[cfg(feature = "battery")]
    {
        // TODO: Move this so it's dynamic in the app itself and automatically hide if
        // there are no batteries?
        if let Ok(battery_manager) = Manager::new() {
            if let Ok(batteries) = battery_manager.batteries() {
                if batteries.count() == 0 {
                    return false;
                }
            }
        }

        if args.battery.battery {
            return true;
        } else if let Some(flags) = &config.flags {
            if let Some(battery) = flags.battery {
                return battery;
            }
        }
    }

    false
}

#[cfg(feature = "gpu")]
fn get_enable_gpu(args: &BottomArgs, config: &Config) -> bool {
    if args.gpu.disable_gpu {
        return false;
    }

    !config
        .flags
        .as_ref()
        .and_then(|f| f.disable_gpu)
        .unwrap_or(false)
}

#[cfg(not(feature = "gpu"))]
fn get_enable_gpu(_: &BottomArgs, _: &Config) -> bool {
    false
}

#[allow(unused_variables)]
fn get_enable_cache_memory(args: &BottomArgs, config: &Config) -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        if args.memory.enable_cache_memory {
            return true;
        } else if let Some(flags) = &config.flags {
            if let Some(enable_cache_memory) = flags.enable_cache_memory {
                return enable_cache_memory;
            }
        }
    }

    false
}

fn get_ignore_list(ignore_list: &Option<IgnoreList>) -> OptionResult<Option<Filter>> {
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

        let list = list.map_err(|err| OptionError::config(err.to_string()))?;

        Ok(Some(Filter::new(ignore_list.is_list_ignored, list)))
    } else {
        Ok(None)
    }
}

fn get_network_unit_type(args: &BottomArgs, config: &Config) -> DataUnit {
    if args.network.network_use_bytes {
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

fn get_network_scale_type(args: &BottomArgs, config: &Config) -> AxisScaling {
    if args.network.network_use_log {
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

fn get_retention(args: &BottomArgs, config: &Config) -> OptionResult<u64> {
    const DEFAULT_RETENTION_MS: u64 = 600 * 1000; // Keep 10 minutes of data.

    parse_ms_option!(
        &args.general.retention,
        config
            .flags
            .as_ref()
            .and_then(|flags| flags.retention.as_ref()),
        DEFAULT_RETENTION_MS,
        "retention",
        None,
        None,
    )
}

fn get_network_legend_position(
    args: &BottomArgs, config: &Config,
) -> OptionResult<Option<LegendPosition>> {
    let result = if let Some(s) = &args.network.network_legend {
        match s.to_ascii_lowercase().trim() {
            "none" => None,
            position => Some(parse_config_value!(position.parse(), "network_legend")?),
        }
    } else if let Some(flags) = &config.flags {
        if let Some(legend) = &flags.network_legend {
            Some(parse_arg_value!(legend.parse(), "network_legend")?)
        } else {
            Some(LegendPosition::default())
        }
    } else {
        Some(LegendPosition::default())
    };

    Ok(result)
}

fn get_memory_legend_position(
    args: &BottomArgs, config: &Config,
) -> OptionResult<Option<LegendPosition>> {
    let result = if let Some(s) = &args.memory.memory_legend {
        match s.to_ascii_lowercase().trim() {
            "none" => None,
            position => Some(parse_config_value!(position.parse(), "memory_legend")?),
        }
    } else if let Some(flags) = &config.flags {
        if let Some(legend) = &flags.memory_legend {
            Some(parse_arg_value!(legend.parse(), "memory_legend")?)
        } else {
            Some(LegendPosition::default())
        }
    } else {
        Some(LegendPosition::default())
    };

    Ok(result)
}

#[cfg(test)]
mod test {
    use clap::Parser;

    use super::{get_time_interval, Config};
    use crate::{
        app::App,
        args::BottomArgs,
        options::{
            config::flags::FlagConfig, get_default_time_value, get_retention, get_update_rate,
            try_parse_ms,
        },
    };

    #[test]
    fn verify_try_parse_ms() {
        let a = "100s";
        let b = "100";
        let c = "1 min";
        let d = "1 hour 1 min";

        assert_eq!(try_parse_ms(a), Ok(100 * 1000));
        assert_eq!(try_parse_ms(b), Ok(100));
        assert_eq!(try_parse_ms(c), Ok(60 * 1000));
        assert_eq!(try_parse_ms(d), Ok(3660 * 1000));

        let a_bad = "1 test";
        let b_bad = "-100";

        assert!(try_parse_ms(a_bad).is_err());
        assert!(try_parse_ms(b_bad).is_err());
    }

    #[test]
    fn matches_human_times() {
        let config = Config::default();

        {
            let delta_args = vec!["btm", "--time_delta", "2 min"];
            let args = BottomArgs::parse_from(delta_args);

            assert_eq!(
                get_time_interval(&args, &config, 60 * 60 * 1000),
                Ok(2 * 60 * 1000)
            );
        }

        {
            let default_time_args = vec!["btm", "--default_time_value", "300s"];
            let args = BottomArgs::parse_from(default_time_args);

            assert_eq!(
                get_default_time_value(&args, &config, 60 * 60 * 1000),
                Ok(5 * 60 * 1000)
            );
        }
    }

    #[test]
    fn matches_number_times() {
        let config = Config::default();

        {
            let delta_args = vec!["btm", "--time_delta", "120000"];
            let args = BottomArgs::parse_from(delta_args);

            assert_eq!(
                get_time_interval(&args, &config, 60 * 60 * 1000),
                Ok(2 * 60 * 1000)
            );
        }

        {
            let default_time_args = vec!["btm", "--default_time_value", "300000"];
            let args = BottomArgs::parse_from(default_time_args);

            assert_eq!(
                get_default_time_value(&args, &config, 60 * 60 * 1000),
                Ok(5 * 60 * 1000)
            );
        }
    }

    #[test]
    fn config_human_times() {
        let args = BottomArgs::parse_from(["btm"]);

        let mut config = Config::default();
        let flags = FlagConfig {
            time_delta: Some("2 min".to_string().into()),
            default_time_value: Some("300s".to_string().into()),
            rate: Some("1s".to_string().into()),
            retention: Some("10m".to_string().into()),
            ..Default::default()
        };

        config.flags = Some(flags);

        assert_eq!(
            get_time_interval(&args, &config, 60 * 60 * 1000),
            Ok(2 * 60 * 1000)
        );

        assert_eq!(
            get_default_time_value(&args, &config, 60 * 60 * 1000),
            Ok(5 * 60 * 1000)
        );

        assert_eq!(get_update_rate(&args, &config), Ok(1000));

        assert_eq!(get_retention(&args, &config), Ok(600000));
    }

    #[test]
    fn config_number_times_as_string() {
        let args = BottomArgs::parse_from(["btm"]);

        let mut config = Config::default();
        let flags = FlagConfig {
            time_delta: Some("120000".to_string().into()),
            default_time_value: Some("300000".to_string().into()),
            rate: Some("1000".to_string().into()),
            retention: Some("600000".to_string().into()),
            ..Default::default()
        };

        config.flags = Some(flags);

        assert_eq!(
            get_time_interval(&args, &config, 60 * 60 * 1000),
            Ok(2 * 60 * 1000)
        );

        assert_eq!(
            get_default_time_value(&args, &config, 60 * 60 * 1000),
            Ok(5 * 60 * 1000)
        );

        assert_eq!(get_update_rate(&args, &config), Ok(1000));

        assert_eq!(get_retention(&args, &config), Ok(600000));
    }

    #[test]
    fn config_number_times_as_num() {
        let args = BottomArgs::parse_from(["btm"]);

        let mut config = Config::default();
        let flags = FlagConfig {
            time_delta: Some(120000.into()),
            default_time_value: Some(300000.into()),
            rate: Some(1000.into()),
            retention: Some(600000.into()),
            ..Default::default()
        };

        config.flags = Some(flags);

        assert_eq!(
            get_time_interval(&args, &config, 60 * 60 * 1000),
            Ok(2 * 60 * 1000)
        );

        assert_eq!(
            get_default_time_value(&args, &config, 60 * 60 * 1000),
            Ok(5 * 60 * 1000)
        );

        assert_eq!(get_update_rate(&args, &config), Ok(1000));

        assert_eq!(get_retention(&args, &config), Ok(600000));
    }

    fn create_app(args: BottomArgs) -> App {
        let config = Config::default();
        super::init_app(args, config).unwrap().0
    }

    // TODO: There's probably a better way to create clap options AND unify together
    // to avoid the possibility of typos/mixing up. Use proc macros to unify on
    // one struct?
    #[test]
    fn verify_cli_options_build() {
        let app = crate::args::build_cmd();

        let default_app = create_app(BottomArgs::parse_from(["btm"]));

        // Skip battery since it's tricky to test depending on the platform/features
        // we're testing with.
        let skip = ["help", "version", "celsius", "battery", "generate_schema"];

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
                let args = BottomArgs::parse_from(arguments);
                let testing_app = create_app(args);

                if (default_app.app_config_fields == testing_app.app_config_fields)
                    && default_app.is_expanded == testing_app.is_expanded
                    && default_app
                        .states
                        .proc_state
                        .widget_states
                        .iter()
                        .zip(testing_app.states.proc_state.widget_states.iter())
                        .all(|(a, b)| (a.1.test_equality(b.1)))
                {
                    panic!("failed on {arg_name}");
                }
            }
        }
    }

    /// This one has slightly more complex behaviour due to `dirs` not respecting XDG on macOS, so we manually
    /// handle it. However, to ensure backwards-compatibility, we also have to do some special cases.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_config_path_macos() {
        use super::get_config_path;
        use super::DEFAULT_CONFIG_FILE_LOCATION;
        use std::path::PathBuf;

        // Case three: no previous config, no XDG var.
        // SAFETY: this is the only test that does this
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }

        let case_1 = dirs::config_dir()
            .map(|mut path| {
                path.push(DEFAULT_CONFIG_FILE_LOCATION);
                path
            })
            .unwrap();

        // Skip this test if the file already exists.
        if !case_1.exists() {
            assert_eq!(get_config_path(None), Some(case_1));
        }

        // Case two: no previous config, XDG var exists.
        std::env::set_var("XDG_CONFIG_HOME", "/tmp");
        let mut case_2 = PathBuf::new();
        case_2.push("/tmp");
        case_2.push(DEFAULT_CONFIG_FILE_LOCATION);

        // Skip this test if the file already exists.
        if !case_2.exists() {
            assert_eq!(get_config_path(None), Some(case_2));
        }

        // Case one: old non-XDG exists already, XDG var exists.
        // let case_3 = case_1;
        // assert_eq!(get_config_path(None), Some(case_1));
    }
}
