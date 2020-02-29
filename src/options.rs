use serde::Deserialize;

use crate::{
    app::{data_harvester, App, WidgetPosition},
    constants::*,
    utils::error::{self, BottomError},
};

#[derive(Default, Deserialize)]
pub struct Config {
    pub flags: Option<ConfigFlags>,
    pub colors: Option<ConfigColours>,
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
    pub basic_mode: Option<bool>,
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

pub fn get_update_rate_in_milliseconds(
    update_rate: &Option<&str>, config: &Config,
) -> error::Result<u128> {
    let update_rate_in_milliseconds = if let Some(update_rate) = update_rate {
        update_rate.parse::<u128>()?
    } else if let Some(flags) = &config.flags {
        if let Some(rate) = flags.rate {
            rate as u128
        } else {
            DEFAULT_REFRESH_RATE_IN_MILLISECONDS
        }
    } else {
        DEFAULT_REFRESH_RATE_IN_MILLISECONDS
    };

    if update_rate_in_milliseconds < 250 {
        return Err(BottomError::InvalidArg(
            "Please set your update rate to be greater than 250 milliseconds.".to_string(),
        ));
    } else if update_rate_in_milliseconds > u128::from(std::u64::MAX) {
        return Err(BottomError::InvalidArg(
            "Please set your update rate to be less than unsigned INT_MAX.".to_string(),
        ));
    }

    Ok(update_rate_in_milliseconds)
}

pub fn get_temperature_option(
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
                "fahrenheit" | "f" => {
                    Ok(data_harvester::temperature::TemperatureType::Fahrenheit)
                }
                "kelvin" | "k" => {
                    Ok(data_harvester::temperature::TemperatureType::Kelvin)
                }
                "celsius" | "c" => {
                    Ok(data_harvester::temperature::TemperatureType::Celsius)
                }
                _ => {
                    Err(BottomError::ConfigError(
                        "Invalid temperature type.  Please have the value be of the form <kelvin|k|celsius|c|fahrenheit|f>".to_string()
                    ))
                }
            };
        }
    }
    Ok(data_harvester::temperature::TemperatureType::Celsius)
}

pub fn get_avg_cpu_option(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("AVG_CPU") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(avg_cpu) = flags.avg_cpu {
            return avg_cpu;
        }
    }

    false
}

pub fn get_use_dot_option(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("DOT_MARKER") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(dot_marker) = flags.dot_marker {
            return dot_marker;
        }
    }
    false
}

pub fn get_use_left_legend_option(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("LEFT_LEGEND") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(left_legend) = flags.left_legend {
            return left_legend;
        }
    }

    false
}

pub fn get_use_current_cpu_total_option(
    matches: &clap::ArgMatches<'static>, config: &Config,
) -> bool {
    if matches.is_present("USE_CURR_USAGE") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(current_usage) = flags.current_usage {
            return current_usage;
        }
    }

    false
}

pub fn get_show_disabled_data_option(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("SHOW_DISABLED_DATA") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(show_disabled_data) = flags.show_disabled_data {
            return show_disabled_data;
        }
    }

    false
}

pub fn get_use_basic_mode_option(matches: &clap::ArgMatches<'static>, config: &Config) -> bool {
    if matches.is_present("BASIC_MODE") {
        return true;
    } else if let Some(flags) = &config.flags {
        if let Some(basic_mode) = flags.basic_mode {
            return basic_mode;
        }
    }

    false
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

pub fn get_default_widget(matches: &clap::ArgMatches<'static>, config: &Config) -> WidgetPosition {
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
