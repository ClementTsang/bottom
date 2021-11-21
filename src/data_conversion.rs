//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.
use crate::app::data_harvester::temperature::TemperatureType;
use crate::app::text_table::TextTableData;
use crate::app::DataCollection;
use crate::{app::data_harvester, utils::gen_util::*};
use crate::{app::AxisScaling, units::data_units::DataUnit, Pid};

use std::borrow::Cow;

/// Point is of time, data
type Point = (f64, f64);

#[derive(Debug)]
pub enum BatteryDuration {
    Charging { short: String, long: String },
    Discharging { short: String, long: String },
    Neither,
}

impl Default for BatteryDuration {
    fn default() -> Self {
        Self::Neither
    }
}

#[derive(Default, Debug)]
pub struct ConvertedBatteryData {
    pub battery_name: String,
    pub charge_percentage: f64,
    pub watt_consumption: String,
    pub charge_times: BatteryDuration,
    pub health: String,
}

#[derive(Default, Debug)]
pub struct ConvertedNetworkData {
    pub rx: Vec<Point>,
    pub tx: Vec<Point>,
    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: Option<String>,
    pub total_tx_display: Option<String>,
    // TODO: [Feature] Networking - add the following stats in the future!
    // min_rx : f64,
    // max_rx : f64,
    // mean_rx: f64,
    // min_tx: f64,
    // max_tx: f64,
    // mean_tx: f64,
}

// TODO: [Refactor] Process data might need some refactoring lol
#[derive(Clone, Default, Debug)]
pub struct ConvertedProcessData {
    pub pid: Pid,
    pub ppid: Option<Pid>,
    pub name: String,
    pub command: String,
    pub is_thread: Option<bool>,
    pub cpu_percent_usage: f64,
    pub mem_percent_usage: f64,
    pub mem_usage_bytes: u64,
    pub mem_usage_str: (f64, String),
    pub group_pids: Vec<Pid>,
    pub read_per_sec: String,
    pub write_per_sec: String,
    pub total_read: String,
    pub total_write: String,
    pub rps_f64: f64,
    pub wps_f64: f64,
    pub tr_f64: f64,
    pub tw_f64: f64,
    pub process_state: String,
    pub process_char: char,
    pub user: Option<String>,

    /// Prefix printed before the process when displayed.
    pub process_description_prefix: Option<String>,
    /// Whether to mark this process entry as disabled (mostly for tree mode).
    pub is_disabled_entry: bool,
    /// Whether this entry is collapsed, hiding all its children (for tree mode).
    pub is_collapsed_entry: bool,
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedCpuData {
    pub cpu_name: String,
    pub short_cpu_name: String,
    /// Tuple is time, value
    pub cpu_data: Vec<Point>,
    /// Represents the value displayed on the legend.
    pub legend_value: String,
}

pub fn convert_temp_row(
    current_data: &DataCollection, temp_type: &TemperatureType,
) -> TextTableData {
    if current_data.temp_harvest.is_empty() {
        vec![vec![
            ("No Sensors Found".into(), Some("N/A".into()), None),
            ("".into(), None, None),
        ]]
    } else {
        let (unit_long, unit_short) = match temp_type {
            data_harvester::temperature::TemperatureType::Celsius => ("°C", "C"),
            data_harvester::temperature::TemperatureType::Kelvin => ("K", "K"),
            data_harvester::temperature::TemperatureType::Fahrenheit => ("°F", "F"),
        };

        current_data
            .temp_harvest
            .iter()
            .map(|temp_harvest| {
                let val = temp_harvest.temperature.ceil().to_string();
                vec![
                    (temp_harvest.name.clone().into(), None, None),
                    (
                        format!("{}{}", val, unit_long).into(),
                        Some(format!("{}{}", val, unit_short).into()),
                        None,
                    ),
                ]
            })
            .collect()
    }
}

pub fn convert_disk_row(current_data: &DataCollection) -> TextTableData {
    if current_data.disk_harvest.is_empty() {
        vec![vec![
            ("No Disks Found".into(), Some("N/A".into()), None),
            ("".into(), None, None),
        ]]
    } else {
        current_data
            .disk_harvest
            .iter()
            .zip(&current_data.io_labels)
            .map(|(disk, (io_read, io_write))| {
                let free_space_fmt = if let Some(free_space) = disk.free_space {
                    let converted_free_space = get_decimal_bytes(free_space);
                    Cow::Owned(format!(
                        "{:.*}{}",
                        0, converted_free_space.0, converted_free_space.1
                    ))
                } else {
                    "N/A".into()
                };
                let total_space_fmt = if let Some(total_space) = disk.total_space {
                    let converted_total_space = get_decimal_bytes(total_space);
                    Cow::Owned(format!(
                        "{:.*}{}",
                        0, converted_total_space.0, converted_total_space.1
                    ))
                } else {
                    "N/A".into()
                };

                let usage_fmt = if let (Some(used_space), Some(total_space)) =
                    (disk.used_space, disk.total_space)
                {
                    Cow::Owned(format!(
                        "{:.0}%",
                        used_space as f64 / total_space as f64 * 100_f64
                    ))
                } else {
                    "N/A".into()
                };

                vec![
                    (disk.name.clone().into(), None, None),
                    (disk.mount_point.clone().into(), None, None),
                    (usage_fmt, None, None),
                    (free_space_fmt, None, None),
                    (total_space_fmt, None, None),
                    (io_read.clone().into(), None, None),
                    (io_write.clone().into(), None, None),
                ]
            })
            .collect::<Vec<_>>()
    }
}

pub fn convert_cpu_data_points(
    current_data: &DataCollection, existing_cpu_data: &mut Vec<ConvertedCpuData>,
) {
    let current_time = current_data.current_instant;

    // Initialize cpu_data_vector if the lengths don't match...
    if let Some((_time, data)) = &current_data.timed_data_vec.last() {
        if data.cpu_data.len() + 1 != existing_cpu_data.len() {
            *existing_cpu_data = vec![ConvertedCpuData {
                cpu_name: "All".to_string(),
                short_cpu_name: "All".to_string(),
                cpu_data: vec![],
                legend_value: String::new(),
            }];

            existing_cpu_data.extend(
                data.cpu_data
                    .iter()
                    .enumerate()
                    .map(|(itx, cpu_usage)| ConvertedCpuData {
                        cpu_name: if let Some(cpu_harvest) = current_data.cpu_harvest.get(itx) {
                            if let Some(cpu_count) = cpu_harvest.cpu_count {
                                format!("{}{}", cpu_harvest.cpu_prefix, cpu_count)
                            } else {
                                cpu_harvest.cpu_prefix.to_string()
                            }
                        } else {
                            String::default()
                        },
                        short_cpu_name: if let Some(cpu_harvest) = current_data.cpu_harvest.get(itx)
                        {
                            if let Some(cpu_count) = cpu_harvest.cpu_count {
                                cpu_count.to_string()
                            } else {
                                cpu_harvest.cpu_prefix.to_string()
                            }
                        } else {
                            String::default()
                        },
                        legend_value: format!("{:.0}%", cpu_usage.round()),
                        cpu_data: vec![],
                    })
                    .collect::<Vec<ConvertedCpuData>>(),
            );
        } else {
            existing_cpu_data
                .iter_mut()
                .skip(1)
                .zip(&data.cpu_data)
                .for_each(|(cpu, cpu_usage)| {
                    cpu.cpu_data = vec![];
                    cpu.legend_value = format!("{:.0}%", cpu_usage.round());
                });
        }

        for (time, data) in &current_data.timed_data_vec {
            let time_from_start: f64 =
                (current_time.duration_since(*time).as_millis() as f64).floor();

            for (itx, cpu) in data.cpu_data.iter().enumerate() {
                if let Some(cpu_data) = existing_cpu_data.get_mut(itx + 1) {
                    cpu_data.cpu_data.push((-time_from_start, *cpu));
                }
            }

            if *time == current_time {
                break;
            }
        }
    } else {
        // No data, clear if non-empty. This is usually the case after a reset.
        if !existing_cpu_data.is_empty() {
            *existing_cpu_data = vec![];
        }
    }
}

pub fn convert_mem_data_points(current_data: &DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = current_data.current_instant;

    for (time, data) in &current_data.timed_data_vec {
        if let Some(mem_data) = data.mem_data {
            let time_from_start: f64 =
                (current_time.duration_since(*time).as_millis() as f64).floor();
            result.push((-time_from_start, mem_data));
            if *time == current_time {
                break;
            }
        }
    }

    result
}

pub fn convert_swap_data_points(current_data: &DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = current_data.current_instant;

    for (time, data) in &current_data.timed_data_vec {
        if let Some(swap_data) = data.swap_data {
            let time_from_start: f64 =
                (current_time.duration_since(*time).as_millis() as f64).floor();
            result.push((-time_from_start, swap_data));
            if *time == current_time {
                break;
            }
        }
    }

    result
}

pub fn convert_mem_labels(
    current_data: &DataCollection,
) -> (Option<(String, String)>, Option<(String, String)>) {
    /// Returns the unit type and denominator for given total amount of memory in kibibytes.
    fn return_unit_and_denominator_for_mem_kib(mem_total_kib: u64) -> (&'static str, f64) {
        if mem_total_kib < 1024 {
            // Stay with KiB
            ("KiB", 1.0)
        } else if mem_total_kib < MEBI_LIMIT {
            // Use MiB
            ("MiB", KIBI_LIMIT_F64)
        } else if mem_total_kib < GIBI_LIMIT {
            // Use GiB
            ("GiB", MEBI_LIMIT_F64)
        } else {
            // Use TiB
            ("TiB", GIBI_LIMIT_F64)
        }
    }

    // TODO: [Refactor] Should probably make this only return none if no data is left/visible?
    (
        if current_data.memory_harvest.mem_total_in_kib > 0 {
            Some((
                format!(
                    "{:3.0}%",
                    current_data.memory_harvest.use_percent.unwrap_or(0.0)
                ),
                {
                    let (unit, denominator) = return_unit_and_denominator_for_mem_kib(
                        current_data.memory_harvest.mem_total_in_kib,
                    );

                    format!(
                        "   {:.1}{}/{:.1}{}",
                        current_data.memory_harvest.mem_used_in_kib as f64 / denominator,
                        unit,
                        (current_data.memory_harvest.mem_total_in_kib as f64 / denominator),
                        unit
                    )
                },
            ))
        } else {
            None
        },
        if current_data.swap_harvest.mem_total_in_kib > 0 {
            Some((
                format!(
                    "{:3.0}%",
                    current_data.swap_harvest.use_percent.unwrap_or(0.0)
                ),
                {
                    let (unit, denominator) = return_unit_and_denominator_for_mem_kib(
                        current_data.swap_harvest.mem_total_in_kib,
                    );

                    format!(
                        "   {:.1}{}/{:.1}{}",
                        current_data.swap_harvest.mem_used_in_kib as f64 / denominator,
                        unit,
                        (current_data.swap_harvest.mem_total_in_kib as f64 / denominator),
                        unit
                    )
                },
            ))
        } else {
            None
        },
    )
}

pub fn get_rx_tx_data_points(
    current_data: &DataCollection, network_scale_type: &AxisScaling, network_unit_type: &DataUnit,
    network_use_binary_prefix: bool,
) -> (Vec<Point>, Vec<Point>) {
    let mut rx: Vec<Point> = Vec::new();
    let mut tx: Vec<Point> = Vec::new();

    let current_time = current_data.current_instant;

    for (time, data) in &current_data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();

        let (rx_data, tx_data) = match network_scale_type {
            AxisScaling::Log => {
                if network_use_binary_prefix {
                    match network_unit_type {
                        DataUnit::Byte => {
                            // As dividing by 8 is equal to subtracting 4 in base 2!
                            ((data.rx_data).log2() - 4.0, (data.tx_data).log2() - 4.0)
                        }
                        DataUnit::Bit => ((data.rx_data).log2(), (data.tx_data).log2()),
                    }
                } else {
                    match network_unit_type {
                        DataUnit::Byte => {
                            ((data.rx_data / 8.0).log10(), (data.tx_data / 8.0).log10())
                        }
                        DataUnit::Bit => ((data.rx_data).log10(), (data.tx_data).log10()),
                    }
                }
            }
            AxisScaling::Linear => match network_unit_type {
                DataUnit::Byte => (data.rx_data / 8.0, data.tx_data / 8.0),
                DataUnit::Bit => (data.rx_data, data.tx_data),
            },
        };

        rx.push((-time_from_start, rx_data));
        tx.push((-time_from_start, tx_data));
        if *time == current_time {
            break;
        }
    }

    (rx, tx)
}

pub fn convert_network_data_points(
    current_data: &DataCollection, need_four_points: bool, network_scale_type: &AxisScaling,
    network_unit_type: &DataUnit, network_use_binary_prefix: bool,
) -> ConvertedNetworkData {
    let (rx, tx) = get_rx_tx_data_points(
        current_data,
        network_scale_type,
        network_unit_type,
        network_use_binary_prefix,
    );

    let unit = match network_unit_type {
        DataUnit::Byte => "B/s",
        DataUnit::Bit => "b/s",
    };

    let (rx_data, tx_data, total_rx_data, total_tx_data) = match network_unit_type {
        DataUnit::Byte => (
            current_data.network_harvest.rx / 8,
            current_data.network_harvest.tx / 8,
            current_data.network_harvest.total_rx / 8,
            current_data.network_harvest.total_tx / 8,
        ),
        DataUnit::Bit => (
            current_data.network_harvest.rx,
            current_data.network_harvest.tx,
            current_data.network_harvest.total_rx / 8, // We always make this bytes...
            current_data.network_harvest.total_tx / 8,
        ),
    };

    let (rx_converted_result, total_rx_converted_result): ((f64, String), (f64, String)) =
        if network_use_binary_prefix {
            (
                get_binary_prefix(rx_data, unit), // If this isn't obvious why there's two functions, one you can configure the unit, the other is always bytes
                get_binary_bytes(total_rx_data),
            )
        } else {
            (
                get_decimal_prefix(rx_data, unit),
                get_decimal_bytes(total_rx_data),
            )
        };

    let (tx_converted_result, total_tx_converted_result): ((f64, String), (f64, String)) =
        if network_use_binary_prefix {
            (
                get_binary_prefix(tx_data, unit),
                get_binary_bytes(total_tx_data),
            )
        } else {
            (
                get_decimal_prefix(tx_data, unit),
                get_decimal_bytes(total_tx_data),
            )
        };

    if need_four_points {
        let rx_display = format!("{:.*}{}", 1, rx_converted_result.0, rx_converted_result.1);
        let total_rx_display = Some(format!(
            "{:.*}{}",
            1, total_rx_converted_result.0, total_rx_converted_result.1
        ));
        let tx_display = format!("{:.*}{}", 1, tx_converted_result.0, tx_converted_result.1);
        let total_tx_display = Some(format!(
            "{:.*}{}",
            1, total_tx_converted_result.0, total_tx_converted_result.1
        ));
        ConvertedNetworkData {
            rx,
            tx,
            rx_display,
            tx_display,
            total_rx_display,
            total_tx_display,
        }
    } else {
        let rx_display = format!(
            "RX: {:<10}  All: {}",
            if network_use_binary_prefix {
                format!("{:.1}{:3}", rx_converted_result.0, rx_converted_result.1)
            } else {
                format!("{:.1}{:2}", rx_converted_result.0, rx_converted_result.1)
            },
            if network_use_binary_prefix {
                format!(
                    "{:.1}{:3}",
                    total_rx_converted_result.0, total_rx_converted_result.1
                )
            } else {
                format!(
                    "{:.1}{:2}",
                    total_rx_converted_result.0, total_rx_converted_result.1
                )
            }
        );
        let tx_display = format!(
            "TX: {:<10}  All: {}",
            if network_use_binary_prefix {
                format!("{:.1}{:3}", tx_converted_result.0, tx_converted_result.1)
            } else {
                format!("{:.1}{:2}", tx_converted_result.0, tx_converted_result.1)
            },
            if network_use_binary_prefix {
                format!(
                    "{:.1}{:3}",
                    total_tx_converted_result.0, total_tx_converted_result.1
                )
            } else {
                format!(
                    "{:.1}{:2}",
                    total_tx_converted_result.0, total_tx_converted_result.1
                )
            }
        );

        ConvertedNetworkData {
            rx,
            tx,
            rx_display,
            tx_display,
            total_rx_display: None,
            total_tx_display: None,
        }
    }
}

/// Given read/s, write/s, total read, and total write values, return 4 strings that represent read/s, write/s, total read, and total write
pub fn get_disk_io_strings(
    rps: u64, wps: u64, total_read: u64, total_write: u64,
) -> (String, String, String, String) {
    (
        get_string_with_bytes(rps),
        get_string_with_bytes(wps),
        get_string_with_bytes(total_read),
        get_string_with_bytes(total_write),
    )
}

/// Returns a string given a value that is converted to the closest SI-variant, per second.
/// If the value is greater than a giga-X, then it will return a decimal place.
pub fn get_string_with_bytes_per_second(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.*}{}/s", 1, converted_values.0, converted_values.1)
    } else {
        format!("{:.*}{}/s", 0, converted_values.0, converted_values.1)
    }
}

/// Returns a string given a value that is converted to the closest SI-variant.
/// If the value is greater than a giga-X, then it will return a decimal place.
pub fn get_string_with_bytes(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.*}{}", 1, converted_values.0, converted_values.1)
    } else {
        format!("{:.*}{}", 0, converted_values.0, converted_values.1)
    }
}

#[cfg(feature = "battery")]
pub fn convert_battery_harvest(current_data: &DataCollection) -> Vec<ConvertedBatteryData> {
    current_data
        .battery_harvest
        .iter()
        .enumerate()
        .map(|(itx, battery_harvest)| ConvertedBatteryData {
            battery_name: format!("Battery {}", itx),
            charge_percentage: battery_harvest.charge_percent,
            watt_consumption: format!("{:.2}W", battery_harvest.power_consumption_rate_watts),
            charge_times: if let Some(secs_till_empty) = battery_harvest.secs_until_empty {
                let time = time::Duration::seconds(secs_till_empty);
                let num_minutes = time.whole_minutes() - time.whole_hours() * 60;
                let num_seconds = time.whole_seconds() - time.whole_minutes() * 60;
                BatteryDuration::Discharging {
                    long: format!(
                        "{} hour{}, {} minute{}, {} second{}",
                        time.whole_hours(),
                        if time.whole_hours() == 1 { "" } else { "s" },
                        num_minutes,
                        if num_minutes == 1 { "" } else { "s" },
                        num_seconds,
                        if num_seconds == 1 { "" } else { "s" },
                    ),
                    short: format!(
                        "{}:{:02}:{:02}",
                        time.whole_hours(),
                        num_minutes,
                        num_seconds
                    ),
                }
            } else if let Some(secs_till_full) = battery_harvest.secs_until_full {
                let time = time::Duration::seconds(secs_till_full); // TODO: [Dependencies] Can I get rid of chrono?
                let num_minutes = time.whole_minutes() - time.whole_hours() * 60;
                let num_seconds = time.whole_seconds() - time.whole_minutes() * 60;
                BatteryDuration::Charging {
                    long: format!(
                        "{} hour{}, {} minute{}, {} second{}",
                        time.whole_hours(),
                        if time.whole_hours() == 1 { "" } else { "s" },
                        num_minutes,
                        if num_minutes == 1 { "" } else { "s" },
                        num_seconds,
                        if num_seconds == 1 { "" } else { "s" },
                    ),
                    short: format!(
                        "{}:{:02}:{:02}",
                        time.whole_hours(),
                        num_minutes,
                        num_seconds
                    ),
                }
            } else {
                BatteryDuration::Neither
            },
            health: format!("{:.2}%", battery_harvest.health_percent),
        })
        .collect()
}
