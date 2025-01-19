//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

// TODO: Split this up!

use std::borrow::Cow;

use crate::{
    app::data::CollectedData,
    canvas::components::time_chart::Point,
    data_collection::{cpu::CpuDataType, temperature::TemperatureType},
    utils::data_units::*,
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

/// Returns the most appropriate binary prefix unit type (e.g. kibibyte) and
/// denominator for the given amount of bytes.
///
/// The expected usage is to divide out the given value with the returned
/// denominator in order to be able to use it with the returned binary unit
/// (e.g. divide 3000 bytes by 1024 to have a value in KiB).
#[inline]
pub fn get_binary_unit_and_denominator(bytes: u64) -> (&'static str, f64) {
    match bytes {
        b if b < KIBI_LIMIT => ("B", 1.0),
        b if b < MEBI_LIMIT => ("KiB", KIBI_LIMIT_F64),
        b if b < GIBI_LIMIT => ("MiB", MEBI_LIMIT_F64),
        b if b < TEBI_LIMIT => ("GiB", GIBI_LIMIT_F64),
        _ => ("TiB", TEBI_LIMIT_F64),
    }
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
