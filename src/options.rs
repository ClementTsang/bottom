use serde::Deserialize;

use std::time::Instant;

use crate::{
    app::{data_harvester, App, AppConfigFields, CpuState, MemState, NetState, WidgetPosition},
    constants::*,
    utils::error::{self, BottomError},
};

use layout_manager::*;

mod layout_manager;

#[derive(Default, Deserialize)]
pub struct Config {
    pub flags: Option<ConfigFlags>,
    pub colors: Option<ConfigColours>,
    pub row: Option<Vec<Row>>,
}

#[derive(Default, Deserialize)]
pub struct ConfigFlags {
    pub avg_cpu: Option<bool>,
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
    pub show_disabled_data: Option<bool>,
    pub basic: Option<bool>,
    pub default_time_value: Option<u64>,
    pub time_delta: Option<u64>,
    pub autohide_time: Option<bool>,
    pub hide_time: Option<bool>,
    //disabled_cpu_cores: Option<Vec<u64>>, // TODO: [FEATURE] Enable disabling cores in config/flags
}

#[derive(Default, Deserialize)]
pub struct ConfigColours {
    pub table_header_color: Option<String>,
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
}

pub fn build_app(matches: &clap::ArgMatches<'static>, config: &Config) -> error::Result<App> {
    let autohide_time = get_autohide_time(&matches, &config);
    let default_time_value = get_default_time_value(&matches, &config)?;
    let default_widget = get_default_widget(&matches, &config);
    let use_basic_mode = get_use_basic_mode(&matches, &config);

    get_layout(config);

    let current_widget_selected = if use_basic_mode {
        match default_widget {
            WidgetPosition::Cpu => WidgetPosition::BasicCpu,
            WidgetPosition::Network => WidgetPosition::BasicNet,
            WidgetPosition::Mem => WidgetPosition::BasicMem,
            _ => default_widget,
        }
    } else {
        default_widget
    };

    let previous_basic_table_selected = if default_widget.is_widget_table() {
        default_widget
    } else {
        WidgetPosition::Process
    };

    let app_config_fields = AppConfigFields {
        update_rate_in_milliseconds: get_update_rate_in_milliseconds(matches, config)?,
        temperature_type: get_temperature(matches, config)?,
        show_average_cpu: get_avg_cpu(matches, config),
        use_dot: get_use_dot(matches, config),
        left_legend: get_use_left_legend(matches, config),
        use_current_cpu_total: get_use_current_cpu_total(matches, config),
        show_disabled_data: get_show_disabled_data(matches, config),
        use_basic_mode,
        default_time_value,
        time_interval: get_time_interval(matches, config)?,
        hide_time: get_hide_time(matches, config),
        autohide_time,
    };

    let time_now = if autohide_time {
        Some(Instant::now())
    } else {
        None
    };

    Ok(App::builder()
        .app_config_fields(app_config_fields)
        .current_widget_selected(current_widget_selected)
        .previous_basic_table_selected(previous_basic_table_selected)
        .cpu_state(CpuState::init(default_time_value, time_now))
        .mem_state(MemState::init(default_time_value, time_now))
        .net_state(NetState::init(default_time_value, time_now))
        .build())
}

fn get_layout(config: &Config) {
    if let Some(rows) = &config.row {
        for row in rows {}
    }
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
                    "Invalid temperature type.  Please have the value be of the form \
						 <kelvin|k|celsius|c|fahrenheit|f>"
                        .to_string(),
                )),
            };
        }
    }
    Ok(data_harvester::temperature::TemperatureType::Celsius)
}

fn get_avg_cpu(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("AVG_CPU") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(avg_cpu) = flags.avg_cpu {
            return avg_cpu;
        }
    }

    false
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

fn get_show_disabled_data(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("SHOW_DISABLED_DATA") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(show_disabled_data) = flags.show_disabled_data {
            return show_disabled_data;
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

pub fn enable_app_grouping(matches: &clap::ArgMatches<'static>, config: &Config, app: &mut App) {
    if matches.is_present("GROUP_PROCESSES") {
        app.toggle_grouping();
    } else if let Some(flags) = &config.flags {
        if let Some(grouping) = flags.group_processes {
            if grouping {
                app.toggle_grouping();
            }
        }
    }
}

pub fn enable_app_case_sensitive(
    matches: &clap::ArgMatches<'static>, config: &Config, app: &mut App,
) {
    if matches.is_present("CASE_SENSITIVE") {
        app.process_search_state.search_toggle_ignore_case();
    } else if let Some(flags) = &config.flags {
        if let Some(case_sensitive) = flags.case_sensitive {
            if case_sensitive {
                app.process_search_state.search_toggle_ignore_case();
            }
        }
    }
}

pub fn enable_app_match_whole_word(
    matches: &clap::ArgMatches<'static>, config: &Config, app: &mut App,
) {
    if matches.is_present("WHOLE_WORD") {
        app.process_search_state.search_toggle_whole_word();
    } else if let Some(flags) = &config.flags {
        if let Some(whole_word) = flags.whole_word {
            if whole_word {
                app.process_search_state.search_toggle_whole_word();
            }
        }
    }
}

pub fn enable_app_use_regex(matches: &clap::ArgMatches<'static>, config: &Config, app: &mut App) {
    if matches.is_present("REGEX_DEFAULT") {
        app.process_search_state.search_toggle_regex();
    } else if let Some(flags) = &config.flags {
        if let Some(regex) = flags.regex {
            if regex {
                app.process_search_state.search_toggle_regex();
            }
        }
    }
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

fn get_default_widget(matches: &clap::ArgMatches<'static>, config: &Config) -> WidgetPosition {
    if matches.is_present("CPU_WIDGET") {
        return WidgetPosition::Cpu;
    } else if matches.is_present("MEM_WIDGET") {
        return WidgetPosition::Mem;
    } else if matches.is_present("DISK_WIDGET") {
        return WidgetPosition::Disk;
    } else if matches.is_present("TEMP_WIDGET") {
        return WidgetPosition::Temp;
    } else if matches.is_present("NET_WIDGET") {
        return WidgetPosition::Network;
    } else if matches.is_present("PROC_WIDGET") {
        return WidgetPosition::Process;
    } else if let Some(flags) = &config.flags {
        if let Some(default_widget) = &flags.default_widget {
            return match default_widget.as_str() {
                "cpu_default" => WidgetPosition::Cpu,
                "memory_default" => WidgetPosition::Mem,
                "processes_default" => WidgetPosition::Process,
                "network_default" => WidgetPosition::Network,
                "temperature_default" => WidgetPosition::Temp,
                "disk_default" => WidgetPosition::Disk,
                _ => WidgetPosition::Process,
            };
        }
    }

    WidgetPosition::Process
}
