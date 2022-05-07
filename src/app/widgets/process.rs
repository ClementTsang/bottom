use std::borrow::Cow;

use crate::{
    app::{
        query::*, AppSearchState, CanvasTableWidthState, CellContent, TableComponentColumn,
        TableComponentHeader, TableComponentState, WidthBounds,
    },
    data_harvester::processes,
};

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

#[derive(Copy, Clone, Debug)]
pub enum ProcWidgetMode {
    Tree,
    Grouped,
    Normal,
}

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

    fn text(&self) -> &CellContent {
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

pub struct ProcWidget {
    pub mode: ProcWidgetMode,

    pub requires_redraw: bool,

    pub search_state: ProcessSearchState,
    pub table_state: TableComponentState<ProcWidgetColumn>,
    pub sort_table_state: TableComponentState<CellContent>,

    pub is_sort_open: bool,
    pub force_update: bool,

    // Hmm...
    pub is_process_sort_descending: bool,
    pub process_sorting_type: processes::ProcessSorting,

    // TO REMOVE
    pub is_using_command: bool,
    pub table_width_state: CanvasTableWidthState,
}

impl ProcWidget {
    pub fn init(
        mode: ProcWidgetMode, is_case_sensitive: bool, is_match_whole_word: bool,
        is_use_regex: bool, show_memory_as_values: bool, is_using_command: bool,
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

        let (process_sorting_type, is_process_sort_descending) =
            if matches!(mode, ProcWidgetMode::Tree) {
                (processes::ProcessSorting::Pid, false)
            } else {
                (processes::ProcessSorting::CpuPercent, true)
            };

        let is_count = matches!(mode, ProcWidgetMode::Grouped);

        let sort_table_state = TableComponentState::new(vec![TableComponentColumn::new(
            CellContent::Simple("Sort By".into()),
            WidthBounds::Hard(7),
        )]);
        let table_state = TableComponentState::new(vec![
            TableComponentColumn::default_hard(ProcWidgetColumn::PidOrCount { is_count }),
            TableComponentColumn::default_soft(
                ProcWidgetColumn::ProcNameOrCommand {
                    is_command: is_using_command,
                },
                Some(0.7),
            ),
            TableComponentColumn::default_hard(ProcWidgetColumn::CpuPercent),
            TableComponentColumn::default_hard(ProcWidgetColumn::Memory {
                show_percentage: !show_memory_as_values,
            }),
            TableComponentColumn::default_hard(ProcWidgetColumn::ReadPerSecond),
            TableComponentColumn::default_hard(ProcWidgetColumn::WritePerSecond),
            TableComponentColumn::default_hard(ProcWidgetColumn::TotalRead),
            TableComponentColumn::default_hard(ProcWidgetColumn::TotalWrite),
            TableComponentColumn::default_hard(ProcWidgetColumn::User),
            TableComponentColumn::default_hard(ProcWidgetColumn::State),
        ]);

        ProcWidget {
            search_state: process_search_state,
            table_state,
            sort_table_state,
            process_sorting_type,
            is_process_sort_descending,
            is_using_command,
            is_sort_open: false,
            table_width_state: CanvasTableWidthState::default(),
            requires_redraw: false,
            mode,
            force_update: false,
        }
    }

    pub fn get_search_cursor_position(&self) -> usize {
        self.search_state.search_state.grapheme_cursor.cur_cursor()
    }

    pub fn get_char_cursor_position(&self) -> usize {
        self.search_state.search_state.char_cursor_position
    }

    pub fn is_search_enabled(&self) -> bool {
        self.search_state.search_state.is_enabled
    }

    pub fn get_current_search_query(&self) -> &String {
        &self.search_state.search_state.current_search_query
    }

    pub fn update_query(&mut self) {
        if self
            .search_state
            .search_state
            .current_search_query
            .is_empty()
        {
            self.search_state.search_state.is_blank_search = true;
            self.search_state.search_state.is_invalid_search = false;
            self.search_state.search_state.error_message = None;
        } else {
            let parsed_query = self.parse_query();
            // debug!("Parsed query: {:#?}", parsed_query);

            if let Ok(parsed_query) = parsed_query {
                self.search_state.search_state.query = Some(parsed_query);
                self.search_state.search_state.is_blank_search = false;
                self.search_state.search_state.is_invalid_search = false;
                self.search_state.search_state.error_message = None;
            } else if let Err(err) = parsed_query {
                self.search_state.search_state.is_blank_search = false;
                self.search_state.search_state.is_invalid_search = true;
                self.search_state.search_state.error_message = Some(err.to_string());
            }
        }
        self.table_state.scroll_bar = 0;
        self.table_state.current_scroll_position = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_state.search_state.reset();
    }

    pub fn search_walk_forward(&mut self, start_position: usize) {
        self.search_state
            .search_state
            .grapheme_cursor
            .next_boundary(
                &self.search_state.search_state.current_search_query[start_position..],
                start_position,
            )
            .unwrap();
    }

    pub fn search_walk_back(&mut self, start_position: usize) {
        self.search_state
            .search_state
            .grapheme_cursor
            .prev_boundary(
                &self.search_state.search_state.current_search_query[..start_position],
                0,
            )
            .unwrap();
    }

    pub fn num_enabled_columns(&self) -> usize {
        self.table_state
            .columns
            .iter()
            .filter(|c| !c.is_skipped())
            .count()
    }
}
