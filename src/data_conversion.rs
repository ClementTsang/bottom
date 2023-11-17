//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use kstring::KString;

use crate::app::data_harvester::memory::MemHarvest;
use crate::app::{
    data_farmer::DataCollection,
    data_harvester::{cpu::CpuDataType, temperature::TemperatureType},
    AxisScaling,
};
use crate::components::tui_widget::time_chart::Point;
use crate::utils::data_prefixes::*;
use crate::utils::data_units::DataUnit;
use crate::utils::gen_util::*;
use crate::widgets::{DiskWidgetData, TempWidgetData};

#[derive(Debug, Default)]
pub enum BatteryDuration {
    ToEmpty(i64),
    ToFull(i64),
    Empty,
    Full,
    #[default]
    Unknown,
}

#[derive(Default, Debug)]
pub struct ConvertedBatteryData {
    pub charge_percentage: f64,
    pub watt_consumption: String,
    pub battery_duration: BatteryDuration,
    pub health: String,
    pub state: String,
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

#[derive(Clone, Debug)]
pub enum CpuWidgetData {
    All,
    Entry {
        data_type: CpuDataType,
        /// A point here represents time (x) and value (y).
        data: Vec<Point>,
        last_entry: f64,
    },
}

#[derive(Default)]
pub struct ConvertedData {
    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: String,
    pub total_tx_display: String,
    pub network_data_rx: Vec<Point>,
    pub network_data_tx: Vec<Point>,

    pub mem_labels: Option<(String, String)>,
    #[cfg(not(target_os = "windows"))]
    pub cache_labels: Option<(String, String)>,
    pub swap_labels: Option<(String, String)>,

    pub mem_data: Vec<Point>, /* TODO: Switch this and all data points over to a better data structure... */
    #[cfg(not(target_os = "windows"))]
    pub cache_data: Vec<Point>,
    pub swap_data: Vec<Point>,

    #[cfg(feature = "zfs")]
    pub arc_labels: Option<(String, String)>,
    #[cfg(feature = "zfs")]
    pub arc_data: Vec<Point>,

    #[cfg(feature = "gpu")]
    pub gpu_data: Option<Vec<ConvertedGpuData>>,

    pub load_avg_data: [f32; 3],
    pub cpu_data: Vec<CpuWidgetData>,
    pub battery_data: Vec<ConvertedBatteryData>,
    pub disk_data: Vec<DiskWidgetData>,
    pub temp_data: Vec<TempWidgetData>,
}

impl ConvertedData {
    // TODO: Can probably heavily reduce this step to avoid clones.
    pub fn ingest_disk_data(&mut self, data: &DataCollection) {
        self.disk_data.clear();

        data.disk_harvest
            .iter()
            .zip(&data.io_labels)
            .for_each(|(disk, (io_read, io_write))| {
                // Because this sometimes does *not* equal to disk.total.
                let summed_total_bytes = match (disk.used_space, disk.free_space) {
                    (Some(used), Some(free)) => Some(used + free),
                    _ => None,
                };

                self.disk_data.push(DiskWidgetData {
                    name: KString::from_ref(&disk.name),
                    mount_point: KString::from_ref(&disk.mount_point),
                    free_bytes: disk.free_space,
                    used_bytes: disk.used_space,
                    total_bytes: disk.total_space,
                    summed_total_bytes,
                    io_read: io_read.into(),
                    io_write: io_write.into(),
                });
            });

        self.disk_data.shrink_to_fit();
    }

    pub fn ingest_temp_data(&mut self, data: &DataCollection, temperature_type: TemperatureType) {
        self.temp_data.clear();

        data.temp_harvest.iter().for_each(|temp_harvest| {
            self.temp_data.push(TempWidgetData {
                sensor: KString::from_ref(&temp_harvest.name),
                temperature_value: temp_harvest.temperature.ceil() as u64,
                temperature_type,
            });
        });

        self.temp_data.shrink_to_fit();
    }

    pub fn ingest_cpu_data(&mut self, current_data: &DataCollection) {
        let current_time = current_data.current_instant;

        // (Re-)initialize the vector if the lengths don't match...
        if let Some((_time, data)) = &current_data.timed_data_vec.last() {
            if data.cpu_data.len() + 1 != self.cpu_data.len() {
                self.cpu_data = Vec::with_capacity(data.cpu_data.len() + 1);
                self.cpu_data.push(CpuWidgetData::All);
                self.cpu_data.extend(
                    data.cpu_data
                        .iter()
                        .zip(&current_data.cpu_harvest)
                        .map(|(cpu_usage, data)| CpuWidgetData::Entry {
                            data_type: data.data_type,
                            data: vec![],
                            last_entry: *cpu_usage,
                        })
                        .collect::<Vec<CpuWidgetData>>(),
                );
            } else {
                self.cpu_data
                    .iter_mut()
                    .skip(1)
                    .zip(&data.cpu_data)
                    .for_each(|(mut cpu, cpu_usage)| match &mut cpu {
                        CpuWidgetData::All => unreachable!(),
                        CpuWidgetData::Entry {
                            data_type: _,
                            data,
                            last_entry,
                        } => {
                            // A bit faster to just update all the times, so we just clear the vector.
                            data.clear();
                            *last_entry = *cpu_usage;
                        }
                    });
            }
        }

        // TODO: [Opt] Can probably avoid data deduplication - store the shift + data + original once.
        // Now push all the data.
        for (itx, mut cpu) in &mut self.cpu_data.iter_mut().skip(1).enumerate() {
            match &mut cpu {
                CpuWidgetData::All => unreachable!(),
                CpuWidgetData::Entry {
                    data_type: _,
                    data,
                    last_entry: _,
                } => {
                    for (time, timed_data) in &current_data.timed_data_vec {
                        let time_start: f64 =
                            (current_time.duration_since(*time).as_millis() as f64).floor();

                        if let Some(val) = timed_data.cpu_data.get(itx) {
                            data.push((-time_start, *val));
                        }

                        if *time == current_time {
                            break;
                        }
                    }

                    data.shrink_to_fit();
                }
            }
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

#[cfg(not(target_os = "windows"))]
pub fn convert_cache_data_points(current_data: &DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = current_data.current_instant;

    for (time, data) in &current_data.timed_data_vec {
        if let Some(cache_data) = data.cache_data {
            let time_from_start: f64 =
                (current_time.duration_since(*time).as_millis() as f64).floor();
            result.push((-time_from_start, cache_data));
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

/// Returns the most appropriate binary prefix unit type (e.g. kibibyte) and denominator for the given amount of bytes.
///
/// The expected usage is to divide out the given value with the returned denominator in order to be able to use it
/// with the returned binary unit (e.g. divide 3000 bytes by 1024 to have a value in KiB).
fn get_mem_binary_unit_and_denominator(bytes: u64) -> (&'static str, f64) {
    if bytes < KIBI_LIMIT {
        // Stick with bytes if under a kibibyte.
        ("B", 1.0)
    } else if bytes < MEBI_LIMIT {
        ("KiB", KIBI_LIMIT_F64)
    } else if bytes < GIBI_LIMIT {
        ("MiB", MEBI_LIMIT_F64)
    } else if bytes < TEBI_LIMIT {
        ("GiB", GIBI_LIMIT_F64)
    } else {
        // Otherwise just use tebibytes, which is probably safe for most use cases.
        ("TiB", TEBI_LIMIT_F64)
    }
}

/// Returns the unit type and denominator for given total amount of memory in kibibytes.
pub fn convert_mem_label(harvest: &MemHarvest) -> Option<(String, String)> {
    if harvest.total_bytes > 0 {
        Some((format!("{:3.0}%", harvest.use_percent.unwrap_or(0.0)), {
            let (unit, denominator) = get_mem_binary_unit_and_denominator(harvest.total_bytes);

            format!(
                "   {:.1}{}/{:.1}{}",
                harvest.used_bytes as f64 / denominator,
                unit,
                (harvest.total_bytes as f64 / denominator),
                unit
            )
        }))
    } else {
        None
    }
}

pub fn get_rx_tx_data_points(
    data: &DataCollection, scale_type: &AxisScaling, unit_type: &DataUnit, use_binary_prefix: bool,
) -> (Vec<Point>, Vec<Point>) {
    let mut rx: Vec<Point> = Vec::new();
    let mut tx: Vec<Point> = Vec::new();

    let current_time = data.current_instant;

    for (time, data) in &data.timed_data_vec {
        let time_from_start: f64 = (current_time.duration_since(*time).as_millis() as f64).floor();

        let (rx_data, tx_data) = match scale_type {
            AxisScaling::Log => {
                if use_binary_prefix {
                    match unit_type {
                        DataUnit::Byte => {
                            // As dividing by 8 is equal to subtracting 4 in base 2!
                            ((data.rx_data).log2() - 4.0, (data.tx_data).log2() - 4.0)
                        }
                        DataUnit::Bit => ((data.rx_data).log2(), (data.tx_data).log2()),
                    }
                } else {
                    match unit_type {
                        DataUnit::Byte => {
                            ((data.rx_data / 8.0).log10(), (data.tx_data / 8.0).log10())
                        }
                        DataUnit::Bit => ((data.rx_data).log10(), (data.tx_data).log10()),
                    }
                }
            }
            AxisScaling::Linear => match unit_type {
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
    data: &DataCollection, need_four_points: bool, scale_type: &AxisScaling, unit_type: &DataUnit,
    use_binary_prefix: bool,
) -> ConvertedNetworkData {
    let (rx, tx) = get_rx_tx_data_points(data, scale_type, unit_type, use_binary_prefix);

    let unit = match unit_type {
        DataUnit::Byte => "B/s",
        DataUnit::Bit => "b/s",
    };

    let (rx_data, tx_data, total_rx_data, total_tx_data) = match unit_type {
        DataUnit::Byte => (
            data.network_harvest.rx / 8,
            data.network_harvest.tx / 8,
            data.network_harvest.total_rx / 8,
            data.network_harvest.total_tx / 8,
        ),
        DataUnit::Bit => (
            data.network_harvest.rx,
            data.network_harvest.tx,
            data.network_harvest.total_rx / 8, // We always make this bytes...
            data.network_harvest.total_tx / 8,
        ),
    };

    let (rx_converted_result, total_rx_converted_result): ((f64, String), (f64, &'static str)) =
        if use_binary_prefix {
            (
                get_binary_prefix(rx_data, unit), /* If this isn't obvious why there's two functions, one you can configure the unit, the other is always bytes */
                get_binary_bytes(total_rx_data),
            )
        } else {
            (
                get_decimal_prefix(rx_data, unit),
                get_decimal_bytes(total_rx_data),
            )
        };

    let (tx_converted_result, total_tx_converted_result): ((f64, String), (f64, &'static str)) =
        if use_binary_prefix {
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
            if use_binary_prefix {
                format!("{:.1}{:3}", rx_converted_result.0, rx_converted_result.1)
            } else {
                format!("{:.1}{:2}", rx_converted_result.0, rx_converted_result.1)
            },
            if use_binary_prefix {
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
            if use_binary_prefix {
                format!("{:.1}{:3}", tx_converted_result.0, tx_converted_result.1)
            } else {
                format!("{:.1}{:2}", tx_converted_result.0, tx_converted_result.1)
            },
            if use_binary_prefix {
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

/// Returns a string given a value that is converted to the closest SI-variant.
/// If the value is greater than a giga-X, then it will return a decimal place.
pub fn dec_bytes_string(value: u64) -> String {
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
        .map(|battery_harvest| ConvertedBatteryData {
            charge_percentage: battery_harvest.charge_percent,
            watt_consumption: format!("{:.2}W", battery_harvest.power_consumption_rate_watts),
            battery_duration: if let Some(secs) = battery_harvest.secs_until_empty {
                BatteryDuration::ToEmpty(secs)
            } else if let Some(secs) = battery_harvest.secs_until_full {
                BatteryDuration::ToFull(secs)
            } else {
                match battery_harvest.state {
                    starship_battery::State::Empty => BatteryDuration::Empty,
                    starship_battery::State::Full => BatteryDuration::Full,
                    _ => BatteryDuration::Unknown,
                }
            },
            health: format!("{:.2}%", battery_harvest.health_percent),
            state: {
                let mut s = battery_harvest.state.to_string();
                if !s.is_empty() {
                    s[0..1].make_ascii_uppercase();
                }
                s
            },
        })
        .collect()
}

#[cfg(feature = "zfs")]
pub fn convert_arc_labels(current_data: &DataCollection) -> Option<(String, String)> {
    if current_data.arc_harvest.total_bytes > 0 {
        Some((
            format!(
                "{:3.0}%",
                current_data.arc_harvest.use_percent.unwrap_or(0.0)
            ),
            {
                let (unit, denominator) =
                    get_mem_binary_unit_and_denominator(current_data.arc_harvest.total_bytes);

                format!(
                    "   {:.1}{unit}/{:.1}{unit}",
                    current_data.arc_harvest.used_bytes as f64 / denominator,
                    (current_data.arc_harvest.total_bytes as f64 / denominator),
                )
            },
        ))
    } else {
        None
    }
}

#[cfg(feature = "zfs")]
pub fn convert_arc_data_points(current_data: &DataCollection) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = current_data.current_instant;

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
pub fn convert_gpu_data(current_data: &DataCollection) -> Option<Vec<ConvertedGpuData>> {
    let current_time = current_data.current_instant;

    // convert points
    let mut point_vec: Vec<Vec<Point>> = Vec::with_capacity(current_data.gpu_harvest.len());
    for (time, data) in &current_data.timed_data_vec {
        data.gpu_data.iter().enumerate().for_each(|(index, point)| {
            if let Some(data_point) = point {
                let time_from_start: f64 =
                    (current_time.duration_since(*time).as_millis() as f64).floor();
                if let Some(point_slot) = point_vec.get_mut(index) {
                    point_slot.push((-time_from_start, *data_point));
                } else {
                    point_vec.push(vec![(-time_from_start, *data_point)]);
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
        .zip(point_vec)
        .map(|(gpu, points)| {
            let short_name = {
                let last_words = gpu.0.split_whitespace().rev().take(2).collect::<Vec<_>>();
                let short_name = format!("{} {}", last_words[1], last_words[0]);
                short_name
            };

            ConvertedGpuData {
                name: short_name,
                points,
                mem_percent: format!("{:3.0}%", gpu.1.use_percent.unwrap_or(0.0)),
                mem_total: {
                    let (unit, denominator) =
                        get_mem_binary_unit_and_denominator(gpu.1.total_bytes);

                    format!(
                        "   {:.1}{unit}/{:.1}{unit}",
                        gpu.1.used_bytes as f64 / denominator,
                        (gpu.1.total_bytes as f64 / denominator),
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
