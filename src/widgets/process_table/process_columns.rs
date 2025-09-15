use std::{borrow::Cow, cmp::Reverse};

use serde::Deserialize;

use super::{ProcWidgetColumn, ProcWidgetData};
use crate::{
    canvas::components::data_table::{ColumnHeader, SortsRow},
    utils::general::sort_partial_fn,
};

/// A column in the process widget.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
#[cfg_attr(
    feature = "generate_schema",
    derive(schemars::JsonSchema, strum::VariantArray)
)]
pub enum ProcColumn {
    CpuPercent,
    MemValue,
    MemPercent,
    VirtualMem,
    Pid,
    Count,
    Name,
    Command,
    ReadPerSecond,
    WritePerSecond,
    TotalRead,
    TotalWrite,
    State,
    User,
    Time,
    #[cfg(feature = "gpu")]
    GpuMemValue,
    #[cfg(feature = "gpu")]
    GpuMemPercent,
    #[cfg(feature = "gpu")]
    GpuUtilPercent,
}

impl ProcColumn {
    /// An ugly hack to generate the JSON schema.
    #[cfg(feature = "generate_schema")]
    pub fn get_schema_names(&self) -> &[&'static str] {
        match self {
            ProcColumn::Pid => &["PID"],
            ProcColumn::Count => &["Count"],
            ProcColumn::Name => &["Name"],
            ProcColumn::Command => &["Command"],
            ProcColumn::CpuPercent => &["CPU%"],
            // TODO: Change this
            ProcColumn::MemValue | ProcColumn::MemPercent => &["Mem", "Mem%", "Memory", "Memory%"],
            ProcColumn::VirtualMem => &["Virt", "Virtual", "VirtMem", "Virtual Memory"],
            ProcColumn::ReadPerSecond => &["R/s", "Read", "Rps"],
            ProcColumn::WritePerSecond => &["W/s", "Write", "Wps"],
            ProcColumn::TotalRead => &["T.Read", "TRead", "Total Read"],
            ProcColumn::TotalWrite => &["T.Write", "TWrite", "Total Write"],
            ProcColumn::State => &["State"],
            ProcColumn::User => &["User"],
            ProcColumn::Time => &["Time"],
            #[cfg(feature = "gpu")]
            // TODO: Change this
            ProcColumn::GpuMemValue | ProcColumn::GpuMemPercent => &["GMem", "GMem%"],
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => &["GPU%"],
        }
    }
}

impl ColumnHeader for ProcColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            ProcColumn::CpuPercent => "CPU%",
            ProcColumn::MemValue => "Mem",
            ProcColumn::MemPercent => "Mem%",
            ProcColumn::VirtualMem => "Virt",
            ProcColumn::Pid => "PID",
            ProcColumn::Count => "Count",
            ProcColumn::Name => "Name",
            ProcColumn::Command => "Command",
            ProcColumn::ReadPerSecond => "R/s",
            ProcColumn::WritePerSecond => "W/s",
            ProcColumn::TotalRead => "T.Read",
            ProcColumn::TotalWrite => "T.Write",
            ProcColumn::State => "State",
            ProcColumn::User => "User",
            ProcColumn::Time => "Time",
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMemValue => "GMem",
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMemPercent => "GMem%",
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => "GPU%",
        }
        .into()
    }

    fn header(&self) -> Cow<'static, str> {
        match self {
            ProcColumn::CpuPercent => "CPU%(c)".into(),
            ProcColumn::MemValue => "Mem(m)".into(),
            ProcColumn::MemPercent => "Mem%(m)".into(),
            ProcColumn::Pid => "PID(p)".into(),
            ProcColumn::Name => "Name(n)".into(),
            ProcColumn::Command => "Command(n)".into(),
            _ => self.text(),
        }
    }
}

impl SortsRow for ProcColumn {
    type DataType = ProcWidgetData;

    fn sort_data(&self, data: &mut [ProcWidgetData], descending: bool) {
        match self {
            ProcColumn::CpuPercent => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(a.cpu_usage_percent, b.cpu_usage_percent)
                });
            }
            ProcColumn::MemValue | ProcColumn::MemPercent => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.mem_usage, &b.mem_usage));
            }
            ProcColumn::VirtualMem => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.virtual_mem, &b.virtual_mem));
            }
            ProcColumn::Pid => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.pid, b.pid));
            }
            ProcColumn::Count => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.num_similar, b.num_similar));
            }
            ProcColumn::Name | ProcColumn::Command => {
                if descending {
                    data.sort_by_cached_key(|pd| Reverse(pd.id.to_lowercase()));
                } else {
                    data.sort_by_cached_key(|pd| pd.id.to_lowercase());
                }
            }
            ProcColumn::ReadPerSecond => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.rps, b.rps));
            }
            ProcColumn::WritePerSecond => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.wps, b.wps));
            }
            ProcColumn::TotalRead => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.total_read, b.total_read));
            }
            ProcColumn::TotalWrite => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.total_write, b.total_write));
            }
            ProcColumn::State => {
                if descending {
                    data.sort_by_cached_key(|pd| Reverse(pd.process_state));
                } else {
                    data.sort_by_cached_key(|pd| pd.process_state);
                }
            }
            ProcColumn::User => {
                // FIXME: Is there a better way here to keep the to_lowercase? Usually it shouldn't matter but...
                if descending {
                    data.sort_by_cached_key(|pd| Reverse(pd.user.clone()));
                } else {
                    data.sort_by_cached_key(|pd| pd.user.clone());
                }
            }
            ProcColumn::Time => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.time, b.time));
            }
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMemValue | ProcColumn::GpuMemPercent => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.gpu_mem_usage, &b.gpu_mem_usage)
                });
            }
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => {
                data.sort_by(|a, b| sort_partial_fn(descending)(a.gpu_usage, b.gpu_usage));
            }
        }
    }
}

impl<'de> Deserialize<'de> for ProcColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        match value.as_str() {
            "cpu%" => Ok(ProcColumn::CpuPercent),
            // TODO: Maybe change this in the future.
            "mem" | "mem%" => Ok(ProcColumn::MemPercent),
            "virt" | "virtual" | "virtmem" | "virtual memory" => Ok(ProcColumn::VirtualMem),
            "pid" => Ok(ProcColumn::Pid),
            "count" => Ok(ProcColumn::Count),
            "name" => Ok(ProcColumn::Name),
            "command" => Ok(ProcColumn::Command),
            "read" | "r/s" | "rps" => Ok(ProcColumn::ReadPerSecond),
            "write" | "w/s" | "wps" => Ok(ProcColumn::WritePerSecond),
            "tread" | "t.read" => Ok(ProcColumn::TotalRead),
            "twrite" | "t.write" => Ok(ProcColumn::TotalWrite),
            "state" => Ok(ProcColumn::State),
            "user" => Ok(ProcColumn::User),
            "time" => Ok(ProcColumn::Time),
            #[cfg(feature = "gpu")]
            // TODO: Maybe change this in the future.
            "gmem" | "gmem%" => Ok(ProcColumn::GpuMemPercent),
            #[cfg(feature = "gpu")]
            "gpu%" => Ok(ProcColumn::GpuUtilPercent),
            _ => Err(serde::de::Error::custom(
                "doesn't match any process column name",
            )),
        }
    }
}

impl From<&ProcColumn> for ProcWidgetColumn {
    fn from(value: &ProcColumn) -> Self {
        match value {
            ProcColumn::Pid | ProcColumn::Count => ProcWidgetColumn::PidOrCount,
            ProcColumn::Name | ProcColumn::Command => ProcWidgetColumn::ProcNameOrCommand,
            ProcColumn::CpuPercent => ProcWidgetColumn::Cpu,
            ProcColumn::MemPercent | ProcColumn::MemValue => ProcWidgetColumn::Mem,
            ProcColumn::VirtualMem => ProcWidgetColumn::VirtualMem,
            ProcColumn::ReadPerSecond => ProcWidgetColumn::ReadPerSecond,
            ProcColumn::WritePerSecond => ProcWidgetColumn::WritePerSecond,
            ProcColumn::TotalRead => ProcWidgetColumn::TotalRead,
            ProcColumn::TotalWrite => ProcWidgetColumn::TotalWrite,
            ProcColumn::State => ProcWidgetColumn::State,
            ProcColumn::User => ProcWidgetColumn::User,
            ProcColumn::Time => ProcWidgetColumn::Time,
            #[cfg(feature = "gpu")]
            ProcColumn::GpuMemPercent | ProcColumn::GpuMemValue => ProcWidgetColumn::GpuMem,
            #[cfg(feature = "gpu")]
            ProcColumn::GpuUtilPercent => ProcWidgetColumn::GpuUtil,
        }
    }
}
