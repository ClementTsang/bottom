pub mod process_columns;
pub mod process_data;
pub mod query;
mod sort_table;

use std::{borrow::Cow, collections::BTreeMap};

use hashbrown::{HashMap, HashSet};
use indexmap::IndexSet;
use itertools::Itertools;
pub use process_columns::*;
pub use process_data::*;
use query::{parse_query, ProcessQuery};
use sort_table::SortTableColumn;

use crate::{
    app::{
        data_farmer::{DataCollection, ProcessData},
        AppConfigFields, AppSearchState,
    },
    canvas::components::data_table::{
        Column, ColumnHeader, ColumnWidthBounds, DataTable, DataTableColumn, DataTableProps,
        DataTableStyling, SortColumn, SortDataTable, SortDataTableProps, SortOrder, SortsRow,
    },
    data_collection::processes::{Pid, ProcessHarvest},
    options::config::style::Styles,
};

/// ProcessSearchState only deals with process' search's current settings and
/// state.
pub struct ProcessSearchState {
    pub search_state: AppSearchState,
    pub is_ignoring_case: bool,
    pub is_searching_whole_word: bool,
    pub is_searching_with_regex: bool,
}

impl Default for ProcessSearchState {
    fn default() -> Self {
        ProcessSearchState {
            search_state: AppSearchState::default(),
            is_ignoring_case: true,
            is_searching_whole_word: false,
            is_searching_with_regex: false,
        }
    }
}

impl ProcessSearchState {
    pub fn search_toggle_ignore_case(&mut self) {
        self.is_ignoring_case = !self.is_ignoring_case;
    }

    pub fn search_toggle_whole_word(&mut self) {
        self.is_searching_whole_word = !self.is_searching_whole_word;
    }

    pub fn search_toggle_regex(&mut self) {
        self.is_searching_with_regex = !self.is_searching_with_regex;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcWidgetMode {
    Tree {
        collapsed_pids: HashSet<Pid>,
        collapse: bool,
    },
    Grouped,
    Normal,
}

type ProcessTable = SortDataTable<ProcWidgetData, ProcColumn>;
type SortTable = DataTable<Cow<'static, str>, SortTableColumn>;
type StringPidMap = HashMap<String, Vec<Pid>>;

fn make_column(column: ProcColumn) -> SortColumn<ProcColumn> {
    use ProcColumn::*;

    match column {
        CpuPercent => SortColumn::new(CpuPercent).default_descending(),
        MemValue => SortColumn::new(MemValue).default_descending(),
        MemPercent => SortColumn::new(MemPercent).default_descending(),
        Pid => SortColumn::new(Pid),
        Count => SortColumn::new(Count),
        Name => SortColumn::soft(Name, Some(0.3)),
        Command => SortColumn::soft(Command, Some(0.3)),
        ReadPerSecond => SortColumn::hard(ReadPerSecond, 8).default_descending(),
        WritePerSecond => SortColumn::hard(WritePerSecond, 8).default_descending(),
        TotalRead => SortColumn::hard(TotalRead, 8).default_descending(),
        TotalWrite => SortColumn::hard(TotalWrite, 8).default_descending(),
        User => SortColumn::soft(User, Some(0.05)),
        State => SortColumn::hard(State, 9),
        Time => SortColumn::new(Time),
        #[cfg(feature = "gpu")]
        GpuMemValue => SortColumn::new(GpuMemValue).default_descending(),
        #[cfg(feature = "gpu")]
        GpuMemPercent => SortColumn::new(GpuMemPercent).default_descending(),
        #[cfg(feature = "gpu")]
        GpuUtilPercent => SortColumn::new(GpuUtilPercent).default_descending(),
    }
}

#[derive(Clone, Copy, Default)]
pub struct ProcTableConfig {
    pub is_case_sensitive: bool,
    pub is_match_whole_word: bool,
    pub is_use_regex: bool,
    pub show_memory_as_values: bool,
    pub is_command: bool,
}

/// A hacky workaround for now.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ProcWidgetColumn {
    PidOrCount,
    ProcNameOrCommand,
    Cpu,
    Mem,
    ReadPerSecond,
    WritePerSecond,
    TotalRead,
    TotalWrite,
    User,
    State,
    Time,
    #[cfg(feature = "gpu")]
    GpuMem,
    #[cfg(feature = "gpu")]
    GpuUtil,
}

// This is temporary. Switch back to `ProcColumn` later!

pub struct ProcWidgetState {
    pub mode: ProcWidgetMode,

    /// The state of the search box.
    pub proc_search: ProcessSearchState,

    /// The state of the main table.
    pub table: ProcessTable,

    /// The state of the togglable table that controls sorting.
    pub sort_table: SortTable,

    /// The internal column mapping as an [`IndexSet`], to allow us to do quick
    /// mappings of column type -> index.
    pub column_mapping: IndexSet<ProcWidgetColumn>,

    /// A name-to-pid mapping.
    pub id_pid_map: StringPidMap,

    /// The default sort index.
    default_sort_index: usize,

    /// The default sort order.
    default_sort_order: SortOrder,

    pub is_sort_open: bool,
    pub force_rerender: bool,
    pub force_update_data: bool,
}

impl ProcWidgetState {
    fn new_sort_table(config: &AppConfigFields, palette: &Styles) -> SortTable {
        const COLUMNS: [Column<SortTableColumn>; 1] = [Column::hard(SortTableColumn, 7)];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: true,
            is_basic: false,
            show_table_scroll_position: false,
            show_current_entry_when_unfocused: false,
        };
        let styling = DataTableStyling::from_palette(palette);

        DataTable::new(COLUMNS, props, styling)
    }

    fn new_process_table(
        config: &AppConfigFields, colours: &Styles, columns: Vec<SortColumn<ProcColumn>>,
        default_index: usize, default_order: SortOrder,
    ) -> ProcessTable {
        let inner_props = DataTableProps {
            title: Some(" Processes ".into()),
            table_gap: config.table_gap,
            left_to_right: true,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };
        let props = SortDataTableProps {
            inner: inner_props,
            sort_index: default_index,
            order: default_order,
        };
        let styling = DataTableStyling::from_palette(colours);

        DataTable::new_sortable(columns, props, styling)
    }

    pub fn new(
        config: &AppConfigFields, mode: ProcWidgetMode, table_config: ProcTableConfig,
        colours: &Styles, config_columns: &Option<IndexSet<ProcWidgetColumn>>,
    ) -> Self {
        let process_search_state = {
            let mut pss = ProcessSearchState::default();

            if table_config.is_case_sensitive {
                // By default it's off.
                pss.search_toggle_ignore_case();
            }
            if table_config.is_match_whole_word {
                pss.search_toggle_whole_word();
            }
            if table_config.is_use_regex {
                pss.search_toggle_regex();
            }

            pss
        };

        let columns: Vec<SortColumn<ProcColumn>> = {
            use ProcColumn::*;

            let is_count = matches!(mode, ProcWidgetMode::Grouped);
            let is_command = table_config.is_command;
            let mem_as_values = table_config.show_memory_as_values;

            match config_columns {
                Some(columns) if !columns.is_empty() => columns
                    .into_iter()
                    .map(|c| {
                        let col = match c {
                            ProcWidgetColumn::PidOrCount => {
                                if is_count {
                                    Count
                                } else {
                                    Pid
                                }
                            }
                            ProcWidgetColumn::ProcNameOrCommand => {
                                if is_command {
                                    Command
                                } else {
                                    Name
                                }
                            }
                            ProcWidgetColumn::Cpu => CpuPercent,
                            ProcWidgetColumn::Mem => {
                                if mem_as_values {
                                    MemValue
                                } else {
                                    MemPercent
                                }
                            }
                            ProcWidgetColumn::ReadPerSecond => ReadPerSecond,
                            ProcWidgetColumn::WritePerSecond => WritePerSecond,
                            ProcWidgetColumn::TotalRead => TotalRead,
                            ProcWidgetColumn::TotalWrite => TotalWrite,
                            ProcWidgetColumn::User => User,
                            ProcWidgetColumn::State => State,
                            ProcWidgetColumn::Time => Time,
                            #[cfg(feature = "gpu")]
                            ProcWidgetColumn::GpuMem => {
                                if mem_as_values {
                                    GpuMemValue
                                } else {
                                    GpuMemPercent
                                }
                            }
                            #[cfg(feature = "gpu")]
                            ProcWidgetColumn::GpuUtil => GpuUtilPercent,
                        };

                        make_column(col)
                    })
                    .collect(),
                _ => {
                    let default_columns = [
                        if is_count { Count } else { Pid },
                        if is_command { Command } else { Name },
                        CpuPercent,
                        if mem_as_values { MemValue } else { MemPercent },
                        ReadPerSecond,
                        WritePerSecond,
                        TotalRead,
                        TotalWrite,
                        User,
                        State,
                        Time,
                    ];

                    default_columns.into_iter().map(make_column).collect()
                }
            }
        };

        let column_mapping = columns
            .iter()
            .map(|col| {
                use ProcColumn::*;

                match col.inner() {
                    CpuPercent => ProcWidgetColumn::Cpu,
                    MemValue | MemPercent => ProcWidgetColumn::Mem,
                    Pid | Count => ProcWidgetColumn::PidOrCount,
                    Name | Command => ProcWidgetColumn::ProcNameOrCommand,
                    ReadPerSecond => ProcWidgetColumn::ReadPerSecond,
                    WritePerSecond => ProcWidgetColumn::WritePerSecond,
                    TotalRead => ProcWidgetColumn::TotalRead,
                    TotalWrite => ProcWidgetColumn::TotalWrite,
                    State => ProcWidgetColumn::State,
                    User => ProcWidgetColumn::User,
                    Time => ProcWidgetColumn::Time,
                    #[cfg(feature = "gpu")]
                    GpuMemValue | GpuMemPercent => ProcWidgetColumn::GpuMem,
                    #[cfg(feature = "gpu")]
                    GpuUtilPercent => ProcWidgetColumn::GpuUtil,
                }
            })
            .collect::<IndexSet<_>>();

        let (default_sort_index, default_sort_order) =
            if matches!(mode, ProcWidgetMode::Tree { .. }) {
                if let Some(index) = column_mapping.get_index_of(&ProcWidgetColumn::PidOrCount) {
                    (index, columns[index].default_order)
                } else {
                    (0, columns[0].default_order)
                }
            } else if let Some(index) = column_mapping.get_index_of(&ProcWidgetColumn::Cpu) {
                (index, columns[index].default_order)
            } else {
                (0, columns[0].default_order)
            };

        let sort_table = Self::new_sort_table(config, colours);
        let table = Self::new_process_table(
            config,
            colours,
            columns,
            default_sort_index,
            default_sort_order,
        );

        let id_pid_map = HashMap::default();

        let mut table = ProcWidgetState {
            proc_search: process_search_state,
            table,
            sort_table,
            id_pid_map,
            column_mapping,
            is_sort_open: false,
            mode,
            force_rerender: true,
            force_update_data: false,
            default_sort_index,
            default_sort_order,
        };
        table.sort_table.set_data(table.column_text());

        table
    }

    pub fn is_using_command(&self) -> bool {
        self.column_mapping
            .get_index_of(&ProcWidgetColumn::ProcNameOrCommand)
            .and_then(|index| {
                self.table
                    .columns
                    .get(index)
                    .map(|col| matches!(col.inner(), ProcColumn::Command))
            })
            .unwrap_or(false)
    }

    pub fn is_mem_percent(&self) -> bool {
        self.column_mapping
            .get_index_of(&ProcWidgetColumn::Mem)
            .and_then(|index| self.table.columns.get(index))
            .map(|col| matches!(col.inner(), ProcColumn::MemPercent))
            .unwrap_or(false)
    }

    fn get_query(&self) -> &Option<ProcessQuery> {
        if self.proc_search.search_state.is_invalid_or_blank_search() {
            &None
        } else {
            &self.proc_search.search_state.query
        }
    }

    /// Update the current table data.
    ///
    /// This function *only* updates the displayed process data. If there is a
    /// need to update the actual *stored* data, call it before this
    /// function.
    pub fn set_table_data(&mut self, data_collection: &DataCollection) {
        let data = match &self.mode {
            ProcWidgetMode::Grouped | ProcWidgetMode::Normal => {
                self.get_normal_data(&data_collection.process_data.process_harvest)
            }
            ProcWidgetMode::Tree {
                collapsed_pids,
                collapse,
            } => self.get_tree_data(collapsed_pids, data_collection, collapse),
        };
        self.table.set_data(data);
    }

    fn get_tree_data(
        &self, collapsed_pids: &HashSet<Pid>, data_collection: &DataCollection, collapse: &bool,
    ) -> Vec<ProcWidgetData> {
        const BRANCH_END: char = '└';
        const BRANCH_SPLIT: char = '├';
        const BRANCH_HORIZONTAL: char = '─';
        const SPACED_BRANCH_VERTICAL: &str = "│  ";

        let search_query = self.get_query();
        let is_using_command = self.is_using_command();
        let is_mem_percent = self.is_mem_percent();

        let ProcessData {
            process_harvest,
            process_parent_mapping,
            orphan_pids,
            ..
        } = &data_collection.process_data;

        // Only keep a set of the kept PIDs.
        let kept_pids = data_collection
            .process_data
            .process_harvest
            .iter()
            .filter_map(|(pid, process)| {
                if search_query
                    .as_ref()
                    .map(|q| q.check(process, is_using_command))
                    .unwrap_or(true)
                {
                    Some(*pid)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

        #[inline]
        fn is_ancestor_shown(
            current_process: &ProcessHarvest, kept_pids: &HashSet<Pid>,
            process_harvest: &BTreeMap<Pid, ProcessHarvest>,
        ) -> bool {
            if let Some(ppid) = current_process.parent_pid {
                if kept_pids.contains(&ppid) {
                    true
                } else if let Some(parent) = process_harvest.get(&ppid) {
                    is_ancestor_shown(parent, kept_pids, process_harvest)
                } else {
                    false
                }
            } else {
                false
            }
        }

        // A process is shown under the filtered tree if at least one of these
        // conditions hold:
        // - The process itself matches.
        // - The process contains some descendant that matches.
        // - The process's parent (and only parent, not any ancestor) matches.
        let filtered_tree = {
            let mut filtered_tree: HashMap<Pid, Vec<Pid>> = HashMap::default();

            // We do a simple DFS traversal to build our filtered parent-to-tree mappings.
            let mut visited_pids: HashMap<Pid, bool> = HashMap::default();
            let mut stack = orphan_pids
                .iter()
                .filter_map(|process| process_harvest.get(process))
                .collect_vec();

            while let Some(process) = stack.last() {
                let is_process_matching = kept_pids.contains(&process.pid);

                if let Some(children_pids) = process_parent_mapping.get(&process.pid) {
                    if children_pids
                        .iter()
                        .all(|pid| visited_pids.contains_key(pid))
                    {
                        let shown_children = children_pids
                            .iter()
                            .filter(|pid| visited_pids.get(*pid).copied().unwrap_or(false))
                            .collect_vec();

                        // Show the entry if it is:
                        // - Matches the filter.
                        // - Has at least one child (doesn't have to be direct) that matches the
                        //   filter.
                        // - Is the child of a shown process.
                        let is_shown = is_process_matching
                            || !shown_children.is_empty()
                            || is_ancestor_shown(process, &kept_pids, process_harvest);
                        visited_pids.insert(process.pid, is_shown);

                        if is_shown {
                            filtered_tree.insert(
                                process.pid,
                                shown_children
                                    .into_iter()
                                    .filter_map(|pid| {
                                        process_harvest.get(pid).map(|process| process.pid)
                                    })
                                    .collect_vec(),
                            );
                        }

                        stack.pop();
                    } else {
                        children_pids
                            .iter()
                            .filter_map(|process| process_harvest.get(process))
                            .rev()
                            .for_each(|process| {
                                stack.push(process);
                            });
                    }
                } else {
                    let is_shown = is_process_matching
                        || is_ancestor_shown(process, &kept_pids, process_harvest);

                    if is_shown {
                        filtered_tree.insert(process.pid, vec![]);
                    }

                    visited_pids.insert(process.pid, is_shown);
                    stack.pop();
                }
            }

            filtered_tree
        };

        let mut data = vec![];
        let mut prefixes = vec![];
        let mut stack = orphan_pids
            .iter()
            .filter_map(|pid| {
                if filtered_tree.contains_key(pid) {
                    process_harvest.get(pid).map(|process| {
                        ProcWidgetData::from_data(process, is_using_command, is_mem_percent)
                    })
                } else {
                    None
                }
            })
            .collect_vec();

        stack.sort_unstable_by_key(|p| p.pid);

        let column = self.table.columns.get(self.table.sort_index()).unwrap();
        sort_skip_pid_asc(column.inner(), &mut stack, self.table.order());

        let mut length_stack = vec![stack.len()];
        stack.reverse();

        while let (Some(process), Some(siblings_left)) = (stack.pop(), length_stack.last_mut()) {
            *siblings_left -= 1;

            let disabled = !kept_pids.contains(&process.pid);
            let is_last = *siblings_left == 0;

            if collapsed_pids.contains(&process.pid).eq(&!collapse) {
                let mut summed_process = process.clone();
                let mut prefix = if prefixes.is_empty() {
                    String::default()
                } else {
                    format!(
                        "{}{}{} ",
                        prefixes.join(""),
                        if is_last { BRANCH_END } else { BRANCH_SPLIT },
                        BRANCH_HORIZONTAL
                    )
                };

                if let Some(children_pids) = filtered_tree.get(&process.pid) {
                    let mut sum_queue = children_pids
                        .iter()
                        .filter_map(|child| {
                            process_harvest.get(child).map(|p| {
                                ProcWidgetData::from_data(p, is_using_command, is_mem_percent)
                            })
                        })
                        .collect_vec();

                    while let Some(process) = sum_queue.pop() {
                        summed_process.add(&process);

                        if let Some(pids) = filtered_tree.get(&process.pid) {
                            sum_queue.extend(pids.iter().filter_map(|child| {
                                process_harvest.get(child).map(|p| {
                                    ProcWidgetData::from_data(p, is_using_command, is_mem_percent)
                                })
                            }));
                        }
                    }
                    if !children_pids.is_empty() {
                        prefix = if prefixes.is_empty() {
                            "+ ".to_string()
                        } else {
                            format!(
                                "{}{}{} + ",
                                prefixes.join(""),
                                if is_last { BRANCH_END } else { BRANCH_SPLIT },
                                BRANCH_HORIZONTAL
                            )
                        };
                    }
                }

                data.push(summed_process.prefix(Some(prefix)).disabled(disabled));
            } else {
                let prefix = if prefixes.is_empty() {
                    String::default()
                } else {
                    format!(
                        "{}{}{} ",
                        prefixes.join(""),
                        if is_last { BRANCH_END } else { BRANCH_SPLIT },
                        BRANCH_HORIZONTAL
                    )
                };
                let pid = process.pid;
                data.push(process.prefix(Some(prefix)).disabled(disabled));

                if let Some(children_pids) = filtered_tree.get(&pid) {
                    if prefixes.is_empty() {
                        prefixes.push("");
                    } else {
                        prefixes.push(if is_last {
                            "   "
                        } else {
                            SPACED_BRANCH_VERTICAL
                        });
                    }

                    let mut children = children_pids
                        .iter()
                        .filter_map(|child_pid| {
                            process_harvest.get(child_pid).map(|p| {
                                ProcWidgetData::from_data(p, is_using_command, is_mem_percent)
                            })
                        })
                        .collect_vec();

                    column.sort_by(&mut children, self.table.order().rev());

                    length_stack.push(children.len());
                    stack.extend(children);
                }
            }

            while let Some(children_left) = length_stack.last() {
                if *children_left == 0 {
                    length_stack.pop();
                    prefixes.pop();
                } else {
                    break;
                }
            }
        }

        data
    }

    fn get_normal_data(
        &mut self, process_harvest: &BTreeMap<Pid, ProcessHarvest>,
    ) -> Vec<ProcWidgetData> {
        let search_query = self.get_query();
        let is_using_command = self.is_using_command();
        let is_mem_percent = self.is_mem_percent();

        let filtered_iter = process_harvest.values().filter(|process| {
            search_query
                .as_ref()
                .map(|query| query.check(process, is_using_command))
                .unwrap_or(true)
        });

        let mut id_pid_map: HashMap<String, Vec<Pid>> = HashMap::default();
        let mut filtered_data: Vec<ProcWidgetData> = if let ProcWidgetMode::Grouped = self.mode {
            let mut id_process_mapping: HashMap<&String, ProcessHarvest> = HashMap::default();
            for process in filtered_iter {
                let id = if is_using_command {
                    &process.command
                } else {
                    &process.name
                };
                let pid = process.pid;

                if let Some(entry) = id_pid_map.get_mut(id) {
                    entry.push(pid);
                } else {
                    id_pid_map.insert(id.clone(), vec![pid]);
                }

                if let Some(grouped_process_harvest) = id_process_mapping.get_mut(id) {
                    grouped_process_harvest.add(process);
                } else {
                    // FIXME: [PERF] could maybe eliminate an allocation here in the grouped mode...
                    // or maybe just avoid the entire transformation step, making an alloc fine.
                    id_process_mapping.insert(id, process.clone());
                }
            }

            id_process_mapping
                .values()
                .map(|process| {
                    let id = if is_using_command {
                        &process.command
                    } else {
                        &process.name
                    };

                    let num_similar = id_pid_map.get(id).map(|val| val.len()).unwrap_or(1) as u64;

                    ProcWidgetData::from_data(process, is_using_command, is_mem_percent)
                        .num_similar(num_similar)
                })
                .collect()
        } else {
            filtered_iter
                .map(|process| ProcWidgetData::from_data(process, is_using_command, is_mem_percent))
                .collect()
        };

        self.id_pid_map = id_pid_map;

        if let Some(column) = self.table.columns.get(self.table.sort_index()) {
            sort_skip_pid_asc(column.inner(), &mut filtered_data, self.table.order());
        }

        filtered_data
    }

    #[inline(always)]
    fn get_mut_proc_col(&mut self, index: usize) -> Option<&mut ProcColumn> {
        self.table.columns.get_mut(index).map(|col| col.inner_mut())
    }

    pub fn toggle_mem_percentage(&mut self) {
        if let Some(index) = self.column_mapping.get_index_of(&ProcWidgetColumn::Mem) {
            if let Some(mem) = self.get_mut_proc_col(index) {
                match mem {
                    ProcColumn::MemValue => {
                        *mem = ProcColumn::MemPercent;
                    }
                    ProcColumn::MemPercent => {
                        *mem = ProcColumn::MemValue;
                    }
                    _ => unreachable!(),
                }

                self.sort_table.set_data(self.column_text());
                self.force_data_update();
            }
        }
        #[cfg(feature = "gpu")]
        if let Some(index) = self.column_mapping.get_index_of(&ProcWidgetColumn::GpuMem) {
            if let Some(mem) = self.get_mut_proc_col(index) {
                match mem {
                    ProcColumn::GpuMemValue => {
                        *mem = ProcColumn::GpuMemPercent;
                    }
                    ProcColumn::GpuMemPercent => {
                        *mem = ProcColumn::GpuMemValue;
                    }
                    _ => unreachable!(),
                }

                self.sort_table.set_data(self.column_text());
                self.force_data_update();
            }
        }
    }

    /// Forces an update of the data stored.
    #[inline]
    pub fn force_data_update(&mut self) {
        self.force_update_data = true;
    }

    /// Forces an entire rerender and update of the data stored.
    #[inline]
    pub fn force_rerender_and_update(&mut self) {
        self.force_rerender = true;
        self.force_update_data = true;
    }

    /// Marks the selected column as hidden, and automatically resets the
    /// selected column to the default sort index and order.
    fn hide_column(&mut self, column: ProcWidgetColumn) {
        if let Some(index) = self.column_mapping.get_index_of(&column) {
            if let Some(col) = self.table.columns.get_mut(index) {
                col.is_hidden = true;

                if self.table.sort_index() == index {
                    self.table.set_sort_index(self.default_sort_index);
                    self.table.set_order(self.default_sort_order);
                }
            }
        }
    }

    /// Marks the selected column as shown.
    fn show_column(&mut self, column: ProcWidgetColumn) {
        if let Some(index) = self.column_mapping.get_index_of(&column) {
            if let Some(col) = self.table.columns.get_mut(index) {
                col.is_hidden = false;
            }
        }
    }

    /// Select a column. If the column is already selected, then just toggle the
    /// sort order.
    pub fn select_column(&mut self, column: ProcWidgetColumn) {
        if let Some(index) = self.column_mapping.get_index_of(&column) {
            self.table.set_sort_index(index);
            self.force_data_update();
        }
    }

    pub fn toggle_current_tree_branch_entry(&mut self) {
        if let ProcWidgetMode::Tree { collapsed_pids, .. } = &mut self.mode {
            if let Some(process) = self.table.current_item() {
                let pid = process.pid;

                if !collapsed_pids.remove(&pid) {
                    collapsed_pids.insert(pid);
                }
                self.force_data_update();
            }
        }
    }

    pub fn toggle_command(&mut self) {
        if let Some(index) = self
            .column_mapping
            .get_index_of(&ProcWidgetColumn::ProcNameOrCommand)
        {
            if let Some(col) = self.table.columns.get_mut(index) {
                let inner = col.inner_mut();
                match inner {
                    ProcColumn::Name => {
                        *inner = ProcColumn::Command;
                        if let ColumnWidthBounds::Soft { max_percentage, .. } = col.bounds_mut() {
                            *max_percentage = Some(0.5);
                        }
                    }
                    ProcColumn::Command => {
                        *inner = ProcColumn::Name;
                        if let ColumnWidthBounds::Soft { max_percentage, .. } = col.bounds_mut() {
                            *max_percentage = match self.mode {
                                ProcWidgetMode::Tree { .. } => Some(0.5),
                                ProcWidgetMode::Grouped | ProcWidgetMode::Normal => Some(0.3),
                            };
                        }
                    }
                    _ => unreachable!(),
                }
                self.sort_table.set_data(self.column_text());
                self.force_rerender_and_update();
            }
        }
    }

    /// Toggles the appropriate columns/settings when tab is pressed.
    ///
    /// If count is enabled, we should set the mode to
    /// [`ProcWidgetMode::Grouped`], and switch off the User and State
    /// columns. We should also move the user off of the columns if they were
    /// selected, as those columns are now hidden (handled by internal
    /// method calls), and go back to the "defaults".
    ///
    /// Otherwise, if count is disabled, then if the columns exist, the User and
    /// State columns should be re-enabled, and the mode switched to
    /// [`ProcWidgetMode::Normal`].
    pub fn toggle_tab(&mut self) {
        if !matches!(self.mode, ProcWidgetMode::Tree { .. }) {
            if let Some(index) = self
                .column_mapping
                .get_index_of(&ProcWidgetColumn::PidOrCount)
            {
                if let Some(sort_col) = self.table.columns.get_mut(index) {
                    let col = sort_col.inner_mut();
                    match col {
                        ProcColumn::Pid => {
                            *col = ProcColumn::Count;
                            sort_col.default_order = SortOrder::Descending;

                            self.hide_column(ProcWidgetColumn::User);
                            self.hide_column(ProcWidgetColumn::State);
                            self.mode = ProcWidgetMode::Grouped;
                        }
                        ProcColumn::Count => {
                            *col = ProcColumn::Pid;
                            sort_col.default_order = SortOrder::Ascending;

                            self.show_column(ProcWidgetColumn::User);
                            self.show_column(ProcWidgetColumn::State);
                            self.mode = ProcWidgetMode::Normal;
                        }
                        _ => unreachable!(),
                    }

                    self.sort_table.set_data(self.column_text());
                    self.force_rerender_and_update();
                }
            }
        }
    }

    pub fn column_text(&self) -> Vec<Cow<'static, str>> {
        self.table
            .columns
            .iter()
            .filter(|c| !c.is_hidden)
            .map(|c| c.inner().text())
            .collect::<Vec<_>>()
    }

    pub fn cursor_char_index(&self) -> usize {
        self.proc_search.search_state.grapheme_cursor.cur_cursor()
    }

    pub fn is_search_enabled(&self) -> bool {
        self.proc_search.search_state.is_enabled
    }

    pub fn current_search_query(&self) -> &str {
        &self.proc_search.search_state.current_search_query
    }

    pub fn update_query(&mut self) {
        if self
            .proc_search
            .search_state
            .current_search_query
            .is_empty()
        {
            self.proc_search.search_state.is_blank_search = true;
            self.proc_search.search_state.is_invalid_search = false;
            self.proc_search.search_state.error_message = None;
        } else {
            match parse_query(
                &self.proc_search.search_state.current_search_query,
                self.proc_search.is_searching_whole_word,
                self.proc_search.is_ignoring_case,
                self.proc_search.is_searching_with_regex,
            ) {
                Ok(parsed_query) => {
                    self.proc_search.search_state.query = Some(parsed_query);
                    self.proc_search.search_state.is_blank_search = false;
                    self.proc_search.search_state.is_invalid_search = false;
                    self.proc_search.search_state.error_message = None;
                }
                Err(err) => {
                    self.proc_search.search_state.is_blank_search = false;
                    self.proc_search.search_state.is_invalid_search = true;
                    self.proc_search.search_state.error_message = Some(err.to_string());
                }
            }
        }
        self.table.state.display_start_index = 0;
        self.table.state.current_index = 0;

        // Update the internal sizes too.
        self.proc_search.search_state.update_sizes();

        self.force_data_update();
    }

    pub fn clear_search(&mut self) {
        self.proc_search.search_state.reset();
        self.force_data_update();
    }

    pub fn search_walk_forward(&mut self) {
        self.proc_search.search_state.walk_forward();
    }

    pub fn search_walk_back(&mut self) {
        self.proc_search.search_state.walk_backward();
    }

    /// Sets the [`ProcWidgetState`]'s current sort index to whatever was in the
    /// sort table if possible, then closes the sort table.
    pub(crate) fn use_sort_table_value(&mut self) {
        self.table.set_sort_index(self.sort_table.current_index());

        self.is_sort_open = false;
        self.force_rerender_and_update();
    }

    #[cfg(test)]
    pub(crate) fn test_equality(&self, other: &Self) -> bool {
        self.mode == other.mode
            && self.proc_search.is_ignoring_case == other.proc_search.is_ignoring_case
            && self.proc_search.is_searching_whole_word == other.proc_search.is_searching_whole_word
            && self.proc_search.is_searching_with_regex == other.proc_search.is_searching_with_regex
            && self
                .table
                .columns
                .iter()
                .map(|c| c.header())
                .collect::<Vec<_>>()
                == other
                    .table
                    .columns
                    .iter()
                    .map(|c| c.header())
                    .collect::<Vec<_>>()
    }
}

#[inline]
fn sort_skip_pid_asc(column: &ProcColumn, data: &mut [ProcWidgetData], order: SortOrder) {
    let descending = matches!(order, SortOrder::Descending);
    match column {
        ProcColumn::Pid if !descending => {}
        _ => {
            column.sort_data(data, descending);
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;
    use crate::widgets::MemUsage;

    #[test]
    fn test_proc_sort() {
        let a = ProcWidgetData {
            pid: 1,
            ppid: None,
            id: "A".into(),
            cpu_usage_percent: 0.0,
            mem_usage: MemUsage::Percent(1.1),
            rps: 0,
            wps: 0,
            total_read: 0,
            total_write: 0,
            process_state: "N/A".to_string(),
            process_char: '?',
            #[cfg(target_family = "unix")]
            user: "root".to_string(),
            #[cfg(not(target_family = "unix"))]
            user: "N/A".to_string(),
            num_similar: 0,
            disabled: false,
            time: Duration::from_secs(0),
            #[cfg(feature = "gpu")]
            gpu_mem_usage: MemUsage::Percent(1.1),
            #[cfg(feature = "gpu")]
            gpu_usage: 0,
        };

        let b = ProcWidgetData {
            pid: 2,
            ppid: Some(1),
            id: "B".into(),
            cpu_usage_percent: 1.1,
            mem_usage: MemUsage::Percent(2.2),
            ..(a.clone())
        };

        let c = ProcWidgetData {
            pid: 3,
            ppid: Some(1),
            id: "C".into(),
            cpu_usage_percent: 2.2,
            mem_usage: MemUsage::Percent(0.0),
            ..(a.clone())
        };

        let d = ProcWidgetData {
            pid: 4,
            ppid: Some(2),
            id: "D".into(),
            cpu_usage_percent: 0.0,
            mem_usage: MemUsage::Percent(0.0),
            ..(a.clone())
        };
        let mut data = vec![d.clone(), b.clone(), c.clone(), a.clone()];

        // Assume we had sorted over by pid.
        data.sort_by_key(|p| p.pid);
        sort_skip_pid_asc(&ProcColumn::CpuPercent, &mut data, SortOrder::Descending);
        assert_eq!(
            [&c, &b, &a, &d].iter().map(|d| (d.pid)).collect::<Vec<_>>(),
            data.iter().map(|d| (d.pid)).collect::<Vec<_>>(),
        );

        // Note that the PID ordering for ties is still ascending.
        data.sort_by_key(|p| p.pid);
        sort_skip_pid_asc(&ProcColumn::CpuPercent, &mut data, SortOrder::Ascending);
        assert_eq!(
            [&a, &d, &b, &c].iter().map(|d| (d.pid)).collect::<Vec<_>>(),
            data.iter().map(|d| (d.pid)).collect::<Vec<_>>(),
        );

        data.sort_by_key(|p| p.pid);
        sort_skip_pid_asc(&ProcColumn::MemPercent, &mut data, SortOrder::Descending);
        assert_eq!(
            [&b, &a, &c, &d].iter().map(|d| (d.pid)).collect::<Vec<_>>(),
            data.iter().map(|d| (d.pid)).collect::<Vec<_>>(),
        );

        // Note that the PID ordering for ties is still ascending.
        data.sort_by_key(|p| p.pid);
        sort_skip_pid_asc(&ProcColumn::MemPercent, &mut data, SortOrder::Ascending);
        assert_eq!(
            [&c, &d, &a, &b].iter().map(|d| (d.pid)).collect::<Vec<_>>(),
            data.iter().map(|d| (d.pid)).collect::<Vec<_>>(),
        );
    }

    fn get_columns(table: &ProcessTable) -> Vec<ProcColumn> {
        table
            .columns
            .iter()
            .filter_map(|c| {
                if c.is_hidden() {
                    None
                } else {
                    Some(*c.inner())
                }
            })
            .collect::<Vec<_>>()
    }

    fn init_state(table_config: ProcTableConfig, columns: &[ProcWidgetColumn]) -> ProcWidgetState {
        let config = AppConfigFields::default();
        let styling = Styles::default();
        let columns = Some(columns.iter().cloned().collect());

        ProcWidgetState::new(
            &config,
            ProcWidgetMode::Normal,
            table_config,
            &styling,
            &columns,
        )
    }

    fn init_default_state(columns: &[ProcWidgetColumn]) -> ProcWidgetState {
        init_state(ProcTableConfig::default(), columns)
    }

    #[test]
    fn custom_columns() {
        let init_columns = vec![
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::ProcNameOrCommand,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
        ];
        let columns = vec![
            ProcColumn::Pid,
            ProcColumn::Name,
            ProcColumn::MemPercent,
            ProcColumn::State,
        ];
        let state = init_default_state(&init_columns);
        assert_eq!(get_columns(&state.table), columns);
    }

    #[test]
    fn toggle_count_pid() {
        let init_columns = [
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::ProcNameOrCommand,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
        ];
        let original_columns = vec![
            ProcColumn::Pid,
            ProcColumn::Name,
            ProcColumn::MemPercent,
            ProcColumn::State,
        ];
        let new_columns = vec![ProcColumn::Count, ProcColumn::Name, ProcColumn::MemPercent];

        let mut state = init_default_state(&init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        // This should hide the state.
        state.toggle_tab();
        assert_eq!(get_columns(&state.table), new_columns);

        // This should re-reveal the state.
        state.toggle_tab();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    #[test]
    fn toggle_count_pid_2() {
        let init_columns = [
            ProcWidgetColumn::ProcNameOrCommand,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::User,
            ProcWidgetColumn::State,
            ProcWidgetColumn::PidOrCount,
        ];
        let original_columns = vec![
            ProcColumn::Name,
            ProcColumn::MemPercent,
            ProcColumn::User,
            ProcColumn::State,
            ProcColumn::Pid,
        ];
        let new_columns = vec![ProcColumn::Name, ProcColumn::MemPercent, ProcColumn::Count];

        let mut state = init_default_state(&init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        // This should hide the state.
        state.toggle_tab();
        assert_eq!(get_columns(&state.table), new_columns);

        // This should re-reveal the state.
        state.toggle_tab();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    #[test]
    fn toggle_command() {
        let init_columns = [
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::State,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::ProcNameOrCommand,
        ];
        let original_columns = vec![
            ProcColumn::Pid,
            ProcColumn::State,
            ProcColumn::MemPercent,
            ProcColumn::Command,
        ];
        let new_columns = vec![
            ProcColumn::Pid,
            ProcColumn::State,
            ProcColumn::MemPercent,
            ProcColumn::Name,
        ];

        let table_config = ProcTableConfig {
            is_command: true,
            ..Default::default()
        };
        let mut state = init_state(table_config, &init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        state.toggle_command();
        assert_eq!(get_columns(&state.table), new_columns);

        state.toggle_command();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    #[test]
    fn toggle_mem_percentage() {
        let init_columns = [
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
            ProcWidgetColumn::ProcNameOrCommand,
        ];
        let original_columns = vec![
            ProcColumn::Pid,
            ProcColumn::MemPercent,
            ProcColumn::State,
            ProcColumn::Name,
        ];
        let new_columns = vec![
            ProcColumn::Pid,
            ProcColumn::MemValue,
            ProcColumn::State,
            ProcColumn::Name,
        ];

        let mut state = init_default_state(&init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        state.toggle_mem_percentage();
        assert_eq!(get_columns(&state.table), new_columns);

        state.toggle_mem_percentage();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    #[test]
    fn toggle_mem_percentage_2() {
        let init_columns = [
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
            ProcWidgetColumn::ProcNameOrCommand,
        ];
        let original_columns = vec![
            ProcColumn::Pid,
            ProcColumn::MemValue,
            ProcColumn::State,
            ProcColumn::Name,
        ];
        let new_columns = vec![
            ProcColumn::Pid,
            ProcColumn::MemPercent,
            ProcColumn::State,
            ProcColumn::Name,
        ];

        let table_config = ProcTableConfig {
            show_memory_as_values: true,
            ..Default::default()
        };
        let mut state = init_state(table_config, &init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        state.toggle_mem_percentage();
        assert_eq!(get_columns(&state.table), new_columns);

        state.toggle_mem_percentage();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    #[test]
    fn columns_and_is_using_command() {
        let init_columns = [
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
            ProcWidgetColumn::ProcNameOrCommand,
        ];
        let original_columns = vec![
            ProcColumn::Pid,
            ProcColumn::MemPercent,
            ProcColumn::State,
            ProcColumn::Command,
        ];

        let table_config = ProcTableConfig {
            is_command: true,
            ..Default::default()
        };
        let mut state = init_state(table_config, &init_columns);
        assert_eq!(get_columns(&state.table), original_columns);
        assert!(state.is_using_command());

        state.toggle_command();
        assert!(!state.is_using_command());

        state.toggle_command();
        assert!(state.is_using_command());
    }

    #[test]
    fn columns_and_is_memory() {
        let init_columns = [
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
            ProcWidgetColumn::ProcNameOrCommand,
        ];
        let original_columns = vec![
            ProcColumn::Pid,
            ProcColumn::MemValue,
            ProcColumn::State,
            ProcColumn::Name,
        ];

        let table_config = ProcTableConfig {
            show_memory_as_values: true,
            ..Default::default()
        };
        let mut state = init_state(table_config, &init_columns);
        assert_eq!(get_columns(&state.table), original_columns);
        assert!(!state.is_mem_percent());

        state.toggle_mem_percentage();
        assert!(state.is_mem_percent());

        state.toggle_mem_percentage();
        assert!(!state.is_mem_percent());
    }

    /// Tests toggling if both mem and mem% columns are configured.
    ///
    /// Currently, this test doesn't really do much, since we treat these two
    /// columns as the same - this test is intended for use later when we
    /// might allow both at the same time.
    #[test]
    fn double_memory_sim_toggle() {
        let init_columns = [
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::State,
            ProcWidgetColumn::ProcNameOrCommand,
            ProcWidgetColumn::Mem,
        ];
        let original_columns = vec![
            ProcColumn::MemPercent,
            ProcColumn::Pid,
            ProcColumn::State,
            ProcColumn::Name,
        ];
        let new_columns = vec![
            ProcColumn::MemValue,
            ProcColumn::Pid,
            ProcColumn::State,
            ProcColumn::Name,
        ];

        let mut state = init_default_state(&init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        state.toggle_mem_percentage();
        assert_eq!(get_columns(&state.table), new_columns);

        state.toggle_mem_percentage();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    /// Tests toggling if both pid and count columns are configured.
    ///
    /// Currently, this test doesn't really do much, since we treat these two
    /// columns as the same - this test is intended for use later when we
    /// might allow both at the same time.
    #[test]
    fn pid_and_count_sim_toggle() {
        let init_columns = [
            ProcWidgetColumn::ProcNameOrCommand,
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::State,
            ProcWidgetColumn::PidOrCount,
        ];
        let original_columns = vec![
            ProcColumn::Name,
            ProcColumn::Pid,
            ProcColumn::MemPercent,
            ProcColumn::State,
        ];
        let new_columns = vec![ProcColumn::Name, ProcColumn::Count, ProcColumn::MemPercent];

        let mut state = init_default_state(&init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        // This should hide the state.
        state.toggle_tab();
        assert_eq!(get_columns(&state.table), new_columns);

        // This should re-reveal the state.
        state.toggle_tab();
        assert_eq!(get_columns(&state.table), original_columns);
    }

    /// Tests toggling if both command and name columns are configured.
    ///
    /// Currently, this test doesn't really do much, since we treat these two
    /// columns as the same - this test is intended for use later when we
    /// might allow both at the same time.
    #[test]
    fn command_name_sim_toggle() {
        let init_columns = [
            ProcWidgetColumn::ProcNameOrCommand,
            ProcWidgetColumn::PidOrCount,
            ProcWidgetColumn::State,
            ProcWidgetColumn::Mem,
            ProcWidgetColumn::ProcNameOrCommand,
        ];
        let original_columns = vec![
            ProcColumn::Command,
            ProcColumn::Pid,
            ProcColumn::State,
            ProcColumn::MemPercent,
        ];
        let new_columns = vec![
            ProcColumn::Name,
            ProcColumn::Pid,
            ProcColumn::State,
            ProcColumn::MemPercent,
        ];

        let table_config = ProcTableConfig {
            is_command: true,
            ..Default::default()
        };
        let mut state = init_state(table_config, &init_columns);
        assert_eq!(get_columns(&state.table), original_columns);

        state.toggle_command();
        assert_eq!(get_columns(&state.table), new_columns);

        state.toggle_command();
        assert_eq!(get_columns(&state.table), original_columns);
    }
}
