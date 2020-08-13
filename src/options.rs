use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::{
    app::{layout_manager::*, *},
    constants::*,
    utils::error::{self, BottomError},
};

use layout_options::*;

mod layout_options;

#[derive(Default, Deserialize)]
pub struct Config {
    pub flags: Option<ConfigFlags>,
    pub colors: Option<ConfigColours>,
    pub row: Option<Vec<Row>>,
}

#[derive(Default, Deserialize)]
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
    pub default_widget: Option<String>,
    pub basic: Option<bool>,
    pub default_time_value: Option<u64>,
    pub time_delta: Option<u64>,
    pub autohide_time: Option<bool>,
    pub hide_time: Option<bool>,
    pub default_widget_type: Option<String>,
    pub default_widget_count: Option<u64>,
    pub use_old_network_legend: Option<bool>,
    pub hide_table_gap: Option<bool>,
    pub battery: Option<bool>,
}

#[derive(Default, Deserialize)]
pub struct ConfigColours {
    pub table_header_color: Option<String>,
    pub all_cpu_color: Option<String>,
    pub avg_cpu_color: Option<String>,
    pub cpu_core_colors: Option<Vec<String>>,
    pub ram_color: Option<String>,
    pub swap_color: Option<String>,
    pub rx_color: Option<String>,
    pub tx_color: Option<String>,
    pub rx_total_color: Option<String>,
    pub tx_total_color: Option<String>,
    pub border_color: Option<String>,
    pub highlighted_border_color: Option<String>,
    pub text_color: Option<String>,
    pub selected_text_color: Option<String>,
    pub selected_bg_color: Option<String>,
    pub widget_title_color: Option<String>,
    pub graph_color: Option<String>,
    pub battery_colors: Option<Vec<String>>,
}

pub fn build_app(
    matches: &clap::ArgMatches<'static>, config: &Config, widget_layout: &BottomLayout,
    default_widget_id: u64,
) -> error::Result<App> {
    use BottomWidgetType::*;
    let autohide_time = get_autohide_time(&matches, &config);
    let default_time_value = get_default_time_value(&matches, &config)?;
    let use_basic_mode = get_use_basic_mode(&matches, &config);

    // For processes
    let is_grouped = get_app_grouping(matches, config);
    let is_case_sensitive = get_app_case_sensitive(matches, config);
    let is_match_whole_word = get_app_match_whole_word(matches, config);
    let is_use_regex = get_app_use_regex(matches, config);

    let mut widget_map = HashMap::new();
    let mut cpu_state_map: HashMap<u64, CpuWidgetState> = HashMap::new();
    let mut mem_state_map: HashMap<u64, MemWidgetState> = HashMap::new();
    let mut net_state_map: HashMap<u64, NetWidgetState> = HashMap::new();
    let mut proc_state_map: HashMap<u64, ProcWidgetState> = HashMap::new();
    let mut temp_state_map: HashMap<u64, TempWidgetState> = HashMap::new();
    let mut disk_state_map: HashMap<u64, DiskWidgetState> = HashMap::new();
    let mut battery_state_map: HashMap<u64, BatteryWidgetState> = HashMap::new();

    let autohide_timer = if autohide_time {
        Some(Instant::now())
    } else {
        None
    };

    let (default_widget_type_option, _) = get_default_widget_and_count(matches, config)?;
    let mut initial_widget_id: u64 = default_widget_id;
    let mut initial_widget_type = Proc;
    let is_custom_layout = config.row.is_some();
    let mut used_widget_set = HashSet::new();

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
                                CpuWidgetState::init(default_time_value, autohide_timer),
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
                            proc_state_map.insert(
                                widget.widget_id,
                                ProcWidgetState::init(
                                    is_case_sensitive,
                                    is_match_whole_word,
                                    is_use_regex,
                                    is_grouped,
                                ),
                            );
                        }
                        Disk => {
                            disk_state_map.insert(widget.widget_id, DiskWidgetState::init());
                        }
                        Temp => {
                            temp_state_map.insert(widget.widget_id, TempWidgetState::init());
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
            },
            _ => BasicTableWidgetState {
                currently_displayed_widget_type: Proc,
                currently_displayed_widget_id: DEFAULT_WIDGET_ID,
                widget_id: 100,
            },
        })
    } else {
        None
    };

    let app_config_fields = AppConfigFields {
        update_rate_in_milliseconds: get_update_rate_in_milliseconds(matches, config)?,
        temperature_type: get_temperature(matches, config)?,
        show_average_cpu: get_show_average_cpu(matches, config),
        use_dot: get_use_dot(matches, config),
        left_legend: get_use_left_legend(matches, config),
        use_current_cpu_total: get_use_current_cpu_total(matches, config),
        use_basic_mode,
        default_time_value,
        time_interval: get_time_interval(matches, config)?,
        hide_time: get_hide_time(matches, config),
        autohide_time,
        use_old_network_legend: get_use_old_network_legend(matches, config),
        table_gap: if get_hide_table_gap(matches, config) {
            0
        } else {
            1
        },
    };

    let used_widgets = UsedWidgets {
        use_cpu: used_widget_set.get(&Cpu).is_some() || used_widget_set.get(&BasicCpu).is_some(),
        use_mem: used_widget_set.get(&Mem).is_some() || used_widget_set.get(&BasicMem).is_some(),
        use_net: used_widget_set.get(&Net).is_some() || used_widget_set.get(&BasicNet).is_some(),
        use_proc: used_widget_set.get(&Proc).is_some(),
        use_disk: used_widget_set.get(&Disk).is_some(),
        use_temp: used_widget_set.get(&Temp).is_some(),
        use_battery: used_widget_set.get(&Battery).is_some(),
    };

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
        .current_widget(widget_map.get(&initial_widget_id).unwrap().clone()) // I think the unwrap is fine here
        .widget_map(widget_map)
        .used_widgets(used_widgets)
        .build())
}

pub fn get_widget_layout(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<(BottomLayout, u64)> {
    let left_legend = get_use_left_legend(matches, config);
    let (default_widget_type, mut default_widget_count) =
        get_default_widget_and_count(matches, config)?;
    let mut default_widget_id = 1;

    let bottom_layout = if get_use_basic_mode(matches, config) {
        default_widget_id = DEFAULT_WIDGET_ID;
        BottomLayout::init_basic_default(get_use_battery(matches, config))
    } else {
        let ref_row: Vec<Row>; // Required to handle reference
        let rows = match &config.row {
            Some(r) => r,
            None => {
                // This cannot (like it really shouldn't) fail!
                ref_row = toml::from_str::<Config>(DEFAULT_LAYOUT)?.row.unwrap();
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

        // Confirm that we have at least ONE widget - if not, error out!
        if iter_id > 0 {
            ret_bottom_layout.get_movement_mappings();
            ret_bottom_layout
        } else {
            return Err(error::BottomError::ConfigError(
                "invalid layout config: please have at least one widget.".to_string(),
            ));
        }
    };

    Ok((bottom_layout, default_widget_id))
}

fn get_update_rate_in_milliseconds(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<u64> {
    let update_rate_in_milliseconds = if let Some(update_rate) = matches.value_of("RATE_MILLIS") {
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
        return Err(BottomError::InvalidArg(
            "Please set your update rate to be at least 250 milliseconds.".to_string(),
        ));
    } else if update_rate_in_milliseconds as u128 > std::u64::MAX as u128 {
        return Err(BottomError::InvalidArg(
            "Please set your update rate to be at most unsigned INT_MAX.".to_string(),
        ));
    }

    Ok(update_rate_in_milliseconds as u64)
}

fn get_temperature(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<data_harvester::temperature::TemperatureType> {
    if matches.is_present("FAHRENHEIT") {
        return Ok(data_harvester::temperature::TemperatureType::Fahrenheit);
    } else if matches.is_present("KELVIN") {
        return Ok(data_harvester::temperature::TemperatureType::Kelvin);
    } else if matches.is_present("CELSIUS") {
        return Ok(data_harvester::temperature::TemperatureType::Celsius);
    } else if let Some(flags) = &config.flags {
        if let Some(temp_type) = &flags.temperature_type {
            // Give lowest priority to config.
            return match temp_type.as_str() {
                "fahrenheit" | "f" => Ok(data_harvester::temperature::TemperatureType::Fahrenheit),
                "kelvin" | "k" => Ok(data_harvester::temperature::TemperatureType::Kelvin),
                "celsius" | "c" => Ok(data_harvester::temperature::TemperatureType::Celsius),
                _ => Err(BottomError::ConfigError(
                    "invalid temperature type: please have the value be of the form \
						 <kelvin|k|celsius|c|fahrenheit|f>"
                        .to_string(),
                )),
            };
        }
    }
    Ok(data_harvester::temperature::TemperatureType::Celsius)
}

/// Yes, this function gets whether to show average CPU (true) or not (false)
fn get_show_average_cpu(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("HIDE_AVG_CPU") {
        return false;
    } else if let Some(flags) = &config.flags {
        if let Some(avg_cpu) = flags.hide_avg_cpu {
            return avg_cpu;
        }
    }

    true
}

fn get_use_dot(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("DOT_MARKER") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(dot_marker) = flags.dot_marker {
            return dot_marker;
        }
    }
    false
}

fn get_use_left_legend(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("LEFT_LEGEND") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(left_legend) = flags.left_legend {
            return left_legend;
        }
    }

    false
}

fn get_use_current_cpu_total(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("USE_CURR_USAGE") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(current_usage) = flags.current_usage {
            return current_usage;
        }
    }

    false
}

fn get_use_basic_mode(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("BASIC_MODE") {
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
    let default_time = if let Some(default_time_value) = matches.value_of("DEFAULT_TIME_VALUE") {
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
        return Err(BottomError::InvalidArg(
            "Please set your default value to be at least 30000 milliseconds.".to_string(),
        ));
    } else if default_time as u128 > STALE_MAX_MILLISECONDS as u128 {
        return Err(BottomError::InvalidArg(format!(
            "Please set your default value to be at most {} milliseconds.",
            STALE_MAX_MILLISECONDS
        )));
    }

    Ok(default_time as u64)
}

fn get_time_interval(matches: &clap::ArgMatches<'static>, config: &Config) -> error::Result<u64> {
    let time_interval = if let Some(time_interval) = matches.value_of("TIME_DELTA") {
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
        return Err(BottomError::InvalidArg(
            "Please set your time delta to be at least 1000 milliseconds.".to_string(),
        ));
    } else if time_interval > STALE_MAX_MILLISECONDS as u128 {
        return Err(BottomError::InvalidArg(format!(
            "Please set your time delta to be at most {} milliseconds.",
            STALE_MAX_MILLISECONDS
        )));
    }

    Ok(time_interval as u64)
}

pub fn get_app_grouping(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("GROUP_PROCESSES") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(grouping) = flags.group_processes {
            if grouping {
                return true;
            }
        }
    }
    false
}

pub fn get_app_case_sensitive(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("CASE_SENSITIVE") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(case_sensitive) = flags.case_sensitive {
            if case_sensitive {
                return true;
            }
        }
    }
    false
}

pub fn get_app_match_whole_word(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("WHOLE_WORD") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(whole_word) = flags.whole_word {
            if whole_word {
                return true;
            }
        }
    }
    false
}

pub fn get_app_use_regex(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("REGEX_DEFAULT") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(regex) = flags.regex {
            if regex {
                return true;
            }
        }
    }
    false
}

fn get_hide_time(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("HIDE_TIME") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(hide_time) = flags.hide_time {
            if hide_time {
                return true;
            }
        }
    }
    false
}

fn get_autohide_time(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("AUTOHIDE_TIME") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(autohide_time) = flags.autohide_time {
            if autohide_time {
                return true;
            }
        }
    }

    false
}

fn get_default_widget_and_count(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> error::Result<(Option<BottomWidgetType>, u64)> {
    let widget_type = if let Some(widget_type) = matches.value_of("DEFAULT_WIDGET_TYPE") {
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

    if widget_type.is_some() {
        let widget_count = if let Some(widget_count) = matches.value_of("DEFAULT_WIDGET_COUNT") {
            widget_count.parse::<u128>()?
        } else if let Some(flags) = &config.flags {
            if let Some(widget_count) = flags.default_widget_count {
                widget_count as u128
            } else {
                1 as u128
            }
        } else {
            1 as u128
        };

        if widget_count > std::u64::MAX as u128 {
            Err(BottomError::InvalidArg(
                "Please set your widget count to be at most unsigned INT_MAX.".to_string(),
            ))
        } else {
            Ok((widget_type, widget_count as u64))
        }
    } else {
        Ok((None, 1))
    }
}

pub fn get_use_old_network_legend(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("USE_OLD_NETWORK_LEGEND") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(use_old_network_legend) = flags.use_old_network_legend {
            if use_old_network_legend {
                return true;
            }
        }
    }
    false
}

pub fn get_hide_table_gap(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("HIDE_TABLE_GAP") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(hide_table_gap) = flags.hide_table_gap {
            if hide_table_gap {
                return true;
            }
        }
    }
    false
}

pub fn get_use_battery(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("BATTERY") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(battery) = flags.battery {
            if battery {
                return true;
            }
        }
    }
    false
}
