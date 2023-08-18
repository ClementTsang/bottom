use std::{
    cmp::{max, Ordering},
    fmt::Display,
    time::Duration,
};

use concat_string::concat_string;
use tui::{text::Text, widgets::Row};

use super::proc_widget_column::ProcColumn;
use crate::{
    app::data_harvester::processes::ProcessHarvest,
    canvas::Painter,
    components::data_table::{DataTableColumn, DataToCell},
    data_conversion::{binary_byte_string, dec_bytes_per_second_string, dec_bytes_string},
    utils::gen_util::truncate_to_text,
    Pid,
};

#[derive(Clone, Debug)]
enum IdType {
    Name(String),
    Command(String),
}

#[derive(Clone, Debug)]
pub struct Id {
    id_type: IdType,
    prefix: Option<String>,
}

impl From<&'static str> for Id {
    fn from(s: &'static str) -> Self {
        Id {
            id_type: IdType::Name(s.to_string()),
            prefix: None,
        }
    }
}

impl Id {
    /// Returns the ID as a lowercase [`String`], with no prefix. This is primarily useful for
    /// cases like sorting where we treat everything as the same case (e.g. `Discord` comes before
    /// `dkms`).
    pub fn to_lowercase(&self) -> String {
        match &self.id_type {
            IdType::Name(name) => name.to_lowercase(),
            IdType::Command(cmd) => cmd.to_lowercase(),
        }
    }

    /// Return the ID as a borrowed [`str`] with no prefix.
    pub fn as_str(&self) -> &str {
        match &self.id_type {
            IdType::Name(name) => name.as_str(),
            IdType::Command(cmd) => cmd.as_str(),
        }
    }

    /// Returns the ID as a [`String`] with prefix.
    pub fn to_prefixed_string(&self) -> String {
        if let Some(prefix) = &self.prefix {
            concat_string!(
                prefix,
                match &self.id_type {
                    IdType::Name(name) => name,
                    IdType::Command(cmd) => cmd,
                }
            )
        } else {
            match &self.id_type {
                IdType::Name(name) => name.to_string(),
                IdType::Command(cmd) => cmd.to_string(),
            }
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MemUsage {
    Percent(f32),
    Bytes(u64),
}

impl PartialOrd for MemUsage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (MemUsage::Percent(a), MemUsage::Percent(b)) => a.partial_cmp(b),
            (MemUsage::Bytes(a), MemUsage::Bytes(b)) => a.partial_cmp(b),
            _ => unreachable!(),
        }
    }
}

impl Display for MemUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemUsage::Percent(percent) => f.write_fmt(format_args!("{:.1}%", percent)),
            MemUsage::Bytes(bytes) => f.write_str(&binary_byte_string(*bytes)),
        }
    }
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

#[derive(Clone, Debug)]
pub struct ProcWidgetData {
    pub pid: Pid,
    pub ppid: Option<Pid>,
    pub id: Id,
    pub cpu_usage_percent: f32,
    pub mem_usage: MemUsage,
    pub rps: u64,
    pub wps: u64,
    pub total_read: u64,
    pub total_write: u64,
    pub process_state: String,
    pub process_char: char,
    pub user: String,
    pub num_similar: u64,
    pub disabled: bool,
    pub time: Duration,
    #[cfg(feature = "gpu")]
    pub gpu_mem_usage: u64,
    #[cfg(feature = "gpu")]
    pub gpu_usage: u32,
}

impl ProcWidgetData {
    pub fn from_data(process: &ProcessHarvest, is_command: bool, is_mem_percent: bool) -> Self {
        let id = Id {
            id_type: if is_command {
                IdType::Command(process.command.clone())
            } else {
                IdType::Name(process.name.clone())
            },
            prefix: None,
        };

        let mem_usage = if is_mem_percent {
            MemUsage::Percent(process.mem_usage_percent)
        } else {
            MemUsage::Bytes(process.mem_usage_bytes)
        };

        Self {
            pid: process.pid,
            ppid: process.parent_pid,
            id,
            cpu_usage_percent: process.cpu_usage_percent,
            mem_usage,
            rps: process.read_bytes_per_sec,
            wps: process.write_bytes_per_sec,
            total_read: process.total_read_bytes,
            total_write: process.total_write_bytes,
            process_state: process.process_state.0.clone(),
            process_char: process.process_state.1,
            user: process.user.to_string(),
            num_similar: 1,
            disabled: false,
            time: process.time,
            #[cfg(feature = "gpu")]
            gpu_mem_usage: process.gpu_mem,
            #[cfg(feature = "gpu")]
            gpu_usage: process.gpu_util,
        }
    }

    pub fn num_similar(mut self, num_similar: u64) -> Self {
        self.num_similar = num_similar;
        self
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
        self.cpu_usage_percent += other.cpu_usage_percent;
        self.mem_usage = match (&self.mem_usage, &other.mem_usage) {
            (MemUsage::Percent(a), MemUsage::Percent(b)) => MemUsage::Percent(a + b),
            (MemUsage::Bytes(a), MemUsage::Bytes(b)) => MemUsage::Bytes(a + b),
            (MemUsage::Percent(_), MemUsage::Bytes(_))
            | (MemUsage::Bytes(_), MemUsage::Percent(_)) => {
                unreachable!("trying to add together two different memory usage types!")
            }
        };
        self.rps += other.rps;
        self.wps += other.wps;
        self.total_read += other.total_read;
        self.total_write += other.total_write;
        #[cfg(feature = "gpu")]
        {
            self.gpu_mem_usage += other.gpu_mem_usage;
            self.gpu_usage += other.gpu_usage;
        }
    }

    fn to_string(&self, column: &ProcColumn) -> String {
        match column {
            ProcColumn::CpuPercent => format!("{:.1}%", self.cpu_usage_percent),
            ProcColumn::MemoryVal | ProcColumn::MemoryPercent => self.mem_usage.to_string(),
            ProcColumn::Pid => self.pid.to_string(),
            ProcColumn::Count => self.num_similar.to_string(),
            ProcColumn::Name | ProcColumn::Command => self.id.to_prefixed_string(),
            ProcColumn::ReadPerSecond => dec_bytes_per_second_string(self.rps),
            ProcColumn::WritePerSecond => dec_bytes_per_second_string(self.wps),
            ProcColumn::TotalRead => dec_bytes_string(self.total_read),
            ProcColumn::TotalWrite => dec_bytes_string(self.total_write),
            ProcColumn::State => self.process_char.to_string(),
            ProcColumn::User => self.user.clone(),
            ProcColumn::Time => format_time(self.time),
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMem => self.gpu_mem_usage.to_string(),
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => format!("{:.1}%", self.gpu_usage),
        }
    }
}

impl DataToCell<ProcColumn> for ProcWidgetData {
    fn to_cell<'a>(&'a self, column: &ProcColumn, calculated_width: u16) -> Option<Text<'a>> {
        if calculated_width == 0 {
            return None;
        }

        // TODO: Optimize the string allocations here...
        // TODO: Also maybe just pull in the to_string call but add a variable for the differences.
        Some(truncate_to_text(
            &match column {
                ProcColumn::CpuPercent => {
                    format!("{:.1}%", self.cpu_usage_percent)
                }
                ProcColumn::MemoryVal | ProcColumn::MemoryPercent => self.mem_usage.to_string(),
                ProcColumn::Pid => self.pid.to_string(),
                ProcColumn::Count => self.num_similar.to_string(),
                ProcColumn::Name | ProcColumn::Command => self.id.to_prefixed_string(),
                ProcColumn::ReadPerSecond => dec_bytes_per_second_string(self.rps),
                ProcColumn::WritePerSecond => dec_bytes_per_second_string(self.wps),
                ProcColumn::TotalRead => dec_bytes_string(self.total_read),
                ProcColumn::TotalWrite => dec_bytes_string(self.total_write),
                ProcColumn::State => {
                    if calculated_width < 8 {
                        self.process_char.to_string()
                    } else {
                        self.process_state.clone()
                    }
                }
                ProcColumn::User => self.user.clone(),
                ProcColumn::Time => format_time(self.time),
                #[cfg(feature = "gpu")]
                ProcColumn::GpuMem => binary_byte_string(self.gpu_mem_usage),
                #[cfg(feature = "gpu")]
                ProcColumn::GpuUtilPercent => format!("{:.1}%", self.gpu_usage),
            },
            calculated_width,
        ))
    }

    #[inline(always)]
    fn style_row<'a>(&self, row: Row<'a>, painter: &Painter) -> Row<'a> {
        if self.disabled {
            row.style(painter.colours.disabled_text_style)
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

    use crate::widgets::proc_widget_data::format_time;

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
}
