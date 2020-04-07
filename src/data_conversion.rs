//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use std::collections::HashMap;

use crate::{
    app::{
        data_farmer,
        data_harvester::{self, processes::ProcessHarvest},
        App,
    },
    utils::gen_util::{get_exact_byte_values, get_simple_byte_values},
};

type Point = (f64, f64);

#[derive(Default, Debug)]
pub struct ConvertedNetworkData {
    pub rx: Vec<Point>,
    pub tx: Vec<Point>,
    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: String,
    pub total_tx_display: String,
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedProcessData {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub mem_usage: f64,
    pub group_pids: Vec<u32>,
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedCpuData {
    pub cpu_name: String,
    /// Tuple is time, value
    pub cpu_data: Vec<Point>,
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
                    cpu_harvest.cpu_name.clone()
                } else {
                    String::default()
                };
                cpu_data_vector.push(new_cpu_data);
            }

            if let Some(cpu_data) = cpu_data_vector.get_mut(itx) {
                cpu_data.legend_value = format!("{:.0}%", cpu.0.round());

                //Insert joiner points
                for &(joiner_offset, joiner_val) in &cpu.1 {
                    let offset_time = time_from_start + joiner_offset as f64;
                    cpu_data.cpu_data.push((-offset_time, joiner_val));
                }

                cpu_data.cpu_data.push((-time_from_start, cpu.0));
            }
        }

        if *time == current_time {
            break;
        }
    }

    cpu_data_vector
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

        //Insert joiner points
        for &(joiner_offset, joiner_val) in &data.mem_data.1 {
            let offset_time = time_from_start + joiner_offset as f64;
            result.push((-offset_time, joiner_val));
        }

        result.push((-time_from_start, data.mem_data.0));

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

        //Insert joiner points
        for &(joiner_offset, joiner_val) in &data.swap_data.1 {
            let offset_time = time_from_start + joiner_offset as f64;
            result.push((-offset_time, joiner_val));
        }

        result.push((-time_from_start, data.swap_data.0));

        if *time == current_time {
            break;
        }
    }

    result
}

pub fn convert_mem_labels(current_data: &data_farmer::DataCollection) -> (String, String) {
    let mem_label = if current_data.memory_harvest.mem_total_in_mb == 0 {
        "".to_string()
    } else {
        "RAM:".to_string()
            + &format!(
                "{:3.0}%",
                (current_data.memory_harvest.mem_used_in_mb as f64 * 100.0
                    / current_data.memory_harvest.mem_total_in_mb as f64)
            )
            + &format!(
                "   {:.1}GB/{:.1}GB",
                current_data.memory_harvest.mem_used_in_mb as f64 / 1024.0,
                (current_data.memory_harvest.mem_total_in_mb as f64 / 1024.0)
            )
    };

    let swap_label = if current_data.swap_harvest.mem_total_in_mb == 0 {
        "".to_string()
    } else {
        "SWP:".to_string()
            + &format!(
                "{:3.0}%",
                (current_data.swap_harvest.mem_used_in_mb as f64 * 100.0
                    / current_data.swap_harvest.mem_total_in_mb as f64)
            )
            + &format!(
                "   {:.1}GB/{:.1}GB",
                current_data.swap_harvest.mem_used_in_mb as f64 / 1024.0,
                (current_data.swap_harvest.mem_total_in_mb as f64 / 1024.0)
            )
    };

    (mem_label, swap_label)
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

        //Insert joiner points
        for &(joiner_offset, joiner_val) in &data.rx_data.1 {
            let offset_time = time_from_start + joiner_offset as f64;
            rx.push((-offset_time, joiner_val));
        }

        for &(joiner_offset, joiner_val) in &data.tx_data.1 {
            let offset_time = time_from_start + joiner_offset as f64;
            tx.push((-offset_time, joiner_val));
        }

        rx.push((-time_from_start, data.rx_data.0));
        tx.push((-time_from_start, data.tx_data.0));

        if *time == current_time {
            break;
        }
    }

    (rx, tx)
}

pub fn convert_network_data_points(
    current_data: &data_farmer::DataCollection, is_frozen: bool,
) -> ConvertedNetworkData {
    let (rx, tx) = get_rx_tx_data_points(current_data, is_frozen);

    let total_rx_converted_result: (f64, String);
    let rx_converted_result: (f64, String);
    let total_tx_converted_result: (f64, String);
    let tx_converted_result: (f64, String);

    rx_converted_result = get_exact_byte_values(current_data.network_harvest.rx, false);
    total_rx_converted_result = get_exact_byte_values(current_data.network_harvest.total_rx, false);
    let rx_display = format!("{:.*}{}", 1, rx_converted_result.0, rx_converted_result.1);
    let total_rx_display = format!(
        "{:.*}{}",
        1, total_rx_converted_result.0, total_rx_converted_result.1
    );

    tx_converted_result = get_exact_byte_values(current_data.network_harvest.tx, false);
    total_tx_converted_result = get_exact_byte_values(current_data.network_harvest.total_tx, false);
    let tx_display = format!("{:.*}{}", 1, tx_converted_result.0, tx_converted_result.1);
    let total_tx_display = format!(
        "{:.*}{}",
        1, total_tx_converted_result.0, total_tx_converted_result.1
    );

    ConvertedNetworkData {
        rx,
        tx,
        rx_display,
        tx_display,
        total_rx_display,
        total_tx_display,
    }
}

pub fn convert_process_data(
    current_data: &data_farmer::DataCollection,
) -> (HashMap<u32, ProcessHarvest>, Vec<ConvertedProcessData>) {
    let mut single_list: HashMap<u32, ProcessHarvest> = HashMap::new();

    // cpu, mem, pids
    let mut grouped_hashmap: HashMap<String, (u32, f64, f64, Vec<u32>)> =
        std::collections::HashMap::new();

    // Go through every single process in the list... and build a hashmap + single list
    for process in &(current_data).process_harvest {
        let entry = grouped_hashmap.entry(process.name.clone()).or_insert((
            process.pid,
            0.0,
            0.0,
            Vec::new(),
        ));

        (*entry).1 += process.cpu_usage_percent;
        (*entry).2 += process.mem_usage_percent;
        (*entry).3.push(process.pid);

        single_list.insert(process.pid, process.clone());
    }

    let grouped_list: Vec<ConvertedProcessData> = grouped_hashmap
        .iter()
        .map(|(name, process_details)| {
            let p = process_details.clone();
            ConvertedProcessData {
                pid: p.0,
                name: name.to_string(),
                cpu_usage: p.1,
                mem_usage: p.2,
                group_pids: p.3,
            }
        })
        .collect::<Vec<_>>();

    (single_list, grouped_list)
}
