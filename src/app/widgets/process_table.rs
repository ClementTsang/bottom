use crate::{
    app::{
        data_farmer::{DataCollection, ProcessData, StringPidMap},
        data_harvester::processes::ProcessHarvest,
        query::*,
        AppSearchState, ScrollDirection, SortState,
    },
    components::old_text_table::{
        CellContent, SortOrder, SortableState, TableComponentColumn, TableComponentState,
        WidthBounds,
    },
    data_conversion::{binary_byte_string, dec_bytes_per_second_string, TableData, TableRow},
    Pid,
};

use concat_string::concat_string;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use std::cmp::max;

pub mod proc_widget_column;
pub use proc_widget_column::*;

pub mod proc_widget_data;
pub use proc_widget_data::*;

/// ProcessSearchState only deals with process' search's current settings and state.
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

#[derive(Clone, Debug)]
pub enum ProcWidgetMode {
    Tree { collapsed_pids: FxHashSet<Pid> },
    Grouped,
    Normal,
}

pub struct ProcWidget {
    pub mode: ProcWidgetMode,

    pub proc_search: ProcessSearchState,
    // pub table: DataTable<ProcWidgetData, ProcWidgetColumn, Sortable>,
    pub table: TableComponentState<ProcWidgetColumn>,
    pub sort_table_state: TableComponentState,

    pub is_sort_open: bool,
    pub force_rerender: bool,
    pub force_update_data: bool,

    pub table_data: TableData,
}

impl ProcWidget {
    pub const PID_OR_COUNT: usize = 0;
    pub const PROC_NAME_OR_CMD: usize = 1;
    pub const CPU: usize = 2;
    pub const MEM: usize = 3;
    pub const RPS: usize = 4;
    pub const WPS: usize = 5;
    pub const T_READ: usize = 6;
    pub const T_WRITE: usize = 7;
    #[cfg(target_family = "unix")]
    pub const USER: usize = 8;
    #[cfg(target_family = "unix")]
    pub const STATE: usize = 9;
    #[cfg(not(target_family = "unix"))]
    pub const STATE: usize = 8;

    pub fn init(
        mode: ProcWidgetMode, is_case_sensitive: bool, is_match_whole_word: bool,
        is_use_regex: bool, show_memory_as_values: bool, is_command: bool,
    ) -> Self {
        let mut process_search_state = ProcessSearchState::default();

        if is_case_sensitive {
            // By default it's off
            process_search_state.search_toggle_ignore_case();
        }
        if is_match_whole_word {
            process_search_state.search_toggle_whole_word();
        }
        if is_use_regex {
            process_search_state.search_toggle_regex();
        }

        let is_count = matches!(mode, ProcWidgetMode::Grouped);

        let mut sort_table_state = TableComponentState::new(vec![TableComponentColumn::new_hard(
            CellContent::Simple("Sort By".into()),
            7,
        )]);
        sort_table_state.columns[0].calculated_width = 7;

        let table_state = {
            let (default_index, default_order) = if matches!(mode, ProcWidgetMode::Tree { .. }) {
                (Self::PID_OR_COUNT, SortOrder::Ascending)
            } else {
                (Self::CPU, SortOrder::Descending)
            };

            let columns = vec![
                TableComponentColumn::new(ProcWidgetColumn::PidOrCount { is_count }),
                TableComponentColumn::new_soft(
                    ProcWidgetColumn::ProcNameOrCommand { is_command },
                    Some(0.3),
                ),
                TableComponentColumn::new(ProcWidgetColumn::CpuPercent),
                TableComponentColumn::new(ProcWidgetColumn::Memory {
                    show_percentage: !show_memory_as_values,
                }),
                TableComponentColumn::new_hard(ProcWidgetColumn::ReadPerSecond, 8),
                TableComponentColumn::new_hard(ProcWidgetColumn::WritePerSecond, 8),
                TableComponentColumn::new_hard(ProcWidgetColumn::TotalRead, 8),
                TableComponentColumn::new_hard(ProcWidgetColumn::TotalWrite, 8),
                #[cfg(target_family = "unix")]
                TableComponentColumn::new_soft(ProcWidgetColumn::User, Some(0.05)),
                TableComponentColumn::new_hard(ProcWidgetColumn::State, 7),
            ];

            let default_sort_orderings = columns
                .iter()
                .map(|column| column.header.default_sort_order())
                .collect();

            TableComponentState::new(columns).sort_state(SortState::Sortable(SortableState::new(
                default_index,
                default_order,
                default_sort_orderings,
            )))
        };

        ProcWidget {
            proc_search: process_search_state,
            table: table_state,
            sort_table_state,
            is_sort_open: false,
            mode,
            force_rerender: true,
            force_update_data: false,
            table_data: TableData::default(),
        }
    }

    pub fn is_using_command(&self) -> bool {
        if let Some(ProcWidgetColumn::ProcNameOrCommand { is_command }) = self
            .table
            .columns
            .get(ProcWidget::PROC_NAME_OR_CMD)
            .map(|col| &col.header)
        {
            *is_command
        } else {
            // Technically impossible.
            false
        }
    }

    /// This function *only* updates the displayed process data. If there is a need to update the actual *stored* data,
    /// call it before this function.
    pub fn update_displayed_process_data(&mut self, data_collection: &DataCollection) {
        let search_query = if self.proc_search.search_state.is_invalid_or_blank_search() {
            &None
        } else {
            &self.proc_search.search_state.query
        };
        let table_data = match &self.mode {
            ProcWidgetMode::Tree { collapsed_pids } => {
                self.get_tree_table_data(collapsed_pids, data_collection, search_query)
            }
            ProcWidgetMode::Grouped | ProcWidgetMode::Normal => {
                self.get_normal_table_data(data_collection, search_query)
            }
        };

        // Now also update the scroll position if needed (that is, the old scroll position was too big for the new list).
        if self.table.current_scroll_position >= table_data.data.len() {
            self.table.current_scroll_position = table_data.data.len().saturating_sub(1);
            self.table.scroll_bar = 0;
            self.table.scroll_direction = ScrollDirection::Down;
        }

        // Finally, move this data to the widget itself.
        self.table_data = table_data;
    }

    fn get_tree_table_data(
        &self, collapsed_pids: &FxHashSet<Pid>, data_collection: &DataCollection,
        search_query: &Option<Query>,
    ) -> TableData {
        const BRANCH_ENDING: char = '└';
        const BRANCH_VERTICAL: char = '│';
        const BRANCH_SPLIT: char = '├';
        const BRANCH_HORIZONTAL: char = '─';

        let ProcessData {
            process_harvest,
            cmd_pid_map,
            name_pid_map,
            process_parent_mapping,
            orphan_pids,
            ..
        } = &data_collection.process_data;

        let mut col_widths = vec![0; self.table.columns.iter().filter(|c| c.is_skipped()).count()];

        let matching_pids = data_collection
            .process_data
            .process_harvest
            .iter()
            .map(|(pid, process)| {
                (
                    *pid,
                    search_query
                        .as_ref()
                        .map(|q| q.check(process, self.is_using_command()))
                        .unwrap_or(true),
                )
            })
            .collect::<FxHashMap<_, _>>();

        let filtered_tree = {
            let mut filtered_tree = FxHashMap::default();

            let mut stack = orphan_pids
                .iter()
                .filter_map(|process| process_harvest.get(process))
                .collect_vec();
            let mut visited_pids = FxHashMap::default();

            while let Some(process) = stack.last() {
                let is_process_matching = *matching_pids.get(&process.pid).unwrap_or(&false);

                if let Some(children_pids) = process_parent_mapping.get(&process.pid) {
                    if children_pids
                        .iter()
                        .all(|pid| visited_pids.contains_key(pid))
                    {
                        let shown_children = children_pids
                            .iter()
                            .filter(|pid| visited_pids.get(*pid).copied().unwrap_or(false))
                            .collect_vec();
                        let is_shown = is_process_matching || !shown_children.is_empty();
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
                    visited_pids.insert(process.pid, is_process_matching);
                    stack.pop();
                }
            }

            filtered_tree
        };

        let mut resulting_strings = vec![];
        let mut prefixes = vec![];
        let mut stack = orphan_pids
            .iter()
            .filter(|pid| filtered_tree.contains_key(*pid))
            .filter_map(|child| process_harvest.get(child))
            .collect_vec();

        self.try_sort(&mut stack, data_collection);

        let mut length_stack = vec![stack.len()];

        while let (Some(process), Some(siblings_left)) = (stack.pop(), length_stack.last_mut()) {
            *siblings_left -= 1;

            let is_disabled = !*matching_pids.get(&process.pid).unwrap_or(&false);
            let is_last = *siblings_left == 0;

            if collapsed_pids.contains(&process.pid) {
                let mut summed_process = process.clone();

                if let Some(children_pids) = filtered_tree.get(&process.pid) {
                    let mut sum_queue = children_pids
                        .iter()
                        .filter_map(|child| process_harvest.get(child))
                        .collect_vec();

                    while let Some(process) = sum_queue.pop() {
                        summed_process.add(process);

                        if let Some(pids) = filtered_tree.get(&process.pid) {
                            sum_queue.extend(pids.iter().filter_map(|c| process_harvest.get(c)));
                        }
                    }
                }

                let prefix = if prefixes.is_empty() {
                    "+ ".to_string()
                } else {
                    format!(
                        "{}{}{} + ",
                        prefixes.join(""),
                        if is_last { BRANCH_ENDING } else { BRANCH_SPLIT },
                        BRANCH_HORIZONTAL
                    )
                };

                let process_text = self.process_to_text(
                    &summed_process,
                    &mut col_widths,
                    cmd_pid_map,
                    name_pid_map,
                    Some(prefix),
                    is_disabled,
                );
                resulting_strings.push(process_text);
            } else {
                let prefix = if prefixes.is_empty() {
                    String::default()
                } else {
                    format!(
                        "{}{}{} ",
                        prefixes.join(""),
                        if is_last { BRANCH_ENDING } else { BRANCH_SPLIT },
                        BRANCH_HORIZONTAL
                    )
                };
                let process_text = self.process_to_text(
                    process,
                    &mut col_widths,
                    cmd_pid_map,
                    name_pid_map,
                    Some(prefix),
                    is_disabled,
                );
                resulting_strings.push(process_text);

                if let Some(children_pids) = filtered_tree.get(&process.pid) {
                    if prefixes.is_empty() {
                        prefixes.push(String::default());
                    } else {
                        prefixes.push(if is_last {
                            "   ".to_string()
                        } else {
                            format!("{}  ", BRANCH_VERTICAL)
                        });
                    }

                    let mut children = children_pids
                        .iter()
                        .filter_map(|child_pid| process_harvest.get(child_pid))
                        .collect_vec();
                    self.try_sort(&mut children, data_collection);
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

        TableData {
            data: resulting_strings,
            col_widths,
        }
    }

    fn get_normal_table_data(
        &self, data_collection: &DataCollection, search_query: &Option<Query>,
    ) -> TableData {
        let mut id_pid_map: FxHashMap<String, ProcessHarvest>;
        let filtered_iter = data_collection
            .process_data
            .process_harvest
            .values()
            .filter(|p| {
                search_query
                    .as_ref()
                    .map(|q| q.check(p, self.is_using_command()))
                    .unwrap_or(true)
            });

        let mut filtered_data = if let ProcWidgetMode::Grouped = self.mode {
            id_pid_map = FxHashMap::default();
            filtered_iter.for_each(|process| {
                let id = if self.is_using_command() {
                    &process.command
                } else {
                    &process.name
                };

                if let Some(grouped_process_harvest) = id_pid_map.get_mut(id) {
                    grouped_process_harvest.add(process);
                } else {
                    id_pid_map.insert(id.clone(), process.clone());
                }
            });

            id_pid_map.values().collect::<Vec<_>>()
        } else {
            filtered_iter.collect::<Vec<_>>()
        };

        self.try_sort(&mut filtered_data, data_collection);
        self.harvest_to_table_data(&filtered_data, data_collection)
    }

    fn try_sort(&self, filtered_data: &mut [&ProcessHarvest], data_collection: &DataCollection) {
        let cmd_pid_map = &data_collection.process_data.cmd_pid_map;
        let name_pid_map = &data_collection.process_data.name_pid_map;

        if let SortState::Sortable(state) = &self.table.sort_state {
            let index = state.current_index;
            let order = &state.order;

            if let Some(column) = self.table.columns.get(index) {
                column.header.sort(
                    order.is_descending(),
                    filtered_data,
                    self.is_using_command(),
                    cmd_pid_map,
                    name_pid_map,
                );
            }
        }
    }

    fn process_to_text(
        &self, process: &ProcessHarvest, col_widths: &mut [usize], cmd_pid_map: &StringPidMap,
        name_pid_map: &StringPidMap, proc_prefix: Option<String>, is_disabled: bool,
    ) -> TableRow {
        let mut contents = Vec::with_capacity(self.num_shown_columns());

        contents.extend(self.table.columns.iter().enumerate().map(|(itx, column)| {
            let col_text = match column.header {
                ProcWidgetColumn::CpuPercent => format!("{:.1}%", process.cpu_usage_percent).into(),
                ProcWidgetColumn::Memory { show_percentage } => {
                    if show_percentage {
                        format!("{:.1}%", process.mem_usage_percent).into()
                    } else {
                        binary_byte_string(process.mem_usage_bytes).into()
                    }
                }
                ProcWidgetColumn::PidOrCount { is_count } => {
                    if is_count {
                        if self.is_using_command() {
                            cmd_pid_map
                                .get(&process.command)
                                .map(|v| v.len())
                                .unwrap_or(0)
                                .to_string()
                                .into()
                        } else {
                            name_pid_map
                                .get(&process.name)
                                .map(|v| v.len())
                                .unwrap_or(0)
                                .to_string()
                                .into()
                        }
                    } else {
                        process.pid.to_string().into()
                    }
                }
                ProcWidgetColumn::ProcNameOrCommand { is_command } => {
                    let val = if is_command {
                        process.command.clone()
                    } else {
                        process.name.clone()
                    };

                    if let Some(prefix) = &proc_prefix {
                        concat_string!(prefix, val).into()
                    } else {
                        val.into()
                    }
                }
                ProcWidgetColumn::ReadPerSecond => {
                    dec_bytes_per_second_string(process.read_bytes_per_sec).into()
                }
                ProcWidgetColumn::WritePerSecond => {
                    dec_bytes_per_second_string(process.write_bytes_per_sec).into()
                }
                ProcWidgetColumn::TotalRead => {
                    dec_bytes_per_second_string(process.total_read_bytes).into()
                }
                ProcWidgetColumn::TotalWrite => {
                    dec_bytes_per_second_string(process.total_write_bytes).into()
                }
                ProcWidgetColumn::State => CellContent::HasAlt {
                    main: process.process_state.0.clone().into(),
                    alt: process.process_state.1.to_string().into(),
                },
                ProcWidgetColumn::User => {
                    #[cfg(target_family = "unix")]
                    {
                        process.user.clone().into()
                    }
                    #[cfg(not(target_family = "unix"))]
                    {
                        "".into()
                    }
                }
            };

            if let Some(curr) = col_widths.get_mut(itx) {
                *curr = max(*curr, col_text.len());
            }

            col_text
        }));

        if is_disabled {
            TableRow::Styled(contents, tui::style::Style::default())
        } else {
            TableRow::Raw(contents)
        }
    }

    fn harvest_to_table_data(
        &self, process_data: &[&ProcessHarvest], data_collection: &DataCollection,
    ) -> TableData {
        let cmd_pid_map = &data_collection.process_data.cmd_pid_map;
        let name_pid_map = &data_collection.process_data.name_pid_map;

        let mut col_widths = vec![0; self.table.columns.len()];

        let data = process_data
            .iter()
            .map(|process| {
                self.process_to_text(
                    process,
                    &mut col_widths,
                    cmd_pid_map,
                    name_pid_map,
                    None,
                    false,
                )
            })
            .collect();

        TableData { data, col_widths }
    }

    fn get_mut_proc_col(&mut self, index: usize) -> Option<&mut ProcWidgetColumn> {
        self.table.columns.get_mut(index).map(|col| &mut col.header)
    }

    pub fn toggle_mem_percentage(&mut self) {
        if let Some(ProcWidgetColumn::Memory { show_percentage }) = self.get_mut_proc_col(Self::MEM)
        {
            *show_percentage = !*show_percentage;
            self.force_data_update();
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

    /// Marks the selected column as hidden, and automatically resets the selected column if currently selected.
    fn hide_column(&mut self, index: usize) {
        if let Some(col) = self.table.columns.get_mut(index) {
            col.is_hidden = true;

            if let SortState::Sortable(state) = &mut self.table.sort_state {
                if state.current_index == index {
                    state.current_index = Self::CPU;
                    state.order = SortOrder::Descending;
                }
            }
        }
    }

    /// Marks the selected column as shown.
    fn show_column(&mut self, index: usize) {
        if let Some(col) = self.table.columns.get_mut(index) {
            col.is_hidden = false;
        }
    }

    /// Select a column. If the column is already selected, then just toggle the sort order.
    pub fn select_column(&mut self, new_sort_index: usize) {
        if let SortState::Sortable(state) = &mut self.table.sort_state {
            state.update_sort_index(new_sort_index);
            self.force_data_update();
        }
    }

    pub fn toggle_tree_branch(&mut self) {
        if let ProcWidgetMode::Tree { collapsed_pids } = &mut self.mode {
            let current_posn = self.table.current_scroll_position;
            if let Some(current_row) = self.table_data.data.get(current_posn) {
                if let Ok(pid) = current_row.row()[ProcWidget::PID_OR_COUNT]
                    .main_text()
                    .parse::<Pid>()
                {
                    if !collapsed_pids.remove(&pid) {
                        collapsed_pids.insert(pid);
                    }
                    self.force_data_update();
                }
            }
        }
    }

    pub fn toggle_command(&mut self) {
        if let Some(col) = self.table.columns.get_mut(Self::PROC_NAME_OR_CMD) {
            if let ProcWidgetColumn::ProcNameOrCommand { is_command } = &mut col.header {
                *is_command = !*is_command;

                if let WidthBounds::Soft { max_percentage, .. } = &mut col.width_bounds {
                    if *is_command {
                        *max_percentage = Some(0.7);
                    } else {
                        *max_percentage = match self.mode {
                            ProcWidgetMode::Tree { .. } => Some(0.5),
                            ProcWidgetMode::Grouped | ProcWidgetMode::Normal => Some(0.3),
                        };
                    }
                }

                self.force_rerender_and_update();
            }
        }
    }

    /// Toggles the appropriate columns/settings when tab is pressed.
    ///
    /// If count is enabled, we should set the mode to [`ProcWidgetMode::Grouped`], and switch off the User and State
    /// columns. We should also move the user off of the columns if they were selected, as those columns are now hidden
    /// (handled by internal method calls), and go back to the "defaults".
    ///
    /// Otherwise, if count is disabled, then the User and State columns should be re-enabled, and the mode switched
    /// to [`ProcWidgetMode::Normal`].
    pub fn toggle_tab(&mut self) {
        if !matches!(self.mode, ProcWidgetMode::Tree { .. }) {
            if let Some(ProcWidgetColumn::PidOrCount { is_count }) =
                self.get_mut_proc_col(Self::PID_OR_COUNT)
            {
                *is_count = !*is_count;

                if *is_count {
                    #[cfg(target_family = "unix")]
                    self.hide_column(Self::USER);
                    self.hide_column(Self::STATE);
                    self.mode = ProcWidgetMode::Grouped;

                    self.sort_table_state.current_scroll_position = self
                        .sort_table_state
                        .current_scroll_position
                        .clamp(0, self.num_enabled_columns().saturating_sub(1));
                } else {
                    #[cfg(target_family = "unix")]
                    self.show_column(Self::USER);
                    self.show_column(Self::STATE);
                    self.mode = ProcWidgetMode::Normal;
                }
                self.force_rerender_and_update();
            }
        }
    }

    pub fn get_search_cursor_position(&self) -> usize {
        self.proc_search.search_state.grapheme_cursor.cur_cursor()
    }

    pub fn get_char_cursor_position(&self) -> usize {
        self.proc_search.search_state.char_cursor_position
    }

    pub fn is_search_enabled(&self) -> bool {
        self.proc_search.search_state.is_enabled
    }

    pub fn get_current_search_query(&self) -> &String {
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
        self.table.scroll_bar = 0;
        self.table.current_scroll_position = 0;

        self.force_data_update();
    }

    pub fn clear_search(&mut self) {
        self.proc_search.search_state.reset();
        self.force_data_update();
    }

    pub fn search_walk_forward(&mut self, start_position: usize) {
        self.proc_search
            .search_state
            .grapheme_cursor
            .next_boundary(
                &self.proc_search.search_state.current_search_query[start_position..],
                start_position,
            )
            .unwrap();
    }

    pub fn search_walk_back(&mut self, start_position: usize) {
        self.proc_search
            .search_state
            .grapheme_cursor
            .prev_boundary(
                &self.proc_search.search_state.current_search_query[..start_position],
                0,
            )
            .unwrap();
    }

    /// Returns the number of columns *visible*.
    pub fn num_shown_columns(&self) -> usize {
        self.table
            .columns
            .iter()
            .filter(|c| !c.is_skipped())
            .count()
    }

    /// Returns the number of columns *enabled*. Note this differs from *visible* - a column may be enabled but not
    /// visible (e.g. off screen).
    pub fn num_enabled_columns(&self) -> usize {
        self.table.columns.iter().filter(|c| !c.is_hidden).count()
    }

    /// Sets the [`ProcWidget`]'s current sort index to whatever was in the sort table.
    pub(crate) fn use_sort_table_value(&mut self) {
        if let SortState::Sortable(st) = &mut self.table.sort_state {
            st.update_sort_index(self.sort_table_state.current_scroll_position);

            self.is_sort_open = false;
            self.force_rerender_and_update();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sort() {}

    #[test]
    fn assert_correct_columns() {
        #[track_caller]
        fn test_columns(mode: ProcWidgetMode, mem_as_val: bool, is_cmd: bool) {
            let is_count = matches!(mode, ProcWidgetMode::Grouped);
            let is_command = is_cmd;
            let show_percentage = !mem_as_val;

            let proc = ProcWidget::init(mode, false, false, false, mem_as_val, is_command);
            let columns = &proc.table.columns;

            assert_eq!(
                columns[ProcWidget::PID_OR_COUNT].header,
                ProcWidgetColumn::PidOrCount { is_count }
            );
            assert_eq!(
                columns[ProcWidget::PROC_NAME_OR_CMD].header,
                ProcWidgetColumn::ProcNameOrCommand { is_command }
            );
            assert!(matches!(
                columns[ProcWidget::CPU].header,
                ProcWidgetColumn::CpuPercent
            ));
            assert_eq!(
                columns[ProcWidget::MEM].header,
                ProcWidgetColumn::Memory { show_percentage }
            );
            assert!(matches!(
                columns[ProcWidget::RPS].header,
                ProcWidgetColumn::ReadPerSecond
            ));
            assert!(matches!(
                columns[ProcWidget::WPS].header,
                ProcWidgetColumn::WritePerSecond
            ));
            assert!(matches!(
                columns[ProcWidget::T_READ].header,
                ProcWidgetColumn::TotalRead
            ));
            assert!(matches!(
                columns[ProcWidget::T_WRITE].header,
                ProcWidgetColumn::TotalWrite
            ));
            #[cfg(target_family = "unix")]
            {
                assert!(matches!(
                    columns[ProcWidget::USER].header,
                    ProcWidgetColumn::User
                ));
            }
            assert!(matches!(
                columns[ProcWidget::STATE].header,
                ProcWidgetColumn::State
            ));
        }

        test_columns(ProcWidgetMode::Grouped, true, true);
        test_columns(ProcWidgetMode::Grouped, false, true);
        test_columns(ProcWidgetMode::Grouped, true, false);
        test_columns(
            ProcWidgetMode::Tree {
                collapsed_pids: Default::default(),
            },
            true,
            true,
        );
        test_columns(ProcWidgetMode::Normal, true, true);
    }
}
