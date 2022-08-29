//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use crate::components::text_table::CellContent;
use crate::components::time_graph::Point;
use crate::{app::AxisScaling, units::data_units::DataUnit, Pid};
use crate::{
    app::{data_farmer, data_harvester, App},
    utils::gen_util::*,
};

use concat_string::concat_string;
use fxhash::FxHashMap;

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
pub struct TableData {
    pub data: Vec<TableRow>,
    pub col_widths: Vec<usize>,
}

#[derive(Debug)]
pub enum TableRow {
    Raw(Vec<CellContent>),
    Styled(Vec<CellContent>, tui::style::Style),
}

impl TableRow {
    pub fn row(&self) -> &[CellContent] {
        match self {
            TableRow::Raw(data) => data,
            TableRow::Styled(data, _) => data,
        }
    }
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

#[derive(Clone, Default, Debug)]
pub struct ConvertedCpuData {
    pub cpu_name: String,
    pub short_cpu_name: String,
    /// Tuple is time, value
    pub cpu_data: Vec<Point>,
    /// Represents the value displayed on the legend.
    pub legend_value: String,
}

#[derive(Default)]
pub struct ConvertedData {
    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: String,
    pub total_tx_display: String,
    pub network_data_rx: Vec<Point>,
    pub network_data_tx: Vec<Point>,
    pub disk_data: TableData,
    pub temp_sensor_data: TableData,

    /// A mapping from a process name to any PID with that name.
    pub process_name_pid_map: FxHashMap<String, Vec<Pid>>,

    /// A mapping from a process command to any PID with that name.
    pub process_cmd_pid_map: FxHashMap<String, Vec<Pid>>,

    pub mem_labels: Option<(String, String)>,
    pub swap_labels: Option<(String, String)>,
    #[cfg(feature = "zfs")]
    pub arc_labels: Option<(String, String)>,
    pub mem_data: Vec<Point>, // TODO: Switch this and all data points over to a better data structure...
    pub swap_data: Vec<Point>,
    #[cfg(feature = "zfs")]
    pub arc_data: Vec<Point>,
    pub load_avg_data: [f32; 3],
    pub cpu_data: Vec<ConvertedCpuData>,
    pub battery_data: Vec<ConvertedBatteryData>,
    #[cfg(feature = "gpu")]
    pub gpu_data: Option<Vec<ConvertedGpuData>>,
}

pub fn convert_temp_row(app: &App) -> TableData {
    let current_data = &app.data_collection;
    let temp_type = &app.app_config_fields.temperature_type;
    let mut col_widths = vec![0; 2];

    let mut sensor_vector: Vec<TableRow> = current_data
        .temp_harvest
        .iter()
        .map(|temp_harvest| {
            let row = vec![
                CellContent::Simple(temp_harvest.name.clone().into()),
                CellContent::Simple(
                    concat_string!(
                        (temp_harvest.temperature.ceil() as u64).to_string(),
                        match temp_type {
                            data_harvester::temperature::TemperatureType::Celsius => "°C",
                            data_harvester::temperature::TemperatureType::Kelvin => "K",
                            data_harvester::temperature::TemperatureType::Fahrenheit => "°F",
                        }
                    )
                    .into(),
                ),
            ];

            col_widths.iter_mut().zip(&row).for_each(|(curr, r)| {
                *curr = std::cmp::max(*curr, r.len());
            });

            TableRow::Raw(row)
        })
        .collect();

    if sensor_vector.is_empty() {
        sensor_vector.push(TableRow::Raw(vec![
            CellContent::Simple("No Sensors Found".into()),
            CellContent::Simple("".into()),
        ]));
    }

    TableData {
        data: sensor_vector,
        col_widths,
    }
}

pub fn convert_disk_row(current_data: &data_farmer::DataCollection) -> TableData {
    let mut disk_vector: Vec<TableRow> = Vec::new();
    let mut col_widths = vec![0; 8];

    current_data
        .disk_harvest
        .iter()
        .zip(&current_data.io_labels)
        .for_each(|(disk, (io_read, io_write))| {
            let free_space_fmt = if let Some(free_space) = disk.free_space {
                let converted_free_space = get_decimal_bytes(free_space);
                format!("{:.*}{}", 0, converted_free_space.0, converted_free_space.1).into()
            } else {
                "N/A".into()
            };
            let total_space_fmt = if let Some(total_space) = disk.total_space {
                let converted_total_space = get_decimal_bytes(total_space);
                format!(
                    "{:.*}{}",
                    0, converted_total_space.0, converted_total_space.1
                )
                .into()
            } else {
                "N/A".into()
            };

            let usage_fmt = if let (Some(used_space), Some(total_space)) =
                (disk.used_space, disk.total_space)
            {
                format!("{:.0}%", used_space as f64 / total_space as f64 * 100_f64).into()
            } else {
                "N/A".into()
            };

            let row = vec![
                CellContent::Simple(disk.name.clone().into()),
                CellContent::Simple(disk.mount_point.clone().into()),
                CellContent::Simple(usage_fmt),
                CellContent::Simple(free_space_fmt),
                CellContent::Simple(total_space_fmt),
                CellContent::Simple(io_read.clone().into()),
                CellContent::Simple(io_write.clone().into()),
            ];
            col_widths.iter_mut().zip(&row).for_each(|(curr, r)| {
                *curr = std::cmp::max(*curr, r.len());
            });
            disk_vector.push(TableRow::Raw(row));
        });

    if disk_vector.is_empty() {
        disk_vector.push(TableRow::Raw(vec![
            CellContent::Simple("No Disks Found".into()),
            CellContent::Simple("".into()),
        ]));
    }

    TableData {
        data: disk_vector,
        col_widths,
    }
}

pub fn convert_cpu_data_points(
    current_data: &data_farmer::DataCollection, existing_cpu_data: &mut Vec<ConvertedCpuData>,
) {
    let current_time = if let Some(frozen_instant) = current_data.frozen_instant {
        frozen_instant
    } else {
        current_data.current_instant
    };

    // Initialize cpu_data_vector if the lengths don't match...
    if let Some((_time, data)) = &current_data.timed_data_vec.last() {
        if data.cpu_data.len() + 1 != existing_cpu_data.len() {
            *existing_cpu_data = vec![ConvertedCpuData {
                cpu_name: "All".to_string(),
                short_cpu_name: "".to_string(),
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
    }

    for (time, data) in &current_data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();

        for (itx, cpu) in data.cpu_data.iter().enumerate() {
            if let Some(cpu_data) = existing_cpu_data.get_mut(itx + 1) {
                cpu_data.cpu_data.push((-time_from_start, *cpu));
            }
        }

        if *time == current_time {
            break;
        }
    }
}

pub fn convert_mem_data_points(current_data: &data_farmer::DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = if let Some(frozen_instant) = current_data.frozen_instant {
        frozen_instant
    } else {
        current_data.current_instant
    };

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

pub fn convert_swap_data_points(current_data: &data_farmer::DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = if let Some(frozen_instant) = current_data.frozen_instant {
        frozen_instant
    } else {
        current_data.current_instant
    };

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
    current_data: &data_farmer::DataCollection,
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
    current_data: &data_farmer::DataCollection, network_scale_type: &AxisScaling,
    network_unit_type: &DataUnit, network_use_binary_prefix: bool,
) -> (Vec<Point>, Vec<Point>) {
    let mut rx: Vec<Point> = Vec::new();
    let mut tx: Vec<Point> = Vec::new();

    let current_time = if let Some(frozen_instant) = current_data.frozen_instant {
        frozen_instant
    } else {
        current_data.current_instant
    };

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
    current_data: &data_farmer::DataCollection, need_four_points: bool,
    network_scale_type: &AxisScaling, network_unit_type: &DataUnit,
    network_use_binary_prefix: bool,
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

/// Returns a string given a value that is converted to the closest binary variant.
/// If the value is greater than a gibibyte, then it will return a decimal place.
pub fn binary_byte_string(value: u64) -> String {
    let converted_values = get_binary_bytes(value);
    if value >= GIBI_LIMIT {
        format!("{:.*}{}", 1, converted_values.0, converted_values.1)
    } else {
        format!("{:.*}{}", 0, converted_values.0, converted_values.1)
    }
}

/// Returns a string given a value that is converted to the closest SI-variant.
/// If the value is greater than a giga-X, then it will return a decimal place.
pub fn dec_bytes_per_string(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.*}{}", 1, converted_values.0, converted_values.1)
    } else {
        format!("{:.*}{}", 0, converted_values.0, converted_values.1)
    }
}

/// Returns a string given a value that is converted to the closest SI-variant, per second.
/// If the value is greater than a giga-X, then it will return a decimal place.
pub fn dec_bytes_per_second_string(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.*}{}/s", 1, converted_values.0, converted_values.1)
    } else {
        format!("{:.*}{}/s", 0, converted_values.0, converted_values.1)
    }
}

#[cfg(feature = "battery")]
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
                let time = time::Duration::seconds(secs_till_empty);
                let num_minutes = time.whole_minutes() - time.whole_hours() * 60;
                let num_seconds = time.whole_seconds() - time.whole_minutes() * 60;
                Some(format!(
                    "{} hour{}, {} minute{}, {} second{}",
                    time.whole_hours(),
                    if time.whole_hours() == 1 { "" } else { "s" },
                    num_minutes,
                    if num_minutes == 1 { "" } else { "s" },
                    num_seconds,
                    if num_seconds == 1 { "" } else { "s" },
                ))
            } else {
                None
            },
            duration_until_full: if let Some(secs_till_full) = battery_harvest.secs_until_full {
                let time = time::Duration::seconds(secs_till_full);
                let num_minutes = time.whole_minutes() - time.whole_hours() * 60;
                let num_seconds = time.whole_seconds() - time.whole_minutes() * 60;
                Some(format!(
                    "{} hour{}, {} minute{}, {} second{}",
                    time.whole_hours(),
                    if time.whole_hours() == 1 { "" } else { "s" },
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

#[cfg(feature = "zfs")]
pub fn convert_arc_labels(current_data: &data_farmer::DataCollection) -> Option<(String, String)> {
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

    if current_data.arc_harvest.mem_total_in_kib > 0 {
        Some((
            format!(
                "{:3.0}%",
                current_data.arc_harvest.use_percent.unwrap_or(0.0)
            ),
            {
                let (unit, denominator) = return_unit_and_denominator_for_mem_kib(
                    current_data.arc_harvest.mem_total_in_kib,
                );

                format!(
                    "   {:.1}{}/{:.1}{}",
                    current_data.arc_harvest.mem_used_in_kib as f64 / denominator,
                    unit,
                    (current_data.arc_harvest.mem_total_in_kib as f64 / denominator),
                    unit
                )
            },
        ))
    } else {
        None
    }
}

#[cfg(feature = "zfs")]
pub fn convert_arc_data_points(current_data: &data_farmer::DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = if let Some(frozen_instant) = current_data.frozen_instant {
        frozen_instant
    } else {
        current_data.current_instant
    };

    for (time, data) in &current_data.timed_data_vec {
        if let Some(arc_data) = data.arc_data {
            let time_from_start: f64 =
                (current_time.duration_since(*time).as_millis() as f64).floor();
            result.push((-time_from_start, arc_data));
            if *time == current_time {
                break;
            }
        }
    }

    result
}

#[cfg(feature = "gpu")]
#[derive(Default, Debug)]
pub struct ConvertedGpuData {
    pub name: String,
    pub mem_total: String,
    pub mem_percent: String,
    pub points: Vec<Point>,
}

#[cfg(feature = "gpu")]
pub fn convert_gpu_data(
    current_data: &data_farmer::DataCollection,
) -> Option<Vec<ConvertedGpuData>> {
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

    let current_time = if let Some(frozen_instant) = current_data.frozen_instant {
        frozen_instant
    } else {
        current_data.current_instant
    };

    // convert points
    let mut point_vec: Vec<Vec<Point>> = Vec::with_capacity(current_data.timed_data_vec.len());
    for (time, data) in &current_data.timed_data_vec {
        data.gpu_data.iter().enumerate().for_each(|(index, point)| {
            if let Some(data_point) = point {
                let time_from_start: f64 =
                    (current_time.duration_since(*time).as_millis() as f64).floor();
                if let Some(point_slot) = point_vec.get_mut(index) {
                    point_slot.push((-time_from_start, *data_point));
                } else {
                    let mut index_vec: Vec<Point> = Vec::with_capacity(data.gpu_data.len());
                    index_vec.push((-time_from_start, *data_point));
                    point_vec.push(index_vec);
                }
            }
        });

        if *time == current_time {
            break;
        }
    }

    // convert labels
    let results = current_data
        .gpu_harvest
        .iter()
        .zip(point_vec.iter())
        .map(|(gpu, points)| {
            let short_name = {
                let last_words = gpu.0.split_whitespace().rev().take(2).collect::<Vec<_>>();
                let short_name = format!("{} {}", last_words[1], last_words[0]);
                short_name
            };

            ConvertedGpuData {
                name: short_name,
                points: points.to_owned(),
                mem_percent: format!("{:3.0}%", gpu.1.use_percent.unwrap_or(0.0)),
                mem_total: {
                    let (unit, denominator) =
                        return_unit_and_denominator_for_mem_kib(gpu.1.mem_total_in_kib);

                    format!(
                        "   {:.1}{}/{:.1}{}",
                        gpu.1.mem_used_in_kib as f64 / denominator,
                        unit,
                        (gpu.1.mem_total_in_kib as f64 / denominator),
                        unit
                    )
                },
            }
        })
        .collect::<Vec<ConvertedGpuData>>();

    if !results.is_empty() {
        Some(results)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_binary_byte_string() {
        assert_eq!(binary_byte_string(0), "0B".to_string());
        assert_eq!(binary_byte_string(1), "1B".to_string());
        assert_eq!(binary_byte_string(1000), "1000B".to_string());
        assert_eq!(binary_byte_string(1023), "1023B".to_string());
        assert_eq!(binary_byte_string(KIBI_LIMIT), "1KiB".to_string());
        assert_eq!(binary_byte_string(KIBI_LIMIT + 1), "1KiB".to_string());
        assert_eq!(binary_byte_string(MEBI_LIMIT), "1MiB".to_string());
        assert_eq!(binary_byte_string(GIBI_LIMIT), "1.0GiB".to_string());
        assert_eq!(binary_byte_string(2 * GIBI_LIMIT), "2.0GiB".to_string());
        assert_eq!(
            binary_byte_string((2.5 * GIBI_LIMIT as f64) as u64),
            "2.5GiB".to_string()
        );
        assert_eq!(
            binary_byte_string((10.34 * TEBI_LIMIT as f64) as u64),
            "10.3TiB".to_string()
        );
        assert_eq!(
            binary_byte_string((10.36 * TEBI_LIMIT as f64) as u64),
            "10.4TiB".to_string()
        );
    }

    #[test]
    fn test_dec_bytes_per_second_string() {
        assert_eq!(dec_bytes_per_second_string(0), "0B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(1), "1B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(900), "900B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(999), "999B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(KILO_LIMIT), "1KB/s".to_string());
        assert_eq!(
            dec_bytes_per_second_string(KILO_LIMIT + 1),
            "1KB/s".to_string()
        );
        assert_eq!(dec_bytes_per_second_string(KIBI_LIMIT), "1KB/s".to_string());
        assert_eq!(dec_bytes_per_second_string(MEGA_LIMIT), "1MB/s".to_string());
        assert_eq!(
            dec_bytes_per_second_string(GIGA_LIMIT),
            "1.0GB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string(2 * GIGA_LIMIT),
            "2.0GB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string((2.5 * GIGA_LIMIT as f64) as u64),
            "2.5GB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string((10.34 * TERA_LIMIT as f64) as u64),
            "10.3TB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string((10.36 * TERA_LIMIT as f64) as u64),
            "10.4TB/s".to_string()
        );
    }
}
