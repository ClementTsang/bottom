//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use std::collections::{HashMap, VecDeque};

use crate::{
    app::{data_farmer, data_harvester, App, Filter},
    utils::gen_util::*,
};

/// Point is of time, data
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
    // TODO: [NETWORKING] add min/max/mean of each
    // min_rx : f64,
    // max_rx : f64,
    // mean_rx: f64,
    // min_tx: f64,
    // max_tx: f64,
    // mean_tx: f64,
}

// TODO: [REFACTOR] Process data... stuff really needs a rewrite.  Again.
#[derive(Clone, Default, Debug)]
pub struct ConvertedProcessData {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub command: String,
    pub is_thread: Option<bool>,
    pub cpu_percent_usage: f64,
    pub mem_percent_usage: f64,
    pub mem_usage_bytes: u64,
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
pub struct ConvertedCpuData {
    pub cpu_name: String,
    /// Tuple is time, value
    pub cpu_data: Vec<Point>,
    /// Represents the value displayed on the legend.
    pub legend_value: String,
}

pub fn convert_temp_row(app: &App) -> Vec<Vec<String>> {
    let current_data = &app.data_collection;
    let temp_type = &app.app_config_fields.temperature_type;
    let temp_filter = &app.filters.temp_filter;

    let mut sensor_vector: Vec<Vec<String>> = current_data
        .temp_harvest
        .iter()
        .filter_map(|temp_harvest| {
            let name = match (&temp_harvest.component_name, &temp_harvest.component_label) {
                (Some(name), Some(label)) => format!("{}: {}", name, label),
                (None, Some(label)) => label.to_string(),
                (Some(name), None) => name.to_string(),
                (None, None) => String::default(),
            };

            let to_keep = if let Some(temp_filter) = temp_filter {
                let mut ret = temp_filter.is_list_ignored;
                for r in &temp_filter.list {
                    if r.is_match(&name) {
                        ret = !temp_filter.is_list_ignored;
                        break;
                    }
                }
                ret
            } else {
                true
            };

            if to_keep {
                Some(vec![
                    name,
                    (temp_harvest.temperature.ceil() as u64).to_string()
                        + match temp_type {
                            data_harvester::temperature::TemperatureType::Celsius => "C",
                            data_harvester::temperature::TemperatureType::Kelvin => "K",
                            data_harvester::temperature::TemperatureType::Fahrenheit => "F",
                        },
                ])
            } else {
                None
            }
        })
        .collect();

    if sensor_vector.is_empty() {
        sensor_vector.push(vec!["No Sensors Found".to_string(), "".to_string()]);
    }

    sensor_vector
}

pub fn convert_disk_row(
    current_data: &data_farmer::DataCollection, disk_filter: &Option<Filter>,
) -> Vec<Vec<String>> {
    let mut disk_vector: Vec<Vec<String>> = Vec::new();

    current_data
        .disk_harvest
        .iter()
        .filter(|disk_harvest| {
            if let Some(disk_filter) = disk_filter {
                for r in &disk_filter.list {
                    if r.is_match(&disk_harvest.name) {
                        return !disk_filter.is_list_ignored;
                    }
                }
                disk_filter.is_list_ignored
            } else {
                true
            }
        })
        .zip(&current_data.io_labels)
        .for_each(|(disk, (io_read, io_write))| {
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
                io_read.to_string(),
                io_write.to_string(),
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
    current_data: &data_farmer::DataCollection,
) -> Vec<ConvertedProcessData> {
    // FIXME: Thread highlighting and hiding support
    // For macOS see https://github.com/hishamhm/htop/pull/848/files

    current_data
        .process_harvest
        .iter()
        .map(|process| {
            let converted_rps = get_exact_byte_values(process.read_bytes_per_sec, false);
            let converted_wps = get_exact_byte_values(process.write_bytes_per_sec, false);
            let converted_total_read = get_exact_byte_values(process.total_read_bytes, false);
            let converted_total_write = get_exact_byte_values(process.total_write_bytes, false);

            let read_per_sec = format!("{:.*}{}/s", 0, converted_rps.0, converted_rps.1);
            let write_per_sec = format!("{:.*}{}/s", 0, converted_wps.0, converted_wps.1);
            let total_read = format!("{:.*}{}", 0, converted_total_read.0, converted_total_read.1);
            let total_write = format!(
                "{:.*}{}",
                0, converted_total_write.0, converted_total_write.1
            );

            ConvertedProcessData {
                pid: process.pid,
                ppid: process.parent_pid,
                is_thread: None,
                name: process.name.to_string(),
                command: process.command.to_string(),
                cpu_percent_usage: process.cpu_usage_percent,
                mem_percent_usage: process.mem_usage_percent,
                mem_usage_bytes: process.mem_usage_bytes,
                mem_usage_str: get_exact_byte_values(process.mem_usage_bytes, false),
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
        .collect::<Vec<_>>()
}

pub fn tree_process_data(
    single_process_data: &[ConvertedProcessData],
) -> Vec<ConvertedProcessData> {
    // Let's first build up a (really terrible) parent -> child mapping...
    // At the same time, let's make a mapping of PID -> process data!
    // TODO: ideally... I shouldn't have to do this... this seems kinda... geh.
    let mut parent_child_mapping: HashMap<u32, Vec<u32>> = HashMap::default();
    let mut pid_process_mapping: HashMap<u32, &ConvertedProcessData> = HashMap::default();

    single_process_data.iter().for_each(|process| {
        parent_child_mapping
            .entry(process.ppid.unwrap_or(0))
            .or_insert_with(Vec::new)
            .push(process.pid);

        // There should be no collisions...
        if pid_process_mapping.contains_key(&process.pid) {
            debug!("There was a PID collision!");
        }
        pid_process_mapping.insert(process.pid, process);
    });

    // Turn the parent-child mapping into a "list" via DFS...
    let mut pids_to_explore: VecDeque<u32> = VecDeque::default();
    let mut explored_pids: Vec<u32> = vec![0];
    if let Some(zero_pid) = parent_child_mapping.get(&0) {
        pids_to_explore.extend(zero_pid);
    } else {
        // FIXME: Remove this, this is for debugging
        debug!("PID 0 had no children during tree building...");
    }

    while let Some(current_pid) = pids_to_explore.pop_front() {
        explored_pids.push(current_pid);
        if let Some(children) = parent_child_mapping.get(&current_pid) {
            for child in children {
                pids_to_explore.push_front(*child);
            }
        }
    }

    // Now let's "rearrange" our current list of converted process data into the correct
    // order required...

    explored_pids
        .iter()
        .filter_map(|pid| match pid_process_mapping.remove(pid) {
            Some(proc) => Some(proc.clone()),
            None => None,
        })
        .collect::<Vec<_>>()
}

pub fn group_process_data(
    single_process_data: &[ConvertedProcessData], is_using_command: ProcessNamingType,
) -> Vec<ConvertedProcessData> {
    #[derive(Clone, Default, Debug)]
    struct SingleProcessData {
        pub pid: u32,
        pub cpu_percent_usage: f64,
        pub mem_percent_usage: f64,
        pub mem_usage_bytes: u64,
        pub group_pids: Vec<u32>,
        pub read_per_sec: f64,
        pub write_per_sec: f64,
        pub total_read: f64,
        pub total_write: f64,
        pub process_state: String,
    }

    let mut grouped_hashmap: HashMap<String, SingleProcessData> = std::collections::HashMap::new();

    single_process_data.iter().for_each(|process| {
        let entry = grouped_hashmap
            .entry(match is_using_command {
                ProcessNamingType::Name => process.name.to_string(),
                ProcessNamingType::Path => process.command.to_string(),
            })
            .or_insert(SingleProcessData {
                pid: process.pid,
                ..SingleProcessData::default()
            });

        (*entry).cpu_percent_usage += process.cpu_percent_usage;
        (*entry).mem_percent_usage += process.mem_percent_usage;
        (*entry).mem_usage_bytes += process.mem_usage_bytes;
        (*entry).group_pids.push(process.pid);
        (*entry).read_per_sec += process.rps_f64;
        (*entry).write_per_sec += process.wps_f64;
        (*entry).total_read += process.tr_f64;
        (*entry).total_write += process.tw_f64;
    });

    grouped_hashmap
        .iter()
        .map(|(identifier, process_details)| {
            let p = process_details.clone();
            let converted_rps = get_exact_byte_values(p.read_per_sec as u64, false);
            let converted_wps = get_exact_byte_values(p.write_per_sec as u64, false);
            let converted_total_read = get_exact_byte_values(p.total_read as u64, false);
            let converted_total_write = get_exact_byte_values(p.total_write as u64, false);

            let read_per_sec = format!("{:.*}{}/s", 0, converted_rps.0, converted_rps.1);
            let write_per_sec = format!("{:.*}{}/s", 0, converted_wps.0, converted_wps.1);
            let total_read = format!("{:.*}{}", 0, converted_total_read.0, converted_total_read.1);
            let total_write = format!(
                "{:.*}{}",
                0, converted_total_write.0, converted_total_write.1
            );

            ConvertedProcessData {
                pid: p.pid,
                ppid: None,
                is_thread: None,
                name: identifier.to_string(),
                command: identifier.to_string(),
                cpu_percent_usage: p.cpu_percent_usage,
                mem_percent_usage: p.mem_percent_usage,
                mem_usage_bytes: p.mem_usage_bytes,
                mem_usage_str: get_exact_byte_values(p.mem_usage_bytes, false),
                group_pids: p.group_pids,
                read_per_sec,
                write_per_sec,
                total_read,
                total_write,
                rps_f64: p.read_per_sec,
                wps_f64: p.write_per_sec,
                tr_f64: p.total_read,
                tw_f64: p.total_write,
                process_state: p.process_state,
            }
        })
        .collect::<Vec<_>>()
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
