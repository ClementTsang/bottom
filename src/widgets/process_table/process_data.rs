use std::{borrow::Cow, cmp::max, num::NonZeroU16, time::Duration};

use concat_string::concat_string;
use ratatui::widgets::Row;

use super::process_columns::ProcColumn;
use crate::{
    canvas::{
        Painter,
        components::data_table::{DataTableColumn, DataToCell},
    },
    collection::processes::{Pid, ProcessHarvest},
    dec_bytes_per_second_string,
    utils::data_units::{GIBI_LIMIT, GIGA_LIMIT, get_binary_bytes, get_decimal_bytes},
};

#[derive(Clone, Debug)]
enum IdType {
    Name,
    Command,
}

#[derive(Clone, Debug)]
pub struct Id {
    id_type: IdType,
    prefix: Option<String>,
}

trait DurationExt {
    fn num_days(&self) -> u64;
    fn num_hours(&self) -> u64;
    fn num_minutes(&self) -> u64;
}

const SECS_PER_DAY: u64 = SECS_PER_HOUR * 24;
const SECS_PER_HOUR: u64 = SECS_PER_MINUTE * 60;
const SECS_PER_MINUTE: u64 = 60;

impl DurationExt for Duration {
    /// Number of full days in this duration.
    #[inline]
    fn num_days(&self) -> u64 {
        self.as_secs() / SECS_PER_DAY
    }

    /// Number of full hours in this duration.
    #[inline]
    fn num_hours(&self) -> u64 {
        self.as_secs() / SECS_PER_HOUR
    }

    /// Number of full minutes in this duration.
    #[inline]
    fn num_minutes(&self) -> u64 {
        self.as_secs() / SECS_PER_MINUTE
    }
}

fn format_time(dur: Duration) -> String {
    if dur.num_days() > 0 {
        format!(
            "{}d {}h {}m",
            dur.num_days(),
            dur.num_hours() % 24,
            dur.num_minutes() % 60
        )
    } else if dur.num_hours() > 0 {
        format!(
            "{}h {}m {}s",
            dur.num_hours(),
            dur.num_minutes() % 60,
            dur.as_secs() % 60
        )
    } else if dur.num_minutes() > 0 {
        format!(
            "{}m {}.{:02}s",
            dur.num_minutes(),
            dur.as_secs() % 60,
            dur.as_millis() % 1000 / 10
        )
    } else {
        format!("{}.{:03}s", dur.as_secs(), dur.as_millis() % 1000)
    }
}

/// Returns a string given a value that is converted to the closest binary
/// variant. If the value is greater than a gibibyte, then it will return a
/// decimal place.
#[inline]
fn binary_byte_string(value: u64) -> String {
    let converted_values = get_binary_bytes(value);
    if value >= GIBI_LIMIT {
        format!("{:.1}{}", converted_values.0, converted_values.1)
    } else {
        format!("{:.0}{}", converted_values.0, converted_values.1)
    }
}

/// Returns a string given a value that is converted to the closest SI-variant.
/// If the value is greater than a giga-X, then it will return a decimal place.
fn dec_bytes_string(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.1}{}", converted_values.0, converted_values.1)
    } else {
        format!("{:.0}{}", converted_values.0, converted_values.1)
    }
}

/// Process widget data.
///
/// TODO: Maybe we should filter out data that isn't set?
#[derive(Clone)]
pub struct ProcWidgetData {
    pub inner: ProcessHarvest,
    pub id: Id,
    pub count: u64,
    pub disabled: bool,
}

impl ProcWidgetData {
    /// Returns the PID.
    #[inline]
    pub fn pid(&self) -> Pid {
        self.inner.pid
    }

    /// Return the current string to be used as an identifier for the process.
    #[inline]
    pub fn current_identifier(&self) -> &str {
        match &self.id.id_type {
            IdType::Name => &self.inner.name,
            IdType::Command => &self.inner.command,
        }
    }

    /// Returns the ID as a [`String`] with prefix.
    fn to_prefixed_string(&self) -> String {
        if let Some(prefix) = &self.id.prefix {
            concat_string!(
                prefix,
                match &self.id.id_type {
                    IdType::Name => &self.inner.name,
                    IdType::Command => &self.inner.command,
                }
            )
        } else {
            self.current_identifier().to_string()
        }
    }

    pub fn from_data(process: &ProcessHarvest, is_command: bool) -> Self {
        let id = Id {
            id_type: if is_command {
                IdType::Command
            } else {
                IdType::Name
            },
            prefix: None,
        };

        Self {
            inner: process.clone(),
            id,
            count: 1,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn prefix(mut self, prefix: Option<String>) -> Self {
        self.id.prefix = prefix;
        self
    }

    pub fn add(&mut self, other: &Self) {
        self.inner.cpu_usage_percent += other.inner.cpu_usage_percent;
        self.inner.mem_usage = self.inner.mem_usage + other.inner.mem_usage;
        self.inner.read_per_sec += other.inner.read_per_sec;
        self.inner.write_per_sec += other.inner.write_per_sec;
        self.inner.total_read += other.inner.total_read;
        self.inner.total_write += other.inner.total_write;
        self.inner.time = self.inner.time.max(other.inner.time);
        #[cfg(feature = "gpu")]
        {
            self.inner.gpu_mem += other.inner.gpu_mem;
            self.inner.gpu_util += other.inner.gpu_util;
        }
    }

    fn to_string(&self, column: &ProcColumn) -> String {
        match column {
            &ProcColumn::Priority => self.inner.priority.to_string(),
            #[cfg(unix)]
            ProcColumn::Nice => self.inner.nice.to_string(),
            ProcColumn::CpuPercent => format!("{:.1}%", self.inner.cpu_usage_percent),
            ProcColumn::MemValue | ProcColumn::MemPercent => self.inner.mem_usage.to_string(),
            ProcColumn::VirtualMem => binary_byte_string(self.inner.virtual_mem),
            ProcColumn::Pid => self.pid().to_string(),
            ProcColumn::Count => self.count.to_string(),
            ProcColumn::Name | ProcColumn::Command => self.to_prefixed_string(),
            ProcColumn::ReadPerSecond => dec_bytes_per_second_string(self.inner.read_per_sec),
            ProcColumn::WritePerSecond => dec_bytes_per_second_string(self.inner.write_per_sec),
            ProcColumn::TotalRead => dec_bytes_string(self.inner.total_read),
            ProcColumn::TotalWrite => dec_bytes_string(self.inner.total_write),
            ProcColumn::State => self.inner.process_state.1.to_string(),
            ProcColumn::User => self
                .inner
                .user
                .as_ref()
                .map(|user| user.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            ProcColumn::Time => format_time(self.inner.time),
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMemValue | ProcColumn::GpuMemPercent => self.inner.gpu_mem.to_string(),
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => format!("{:.1}%", self.inner.gpu_util),
        }
    }
}

impl DataToCell<ProcColumn> for ProcWidgetData {
    fn to_cell_text(
        &self, column: &ProcColumn, calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        // TODO: Optimize the string allocations here...
        // TODO: Also maybe just pull in the to_string call but add a variable for the
        // differences.
        Some(match column {
            #[cfg(unix)]
            ProcColumn::Nice => self.inner.nice.to_string().into(),
            &ProcColumn::Priority => self.inner.priority.to_string().into(),
            ProcColumn::CpuPercent => format!("{:.1}%", self.inner.cpu_usage_percent).into(),
            ProcColumn::MemValue | ProcColumn::MemPercent => {
                self.inner.mem_usage.to_string().into()
            }
            ProcColumn::VirtualMem => binary_byte_string(self.inner.virtual_mem).into(),
            ProcColumn::Pid => self.pid().to_string().into(),
            ProcColumn::Count => self.count.to_string().into(),
            ProcColumn::Name | ProcColumn::Command => self.to_prefixed_string().into(),
            ProcColumn::ReadPerSecond => {
                dec_bytes_per_second_string(self.inner.read_per_sec).into()
            }
            ProcColumn::WritePerSecond => {
                dec_bytes_per_second_string(self.inner.write_per_sec).into()
            }
            ProcColumn::TotalRead => dec_bytes_string(self.inner.total_read).into(),
            ProcColumn::TotalWrite => dec_bytes_string(self.inner.total_write).into(),
            ProcColumn::State => {
                if calculated_width.get() < 8 {
                    self.inner.process_state.1.to_string().into()
                } else {
                    self.inner.process_state.0.into()
                }
            }
            ProcColumn::User => self
                .inner
                .user
                .as_ref()
                .map(|user| user.to_string().into())
                .unwrap_or_else(|| "N/A".into()),
            ProcColumn::Time => format_time(self.inner.time).into(),
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMemValue | ProcColumn::GpuMemPercent => {
                self.inner.gpu_mem.to_string().into()
            }
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => format!("{:.1}%", self.inner.gpu_util).into(),
        })
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]
    fn style_cell(&self, column: &ProcColumn, painter: &Painter) -> Option<ratatui::style::Style> {
        match column {
            ProcColumn::Name | ProcColumn::Command if self.inner.process_type.is_thread() => {
                Some(painter.styles.thread_text_style)
            }
            _ => None,
        }
    }

    #[inline(always)]
    fn style_row<'a>(&self, row: Row<'a>, painter: &Painter) -> Row<'a> {
        if self.disabled {
            row.style(painter.styles.disabled_text_style)
        } else {
            row
        }
    }

    fn column_widths<C: DataTableColumn<ProcColumn>>(data: &[Self], columns: &[C]) -> Vec<u16>
    where
        Self: Sized,
    {
        let mut widths = vec![0; columns.len()];

        for d in data {
            for (w, c) in widths.iter_mut().zip(columns) {
                *w = max(*w, d.to_string(c.inner()).len() as u16);
            }
        }

        widths
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;
    use crate::utils::data_units::*;

    #[test]
    fn test_format_time() {
        const ONE_DAY: u64 = 24 * 60 * 60;

        assert_eq!(format_time(Duration::from_millis(500)), "0.500s");
        assert_eq!(format_time(Duration::from_millis(900)), "0.900s");
        assert_eq!(format_time(Duration::from_secs(1)), "1.000s");
        assert_eq!(format_time(Duration::from_secs(10)), "10.000s");
        assert_eq!(format_time(Duration::from_secs(60)), "1m 0.00s");
        assert_eq!(format_time(Duration::from_secs(61)), "1m 1.00s");
        assert_eq!(format_time(Duration::from_secs(600)), "10m 0.00s");
        assert_eq!(format_time(Duration::from_secs(601)), "10m 1.00s");
        assert_eq!(format_time(Duration::from_secs(3600)), "1h 0m 0s");
        assert_eq!(format_time(Duration::from_secs(3601)), "1h 0m 1s");
        assert_eq!(format_time(Duration::from_secs(3660)), "1h 1m 0s");
        assert_eq!(format_time(Duration::from_secs(3661)), "1h 1m 1s");
        assert_eq!(format_time(Duration::from_secs(ONE_DAY - 1)), "23h 59m 59s");
        assert_eq!(format_time(Duration::from_secs(ONE_DAY)), "1d 0h 0m");
        assert_eq!(format_time(Duration::from_secs(ONE_DAY + 1)), "1d 0h 0m");
        assert_eq!(format_time(Duration::from_secs(ONE_DAY + 60)), "1d 0h 1m");
        assert_eq!(
            format_time(Duration::from_secs(ONE_DAY + 3600 - 1)),
            "1d 0h 59m"
        );
        assert_eq!(format_time(Duration::from_secs(ONE_DAY + 3600)), "1d 1h 0m");
        assert_eq!(
            format_time(Duration::from_secs(ONE_DAY * 365 - 1)),
            "364d 23h 59m"
        );
    }

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
}
