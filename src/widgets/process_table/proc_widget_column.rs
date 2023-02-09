use std::{borrow::Cow, cmp::Reverse};

use super::ProcWidgetData;
use crate::{
    components::data_table::{ColumnHeader, SortsRow},
    utils::gen_util::sort_partial_fn,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ProcColumn {
    CpuPercent,
    MemoryVal,
    MemoryPercent,
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
}

impl ColumnHeader for ProcColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            ProcColumn::CpuPercent => "CPU%",
            ProcColumn::MemoryVal => "Mem",
            ProcColumn::MemoryPercent => "Mem%",
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
        }
        .into()
    }

    fn header(&self) -> Cow<'static, str> {
        match self {
            ProcColumn::CpuPercent => "CPU%(c)",
            ProcColumn::MemoryVal => "Mem(m)",
            ProcColumn::MemoryPercent => "Mem%(m)",
            ProcColumn::Pid => "PID(p)",
            ProcColumn::Count => "Count",
            ProcColumn::Name => "Name(n)",
            ProcColumn::Command => "Command(n)",
            ProcColumn::ReadPerSecond => "R/s",
            ProcColumn::WritePerSecond => "W/s",
            ProcColumn::TotalRead => "T.Read",
            ProcColumn::TotalWrite => "T.Write",
            ProcColumn::State => "State",
            ProcColumn::User => "User",
        }
        .into()
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
            ProcColumn::MemoryVal | ProcColumn::MemoryPercent => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.mem_usage, &b.mem_usage));
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
                    data.sort_by_cached_key(|pd| Reverse(pd.process_state.to_lowercase()));
                } else {
                    data.sort_by_cached_key(|pd| pd.process_state.to_lowercase());
                }
            }
            ProcColumn::User => {
                if descending {
                    data.sort_by_cached_key(|pd| Reverse(pd.user.to_lowercase()));
                } else {
                    data.sort_by_cached_key(|pd| pd.user.to_lowercase());
                }
            }
        }
    }
}
