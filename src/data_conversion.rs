//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

// TODO: Split this up!

use std::borrow::Cow;

use crate::{
    app::data::CollectedData,
    canvas::components::time_chart::Point,
    data_collection::{cpu::CpuDataType, memory::MemHarvest, temperature::TemperatureType},
    utils::data_prefixes::*,
    widgets::{DiskWidgetData, TempWidgetData},
};

#[derive(Clone, Debug)]
pub enum CpuWidgetData {
    All,
    Entry {
        data_type: CpuDataType,
        data: Vec<Point>,
        last_entry: f64,
    },
}

#[derive(Default)]
pub struct ConvertedData {
    pub mem_labels: Option<(String, String)>,
    #[cfg(not(target_os = "windows"))]
    pub cache_labels: Option<(String, String)>,
    pub swap_labels: Option<(String, String)>,

    // TODO: Switch this and all data points over to a better data structure.
    //
    // We can dedupe the f64 for time by storing it alongside this data structure.
    // We can also just store everything via an references and iterators to avoid
    // duplicating data, I guess.
    pub mem_data: Vec<Point>,
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

    pub disk_data: Vec<DiskWidgetData>,
    pub temp_data: Vec<TempWidgetData>,
}

impl ConvertedData {
    // TODO: Can probably heavily reduce this step to avoid clones.
    pub fn convert_disk_data(&mut self, data: &CollectedData) {
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
                    name: Cow::Owned(disk.name.to_string()),
                    mount_point: Cow::Owned(disk.mount_point.to_string()),
                    free_bytes: disk.free_space,
                    used_bytes: disk.used_space,
                    total_bytes: disk.total_space,
                    summed_total_bytes,
                    io_read: Cow::Owned(io_read.to_string()),
                    io_write: Cow::Owned(io_write.to_string()),
                });
            });

        self.disk_data.shrink_to_fit();
    }

    pub fn convert_temp_data(&mut self, data: &CollectedData, temperature_type: TemperatureType) {
        self.temp_data.clear();

        data.temp_harvest.iter().for_each(|temp_harvest| {
            self.temp_data.push(TempWidgetData {
                sensor: Cow::Owned(temp_harvest.name.to_string()),
                temperature_value: temp_harvest.temperature.map(|temp| temp.ceil() as u64),
                temperature_type,
            });
        });

        self.temp_data.shrink_to_fit();
    }

    pub fn convert_cpu_data(&mut self, current_data: &CollectedData) {
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
                            // A bit faster to just update all the times, so we just clear the
                            // vector.
                            data.clear();
                            *last_entry = *cpu_usage;
                        }
                    });
            }
        }

        // TODO: [Opt] Can probably avoid data deduplication - store the shift + data +
        // original once. Now push all the data.
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

pub fn convert_mem_data_points(data: &CollectedData) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = data.current_instant;

    for (time, data) in &data.timed_data_vec {
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
pub fn convert_cache_data_points(data: &CollectedData) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = data.current_instant;

    for (time, data) in &data.timed_data_vec {
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

pub fn convert_swap_data_points(data: &CollectedData) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    let current_time = data.current_instant;

    for (time, data) in &data.timed_data_vec {
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

/// Returns the most appropriate binary prefix unit type (e.g. kibibyte) and
/// denominator for the given amount of bytes.
///
/// The expected usage is to divide out the given value with the returned
/// denominator in order to be able to use it with the returned binary unit
/// (e.g. divide 3000 bytes by 1024 to have a value in KiB).
#[inline]
fn get_binary_unit_and_denominator(bytes: u64) -> (&'static str, f64) {
    match bytes {
        b if b < KIBI_LIMIT => ("B", 1.0),
        b if b < MEBI_LIMIT => ("KiB", KIBI_LIMIT_F64),
        b if b < GIBI_LIMIT => ("MiB", MEBI_LIMIT_F64),
        b if b < TEBI_LIMIT => ("GiB", GIBI_LIMIT_F64),
        _ => ("TiB", TEBI_LIMIT_F64),
    }
}

/// Returns the unit type and denominator for given total amount of memory in
/// kibibytes.
pub fn convert_mem_label(harvest: &MemHarvest) -> Option<(String, String)> {
    (harvest.total_bytes > 0).then(|| {
        let percentage = harvest.used_bytes as f64 / harvest.total_bytes as f64 * 100.0;
        (format!("{percentage:3.0}%"), {
            let (unit, denominator) = get_binary_unit_and_denominator(harvest.total_bytes);

            format!(
                "   {:.1}{}/{:.1}{}",
                harvest.used_bytes as f64 / denominator,
                unit,
                (harvest.total_bytes as f64 / denominator),
                unit
            )
        })
    })
}

/// Returns a string given a value that is converted to the closest binary
/// variant. If the value is greater than a gibibyte, then it will return a
/// decimal place.
#[inline]
pub fn binary_byte_string(value: u64) -> String {
    let converted_values = get_binary_bytes(value);
    if value >= GIBI_LIMIT {
        format!("{:.1}{}", converted_values.0, converted_values.1)
    } else {
        format!("{:.0}{}", converted_values.0, converted_values.1)
    }
}

/// Returns a string given a value that is converted to the closest SI-variant,
/// per second. If the value is greater than a giga-X, then it will return a
/// decimal place.
#[inline]
pub fn dec_bytes_per_second_string(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.1}{}/s", converted_values.0, converted_values.1)
    } else {
        format!("{:.0}{}/s", converted_values.0, converted_values.1)
    }
}

/// Returns a string given a value that is converted to the closest SI-variant.
/// If the value is greater than a giga-X, then it will return a decimal place.
pub fn dec_bytes_string(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.1}{}", converted_values.0, converted_values.1)
    } else {
        format!("{:.0}{}", converted_values.0, converted_values.1)
    }
}

#[cfg(feature = "zfs")]
pub fn convert_arc_data_points(current_data: &CollectedData) -> Vec<Point> {
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
pub fn convert_gpu_data(current_data: &CollectedData) -> Option<Vec<ConvertedGpuData>> {
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
        .filter_map(|(gpu, points)| {
            (gpu.1.total_bytes > 0).then(|| {
                let short_name = {
                    let last_words = gpu.0.split_whitespace().rev().take(2).collect::<Vec<_>>();
                    let short_name = format!("{} {}", last_words[1], last_words[0]);
                    short_name
                };

                let percent = gpu.1.used_bytes as f64 / gpu.1.total_bytes as f64 * 100.0;

                ConvertedGpuData {
                    name: short_name,
                    points,
                    mem_percent: format!("{percent:3.0}%"),
                    mem_total: {
                        let (unit, denominator) =
                            get_binary_unit_and_denominator(gpu.1.total_bytes);

                        format!(
                            "   {:.1}{unit}/{:.1}{unit}",
                            gpu.1.used_bytes as f64 / denominator,
                            (gpu.1.total_bytes as f64 / denominator),
                        )
                    },
                }
            })
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
