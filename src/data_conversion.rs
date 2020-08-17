//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use std::collections::HashMap;

use crate::{
    app::{data_farmer, data_harvester, App},
    utils::gen_util::{get_exact_byte_values, get_simple_byte_values},
};

type Point = (f64, f64);

#[derive(Default, Debug)]
pub struct ConvertedBatteryData {
    pub battery_name: String,
    pub charge_percentage: f64,
    pub watt_consumption: String,
    pub duration_until_full: Option<String>,
    pub duration_until_empty: Option<String>,
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
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedProcessData {
    pub pid: u32,
    pub name: String,
    pub cpu_percent_usage: f64,
    pub mem_percent_usage: f64,
    pub mem_usage_kb: u64,
    pub mem_usage_str: (f64, String),
    pub group_pids: Vec<u32>,
    pub read_per_sec: String,
    pub write_per_sec: String,
    pub total_read: String,
    pub total_write: String,
    pub rps_f64: f64,
    pub wps_f64: f64,
    pub tr_f64: f64,
    pub tw_f64: f64,
    pub process_state: String,
}

#[derive(Clone, Default, Debug)]
pub struct SingleProcessData {
    pub pid: u32,
    pub cpu_percent_usage: f64,
    pub mem_percent_usage: f64,
    pub mem_usage_kb: u64,
    pub group_pids: Vec<u32>,
    pub read_per_sec: u64,
    pub write_per_sec: u64,
    pub total_read: u64,
    pub total_write: u64,
    pub process_state: String,
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedCpuData {
    pub cpu_name: String,
    /// Tuple is time, value
    pub cpu_data: Vec<Point>,
    /// Represents the value displayed on the legend.
    pub legend_value: String,
}

pub fn convert_temp_row(app: &App) -> Vec<Vec<String>> {
    let mut sensor_vector: Vec<Vec<String>> = Vec::new();

    let current_data = &app.data_collection;
    let temp_type = &app.app_config_fields.temperature_type;

    if current_data.temp_harvest.is_empty() {
        sensor_vector.push(vec!["No Sensors Found".to_string(), "".to_string()])
    } else {
        for sensor in &current_data.temp_harvest {
            sensor_vector.push(vec![
                sensor.component_name.to_string(),
                (sensor.temperature.ceil() as u64).to_string()
                    + match temp_type {
                        data_harvester::temperature::TemperatureType::Celsius => "C",
                        data_harvester::temperature::TemperatureType::Kelvin => "K",
                        data_harvester::temperature::TemperatureType::Fahrenheit => "F",
                    },
            ]);
        }
    }

    sensor_vector
}

pub fn convert_disk_row(current_data: &data_farmer::DataCollection) -> Vec<Vec<String>> {
    let mut disk_vector: Vec<Vec<String>> = Vec::new();
    current_data
        .disk_harvest
        .iter()
        .zip(&current_data.io_labels_and_prev)
        .for_each(|(disk, (io_label, _io_prev))| {
            let converted_read = get_simple_byte_values(io_label.0, false);
            let converted_write = get_simple_byte_values(io_label.1, false);
            let io_activity = (
                format!("{:.*}{}/s", 0, converted_read.0, converted_read.1),
                format!("{:.*}{}/s", 0, converted_write.0, converted_write.1),
            );

            let converted_free_space = get_simple_byte_values(disk.free_space, false);
            let converted_total_space = get_simple_byte_values(disk.total_space, false);
            disk_vector.push(vec![
                disk.name.to_string(),
                disk.mount_point.to_string(),
                format!(
                    "{:.0}%",
                    disk.used_space as f64 / disk.total_space as f64 * 100_f64
                ),
                format!("{:.*}{}", 0, converted_free_space.0, converted_free_space.1),
                format!(
                    "{:.*}{}",
                    0, converted_total_space.0, converted_total_space.1
                ),
                io_activity.0,
                io_activity.1,
            ]);
        });

    disk_vector
}

pub fn convert_cpu_data_points(
    current_data: &data_farmer::DataCollection, is_frozen: bool,
) -> Vec<ConvertedCpuData> {
    let mut cpu_data_vector: Vec<ConvertedCpuData> = Vec::new();
    let current_time = if is_frozen {
        if let Some(frozen_instant) = current_data.frozen_instant {
            frozen_instant
        } else {
            current_data.current_instant
        }
    } else {
        current_data.current_instant
    };

    for (time, data) in &current_data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();

        for (itx, cpu) in data.cpu_data.iter().enumerate() {
            // Check if the vector exists yet
            if cpu_data_vector.len() <= itx {
                let mut new_cpu_data = ConvertedCpuData::default();
                new_cpu_data.cpu_name = if let Some(cpu_harvest) = current_data.cpu_harvest.get(itx)
                {
                    cpu_harvest.cpu_name.to_string()
                } else {
                    String::default()
                };
                cpu_data_vector.push(new_cpu_data);
            }

            if let Some(cpu_data) = cpu_data_vector.get_mut(itx) {
                cpu_data.legend_value = format!("{:.0}%", cpu.round());
                cpu_data.cpu_data.push((-time_from_start, *cpu));
            }
        }

        if *time == current_time {
            break;
        }
    }

    let mut extended_vec = vec![ConvertedCpuData {
        cpu_name: "All".to_string(),
        cpu_data: vec![],
        legend_value: String::new(),
    }];
    extended_vec.extend(cpu_data_vector);
    extended_vec
}

pub fn convert_mem_data_points(
    current_data: &data_farmer::DataCollection, is_frozen: bool,
) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = if is_frozen {
        if let Some(frozen_instant) = current_data.frozen_instant {
            frozen_instant
        } else {
            current_data.current_instant
        }
    } else {
        current_data.current_instant
    };

    for (time, data) in &current_data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();
        result.push((-time_from_start, data.mem_data));
        if *time == current_time {
            break;
        }
    }

    result
}

pub fn convert_swap_data_points(
    current_data: &data_farmer::DataCollection, is_frozen: bool,
) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = if is_frozen {
        if let Some(frozen_instant) = current_data.frozen_instant {
            frozen_instant
        } else {
            current_data.current_instant
        }
    } else {
        current_data.current_instant
    };

    for (time, data) in &current_data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();
        result.push((-time_from_start, data.swap_data));
        if *time == current_time {
            break;
        }
    }

    result
}

pub fn convert_mem_labels(
    current_data: &data_farmer::DataCollection,
) -> (String, String, String, String) {
    (
        format!(
            "{:3.0}%",
            match current_data.memory_harvest.mem_total_in_mb {
                0 => 0.0,
                _ =>
                    current_data.memory_harvest.mem_used_in_mb as f64 * 100.0
                        / current_data.memory_harvest.mem_total_in_mb as f64,
            }
        ),
        format!(
            "   {:.1}GB/{:.1}GB",
            current_data.memory_harvest.mem_used_in_mb as f64 / 1024.0,
            (current_data.memory_harvest.mem_total_in_mb as f64 / 1024.0)
        ),
        format!(
            "{:3.0}%",
            match current_data.swap_harvest.mem_total_in_mb {
                0 => 0.0,
                _ =>
                    current_data.swap_harvest.mem_used_in_mb as f64 * 100.0
                        / current_data.swap_harvest.mem_total_in_mb as f64,
            }
        ),
        format!(
            "   {:.1}GB/{:.1}GB",
            current_data.swap_harvest.mem_used_in_mb as f64 / 1024.0,
            (current_data.swap_harvest.mem_total_in_mb as f64 / 1024.0)
        ),
    )
}

pub fn get_rx_tx_data_points(
    current_data: &data_farmer::DataCollection, is_frozen: bool,
) -> (Vec<Point>, Vec<Point>) {
    let mut rx: Vec<Point> = Vec::new();
    let mut tx: Vec<Point> = Vec::new();

    let current_time = if is_frozen {
        if let Some(frozen_instant) = current_data.frozen_instant {
            frozen_instant
        } else {
            current_data.current_instant
        }
    } else {
        current_data.current_instant
    };

    for (time, data) in &current_data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();
        rx.push((-time_from_start, data.rx_data));
        tx.push((-time_from_start, data.tx_data));
        if *time == current_time {
            break;
        }
    }

    (rx, tx)
}

pub fn convert_network_data_points(
    current_data: &data_farmer::DataCollection, is_frozen: bool, need_four_points: bool,
) -> ConvertedNetworkData {
    let (rx, tx) = get_rx_tx_data_points(current_data, is_frozen);

    let total_rx_converted_result: (f64, String);
    let rx_converted_result: (f64, String);
    let total_tx_converted_result: (f64, String);
    let tx_converted_result: (f64, String);

    rx_converted_result = get_exact_byte_values(current_data.network_harvest.rx, false);
    total_rx_converted_result = get_exact_byte_values(current_data.network_harvest.total_rx, false);

    tx_converted_result = get_exact_byte_values(current_data.network_harvest.tx, false);
    total_tx_converted_result = get_exact_byte_values(current_data.network_harvest.total_tx, false);

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
            "RX: {:<9} All: {:<9}",
            format!("{:.1}{:3}", rx_converted_result.0, rx_converted_result.1),
            format!(
                "{:.1}{:3}",
                total_rx_converted_result.0, total_rx_converted_result.1
            )
        );
        let tx_display = format!(
            "TX: {:<9} All: {:<9}",
            format!("{:.1}{:3}", tx_converted_result.0, tx_converted_result.1),
            format!(
                "{:.1}{:3}",
                total_tx_converted_result.0, total_tx_converted_result.1
            )
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

pub enum ProcessGroupingType {
    Grouped,
    Ungrouped,
}

pub enum ProcessNamingType {
    Name,
    Path,
}

pub fn convert_process_data(
    current_data: &data_farmer::DataCollection, grouping_type: ProcessGroupingType,
    name_type: ProcessNamingType,
) -> Vec<ConvertedProcessData> {
    match grouping_type {
        ProcessGroupingType::Ungrouped => current_data
            .process_harvest
            .iter()
            .map(|process| {
                let converted_rps = get_exact_byte_values(process.read_bytes_per_sec, false);
                let converted_wps = get_exact_byte_values(process.write_bytes_per_sec, false);
                let converted_total_read = get_exact_byte_values(process.total_read_bytes, false);
                let converted_total_write = get_exact_byte_values(process.total_write_bytes, false);

                let read_per_sec = format!("{:.*}{}/s", 0, converted_rps.0, converted_rps.1);
                let write_per_sec = format!("{:.*}{}/s", 0, converted_wps.0, converted_wps.1);
                let total_read =
                    format!("{:.*}{}", 0, converted_total_read.0, converted_total_read.1);
                let total_write = format!(
                    "{:.*}{}",
                    0, converted_total_write.0, converted_total_write.1
                );

                ConvertedProcessData {
                    pid: process.pid,
                    name: match name_type {
                        ProcessNamingType::Name => process.name.to_string(),
                        ProcessNamingType::Path => process.path.to_string(),
                    },
                    cpu_percent_usage: process.cpu_usage_percent,
                    mem_percent_usage: process.mem_usage_percent,
                    mem_usage_kb: process.mem_usage_kb,
                    mem_usage_str: get_exact_byte_values(process.mem_usage_kb * 1024, false),
                    group_pids: vec![process.pid],
                    read_per_sec,
                    write_per_sec,
                    total_read,
                    total_write,
                    rps_f64: process.read_bytes_per_sec as f64,
                    wps_f64: process.write_bytes_per_sec as f64,
                    tr_f64: process.total_read_bytes as f64,
                    tw_f64: process.total_write_bytes as f64,
                    process_state: process.process_state.to_owned(),
                }
            })
            .collect::<Vec<_>>(),
        ProcessGroupingType::Grouped => {
            let mut grouped_hashmap: HashMap<String, SingleProcessData> =
                std::collections::HashMap::new();

            current_data.process_harvest.iter().for_each(|process| {
                let entry = grouped_hashmap
                    .entry(match name_type {
                        ProcessNamingType::Name => process.name.to_string(),
                        ProcessNamingType::Path => process.path.to_string(),
                    })
                    .or_insert(SingleProcessData {
                        pid: process.pid,
                        ..SingleProcessData::default()
                    });

                (*entry).cpu_percent_usage += process.cpu_usage_percent;
                (*entry).mem_percent_usage += process.mem_usage_percent;
                (*entry).mem_usage_kb += process.mem_usage_kb;
                (*entry).group_pids.push(process.pid);
                (*entry).read_per_sec += process.read_bytes_per_sec;
                (*entry).write_per_sec += process.write_bytes_per_sec;
                (*entry).total_read += process.total_read_bytes;
                (*entry).total_write += process.total_write_bytes;
            });

            grouped_hashmap
                .iter()
                .map(|(identifier, process_details)| {
                    let p = process_details.clone();
                    let converted_rps = get_exact_byte_values(p.read_per_sec, false);
                    let converted_wps = get_exact_byte_values(p.write_per_sec, false);
                    let converted_total_read = get_exact_byte_values(p.total_read, false);
                    let converted_total_write = get_exact_byte_values(p.total_write, false);

                    let read_per_sec = format!("{:.*}{}/s", 0, converted_rps.0, converted_rps.1);
                    let write_per_sec = format!("{:.*}{}/s", 0, converted_wps.0, converted_wps.1);
                    let total_read =
                        format!("{:.*}{}", 0, converted_total_read.0, converted_total_read.1);
                    let total_write = format!(
                        "{:.*}{}",
                        0, converted_total_write.0, converted_total_write.1
                    );

                    ConvertedProcessData {
                        pid: p.pid,
                        name: identifier.to_string(),
                        cpu_percent_usage: p.cpu_percent_usage,
                        mem_percent_usage: p.mem_percent_usage,
                        mem_usage_kb: p.mem_usage_kb,
                        mem_usage_str: get_exact_byte_values(p.mem_usage_kb * 1024, false),
                        group_pids: p.group_pids,
                        read_per_sec,
                        write_per_sec,
                        total_read,
                        total_write,
                        rps_f64: p.read_per_sec as f64,
                        wps_f64: p.write_per_sec as f64,
                        tr_f64: p.total_read as f64,
                        tw_f64: p.total_write as f64,
                        process_state: p.process_state,
                    }
                })
                .collect::<Vec<_>>()
        }
    }
}

pub fn convert_battery_harvest(
    current_data: &data_farmer::DataCollection,
) -> Vec<ConvertedBatteryData> {
    current_data
        .battery_harvest
        .iter()
        .enumerate()
        .map(|(itx, battery_harvest)| ConvertedBatteryData {
            battery_name: format!("Battery {}", itx),
            charge_percentage: battery_harvest.charge_percent,
            watt_consumption: format!("{:.2}W", battery_harvest.power_consumption_rate_watts),
            duration_until_empty: if let Some(secs_till_empty) = battery_harvest.secs_until_empty {
                let time = chrono::Duration::seconds(secs_till_empty);
                let num_minutes = time.num_minutes() - time.num_hours() * 60;
                let num_seconds = time.num_seconds() - time.num_minutes() * 60;
                Some(format!(
                    "{} hour{}, {} minute{}, {} second{}",
                    time.num_hours(),
                    if time.num_hours() == 1 { "" } else { "s" },
                    num_minutes,
                    if num_minutes == 1 { "" } else { "s" },
                    num_seconds,
                    if num_seconds == 1 { "" } else { "s" },
                ))
            } else {
                None
            },
            duration_until_full: if let Some(secs_till_full) = battery_harvest.secs_until_full {
                let time = chrono::Duration::seconds(secs_till_full);
                let num_minutes = time.num_minutes() - time.num_hours() * 60;
                let num_seconds = time.num_seconds() - time.num_minutes() * 60;
                Some(format!(
                    "{} hour{}, {} minute{}, {} second{}",
                    time.num_hours(),
                    if time.num_hours() == 1 { "" } else { "s" },
                    num_minutes,
                    if num_minutes == 1 { "" } else { "s" },
                    num_seconds,
                    if num_seconds == 1 { "" } else { "s" },
                ))
            } else {
                None
            },
            health: format!("{:.2}%", battery_harvest.health_percent),
        })
        .collect()
}
