use crate::{
    app::{data_farmer::StringPidMap, data_harvester::processes::ProcessHarvest},
    components::old_text_table::{CellContent, SortOrder, TableComponentHeader},
    utils::gen_util::sort_partial_fn,
};

use std::{borrow::Cow, cmp::Reverse};

#[derive(Debug, PartialEq, Eq)]
pub enum ProcWidgetColumn {
    CpuPercent,
    Memory { show_percentage: bool },
    PidOrCount { is_count: bool },
    ProcNameOrCommand { is_command: bool },
    ReadPerSecond,
    WritePerSecond,
    TotalRead,
    TotalWrite,
    State,
    User,
}

impl ProcWidgetColumn {
    const CPU_PERCENT: CellContent = CellContent::Simple(Cow::Borrowed("CPU%"));
    const MEM_PERCENT: CellContent = CellContent::Simple(Cow::Borrowed("Mem%"));
    const MEM: CellContent = CellContent::Simple(Cow::Borrowed("Mem"));
    const READS_PER_SECOND: CellContent = CellContent::Simple(Cow::Borrowed("R/s"));
    const WRITES_PER_SECOND: CellContent = CellContent::Simple(Cow::Borrowed("W/s"));
    const TOTAL_READ: CellContent = CellContent::Simple(Cow::Borrowed("T.Read"));
    const TOTAL_WRITE: CellContent = CellContent::Simple(Cow::Borrowed("T.Write"));
    const STATE: CellContent = CellContent::Simple(Cow::Borrowed("State"));
    const PROCESS_NAME: CellContent = CellContent::Simple(Cow::Borrowed("Name"));
    const COMMAND: CellContent = CellContent::Simple(Cow::Borrowed("Command"));
    const PID: CellContent = CellContent::Simple(Cow::Borrowed("PID"));
    const COUNT: CellContent = CellContent::Simple(Cow::Borrowed("Count"));
    const USER: CellContent = CellContent::Simple(Cow::Borrowed("User"));

    const SHORTCUT_CPU_PERCENT: CellContent = CellContent::Simple(Cow::Borrowed("CPU%(c)"));
    const SHORTCUT_MEM_PERCENT: CellContent = CellContent::Simple(Cow::Borrowed("Mem%(m)"));
    const SHORTCUT_MEM: CellContent = CellContent::Simple(Cow::Borrowed("Mem(m)"));
    const SHORTCUT_PROCESS_NAME: CellContent = CellContent::Simple(Cow::Borrowed("Name(n)"));
    const SHORTCUT_COMMAND: CellContent = CellContent::Simple(Cow::Borrowed("Command(n)"));
    const SHORTCUT_PID: CellContent = CellContent::Simple(Cow::Borrowed("PID(p)"));

    pub fn text(&self) -> &CellContent {
        match self {
            ProcWidgetColumn::CpuPercent => &Self::CPU_PERCENT,
            ProcWidgetColumn::Memory { show_percentage } => {
                if *show_percentage {
                    &Self::MEM_PERCENT
                } else {
                    &Self::MEM
                }
            }
            ProcWidgetColumn::PidOrCount { is_count } => {
                if *is_count {
                    &Self::COUNT
                } else {
                    &Self::PID
                }
            }
            ProcWidgetColumn::ProcNameOrCommand { is_command } => {
                if *is_command {
                    &Self::COMMAND
                } else {
                    &Self::PROCESS_NAME
                }
            }
            ProcWidgetColumn::ReadPerSecond => &Self::READS_PER_SECOND,
            ProcWidgetColumn::WritePerSecond => &Self::WRITES_PER_SECOND,
            ProcWidgetColumn::TotalRead => &Self::TOTAL_READ,
            ProcWidgetColumn::TotalWrite => &Self::TOTAL_WRITE,
            ProcWidgetColumn::State => &Self::STATE,
            ProcWidgetColumn::User => &Self::USER,
        }
    }

    /// Sorts the given data in-place.
    pub fn sort(
        &self, sort_descending: bool, data: &mut [&ProcessHarvest], is_using_command: bool,
        cmd_pid_map: &StringPidMap, name_pid_map: &StringPidMap,
    ) {
        match self {
            ProcWidgetColumn::CpuPercent => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                data.sort_by(|a, b| {
                    sort_partial_fn(sort_descending)(a.cpu_usage_percent, b.cpu_usage_percent)
                });
            }
            ProcWidgetColumn::Memory { show_percentage } => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                if *show_percentage {
                    data.sort_by(|a, b| {
                        sort_partial_fn(sort_descending)(a.mem_usage_percent, b.mem_usage_percent)
                    });
                } else {
                    data.sort_by(|a, b| {
                        sort_partial_fn(sort_descending)(a.mem_usage_bytes, b.mem_usage_bytes)
                    });
                }
            }
            ProcWidgetColumn::PidOrCount { is_count } => {
                data.sort_by_cached_key(|c| c.name.to_lowercase());
                if *is_count {
                    if is_using_command {
                        if sort_descending {
                            data.sort_by_cached_key(|p| {
                                Reverse(cmd_pid_map.get(&p.command).map(|v| v.len()).unwrap_or(0))
                            })
                        } else {
                            data.sort_by_cached_key(|p| {
                                cmd_pid_map.get(&p.command).map(|v| v.len()).unwrap_or(0)
                            })
                        }
                    } else {
                        #[allow(clippy::collapsible-else-if)]
                        if sort_descending {
                            data.sort_by_cached_key(|p| {
                                Reverse(name_pid_map.get(&p.name).map(|v| v.len()).unwrap_or(0))
                            })
                        } else {
                            data.sort_by_cached_key(|p| {
                                name_pid_map.get(&p.name).map(|v| v.len()).unwrap_or(0)
                            })
                        }
                    }
                } else {
                    data.sort_by(|a, b| sort_partial_fn(sort_descending)(a.pid, b.pid));
                }
            }
            ProcWidgetColumn::ProcNameOrCommand { is_command } => {
                if *is_command {
                    if sort_descending {
                        data.sort_by_cached_key(|p| Reverse(p.command.to_lowercase()));
                    } else {
                        data.sort_by_cached_key(|p| p.command.to_lowercase());
                    }
                } else if sort_descending {
                    data.sort_by_cached_key(|p| Reverse(p.name.to_lowercase()));
                } else {
                    data.sort_by_cached_key(|p| p.name.to_lowercase());
                }
            }
            ProcWidgetColumn::ReadPerSecond => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                if sort_descending {
                    data.sort_by_key(|a| Reverse(a.read_bytes_per_sec));
                } else {
                    data.sort_by_key(|a| a.read_bytes_per_sec);
                }
            }
            ProcWidgetColumn::WritePerSecond => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                if sort_descending {
                    data.sort_by_key(|a| Reverse(a.write_bytes_per_sec));
                } else {
                    data.sort_by_key(|a| a.write_bytes_per_sec);
                }
            }
            ProcWidgetColumn::TotalRead => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                if sort_descending {
                    data.sort_by_key(|a| Reverse(a.total_read_bytes));
                } else {
                    data.sort_by_key(|a| a.total_read_bytes);
                }
            }
            ProcWidgetColumn::TotalWrite => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                if sort_descending {
                    data.sort_by_key(|a| Reverse(a.total_write_bytes));
                } else {
                    data.sort_by_key(|a| a.total_write_bytes);
                }
            }
            ProcWidgetColumn::State => {
                data.sort_by_cached_key(|p| p.name.to_lowercase());
                if sort_descending {
                    data.sort_by_cached_key(|p| Reverse(p.process_state.0.to_lowercase()));
                } else {
                    data.sort_by_cached_key(|p| p.process_state.0.to_lowercase());
                }
            }
            ProcWidgetColumn::User => {
                #[cfg(target_family = "unix")]
                {
                    data.sort_by_cached_key(|p| p.name.to_lowercase());
                    if sort_descending {
                        data.sort_by_cached_key(|p| Reverse(p.user.to_lowercase()));
                    } else {
                        data.sort_by_cached_key(|p| p.user.to_lowercase());
                    }
                }
            }
        }
    }

    /// Basically, anything "alphabetical" should sort in ascending order by default. This also includes something like
    /// PID, as one would probably want PID to sort by default starting from 0 or 1.
    pub fn default_sort_order(&self) -> SortOrder {
        match self {
            ProcWidgetColumn::PidOrCount { is_count: true }
            | ProcWidgetColumn::CpuPercent
            | ProcWidgetColumn::ReadPerSecond
            | ProcWidgetColumn::WritePerSecond
            | ProcWidgetColumn::TotalRead
            | ProcWidgetColumn::TotalWrite
            | ProcWidgetColumn::Memory { .. } => SortOrder::Descending,

            ProcWidgetColumn::PidOrCount { is_count: false }
            | ProcWidgetColumn::ProcNameOrCommand { .. }
            | ProcWidgetColumn::State
            | ProcWidgetColumn::User => SortOrder::Ascending,
        }
    }
}

impl TableComponentHeader for ProcWidgetColumn {
    fn header_text(&self) -> &CellContent {
        match self {
            ProcWidgetColumn::CpuPercent => &Self::SHORTCUT_CPU_PERCENT,
            ProcWidgetColumn::Memory { show_percentage } => {
                if *show_percentage {
                    &Self::SHORTCUT_MEM_PERCENT
                } else {
                    &Self::SHORTCUT_MEM
                }
            }
            ProcWidgetColumn::PidOrCount { is_count } => {
                if *is_count {
                    &Self::COUNT
                } else {
                    &Self::SHORTCUT_PID
                }
            }
            ProcWidgetColumn::ProcNameOrCommand { is_command } => {
                if *is_command {
                    &Self::SHORTCUT_COMMAND
                } else {
                    &Self::SHORTCUT_PROCESS_NAME
                }
            }
            ProcWidgetColumn::ReadPerSecond => &Self::READS_PER_SECOND,
            ProcWidgetColumn::WritePerSecond => &Self::WRITES_PER_SECOND,
            ProcWidgetColumn::TotalRead => &Self::TOTAL_READ,
            ProcWidgetColumn::TotalWrite => &Self::TOTAL_WRITE,
            ProcWidgetColumn::State => &Self::STATE,
            ProcWidgetColumn::User => &Self::USER,
        }
    }
}
