use std::{
    cmp::{max, Ordering},
    fmt::Display,
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

// TODO: Can reduce this to 32 bytes.
#[derive(PartialEq, Clone, Debug)]
pub enum MemUsage {
    Percent(f64),
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

#[derive(Clone, Debug)]
pub struct ProcWidgetData {
    pub pid: Pid,
    pub ppid: Option<Pid>,
    pub id: Id,
    pub cpu_usage_percent: f64,
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
