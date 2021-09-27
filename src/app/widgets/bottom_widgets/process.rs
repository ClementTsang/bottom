use std::{borrow::Cow, collections::HashMap};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use float_ord::FloatOrd;
use itertools::{Either, Itertools};
use once_cell::unsync::Lazy;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Borders, Paragraph},
    Frame,
};

use crate::{
    app::{
        data_harvester::processes::ProcessHarvest,
        event::{ComponentEventResult, MultiKey, MultiKeyResult, ReturnSignal, SelectionAction},
        query::*,
        text_table::DesiredColumnWidth,
        widgets::tui_stuff::BlockBuilder,
        AppConfigFields, DataCollection,
    },
    canvas::Painter,
    data_conversion::get_string_with_bytes,
    options::{layout_options::LayoutRule, ProcessDefaults},
    utils::error::BottomError,
};

use crate::app::{
    does_bound_intersect_coordinate,
    sort_text_table::{SimpleSortableColumn, SortStatus, SortableColumn},
    text_table::TextTableData,
    Component, SortMenu, SortableTextTable, TextInput, Widget,
};

/// The currently selected part of a [`ProcessManager`]
#[derive(PartialEq, Eq, Clone, Copy)]
enum ProcessManagerSelection {
    Processes,
    Sort,
    Search,
}

#[derive(Default)]
/// The state of the search modifiers.
struct SearchModifiers {
    enable_case_sensitive: bool,
    enable_whole_word: bool,
    enable_regex: bool,
}

impl SearchModifiers {
    fn toggle_case_sensitive(&mut self) {
        self.enable_case_sensitive = !self.enable_case_sensitive;
    }

    fn toggle_whole_word(&mut self) {
        self.enable_whole_word = !self.enable_whole_word;
    }

    fn toggle_regex(&mut self) {
        self.enable_regex = !self.enable_regex;
    }
}

enum FlexColumn {
    Flex(f64),
    Hard(Option<u16>),
}

pub enum ProcessSortType {
    Pid,
    Count,
    Name,
    Command,
    Cpu,
    Mem,
    MemPercent,
    Rps,
    Wps,
    TotalRead,
    TotalWrite,
    User,
    State,
}

impl ProcessSortType {
    fn to_str(&self) -> &'static str {
        match self {
            ProcessSortType::Pid => "PID",
            ProcessSortType::Count => "Count",
            ProcessSortType::Name => "Name",
            ProcessSortType::Command => "Command",
            ProcessSortType::Cpu => "CPU%",
            ProcessSortType::Mem => "Mem",
            ProcessSortType::MemPercent => "Mem%",
            ProcessSortType::Rps => "R/s",
            ProcessSortType::Wps => "W/s",
            ProcessSortType::TotalRead => "T.Read",
            ProcessSortType::TotalWrite => "T.Write",
            ProcessSortType::User => "User",
            ProcessSortType::State => "State",
        }
    }

    fn shortcut(&self) -> Option<KeyEvent> {
        match self {
            ProcessSortType::Pid => Some(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)),
            ProcessSortType::Count => None,
            ProcessSortType::Name => Some(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
            ProcessSortType::Command => None,
            ProcessSortType::Cpu => Some(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            ProcessSortType::Mem => Some(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)),
            ProcessSortType::MemPercent => {
                Some(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE))
            }
            ProcessSortType::Rps => None,
            ProcessSortType::Wps => None,
            ProcessSortType::TotalRead => None,
            ProcessSortType::TotalWrite => None,
            ProcessSortType::User => None,
            ProcessSortType::State => None,
        }
    }

    fn column_type(&self) -> FlexColumn {
        use FlexColumn::*;

        match self {
            ProcessSortType::Pid => Hard(Some(7)),
            ProcessSortType::Count => Hard(Some(8)),
            ProcessSortType::Name => Flex(0.3),
            ProcessSortType::Command => Flex(0.7),
            ProcessSortType::Cpu => Hard(Some(8)),
            ProcessSortType::Mem => Hard(Some(8)),
            ProcessSortType::MemPercent => Hard(Some(8)),
            ProcessSortType::Rps => Hard(Some(8)),
            ProcessSortType::Wps => Hard(Some(8)),
            ProcessSortType::TotalRead => Hard(Some(7)),
            ProcessSortType::TotalWrite => Hard(Some(8)),
            ProcessSortType::User => Flex(0.08),
            ProcessSortType::State => Hard(Some(8)),
        }
    }

    fn default_descending(&self) -> bool {
        match self {
            ProcessSortType::Pid => false,
            ProcessSortType::Count => true,
            ProcessSortType::Name => false,
            ProcessSortType::Command => false,
            ProcessSortType::Cpu => true,
            ProcessSortType::Mem => true,
            ProcessSortType::MemPercent => true,
            ProcessSortType::Rps => true,
            ProcessSortType::Wps => true,
            ProcessSortType::TotalRead => true,
            ProcessSortType::TotalWrite => true,
            ProcessSortType::User => false,
            ProcessSortType::State => false,
        }
    }
}

/// A thin wrapper around a [`SortableColumn`] to help keep track of
/// how to sort given a chosen column.
pub struct ProcessSortColumn {
    /// The underlying column.
    sortable_column: SimpleSortableColumn,

    /// The *type* of column. Useful for determining how to sort.
    sort_type: ProcessSortType,
}

impl ProcessSortColumn {
    pub fn new(sort_type: ProcessSortType) -> Self {
        let sortable_column = {
            let name = sort_type.to_str().into();
            let shortcut = sort_type.shortcut();
            let default_descending = sort_type.default_descending();

            match sort_type.column_type() {
                FlexColumn::Flex(max_percentage) => SimpleSortableColumn::new_flex(
                    name,
                    shortcut,
                    default_descending,
                    max_percentage,
                ),
                FlexColumn::Hard(hard_length) => {
                    SimpleSortableColumn::new_hard(name, shortcut, default_descending, hard_length)
                }
            }
        };

        Self {
            sortable_column,
            sort_type,
        }
    }
}

impl SortableColumn for ProcessSortColumn {
    fn original_name(&self) -> &Cow<'static, str> {
        self.sortable_column.original_name()
    }

    fn shortcut(&self) -> &Option<(KeyEvent, String)> {
        self.sortable_column.shortcut()
    }

    fn default_descending(&self) -> bool {
        self.sortable_column.default_descending()
    }

    fn sorting_status(&self) -> SortStatus {
        self.sortable_column.sorting_status()
    }

    fn set_sorting_status(&mut self, sorting_status: SortStatus) {
        self.sortable_column.set_sorting_status(sorting_status)
    }

    fn display_name(&self) -> Cow<'static, str> {
        self.sortable_column.display_name()
    }

    fn get_desired_width(&self) -> &DesiredColumnWidth {
        self.sortable_column.get_desired_width()
    }

    fn get_x_bounds(&self) -> Option<(u16, u16)> {
        self.sortable_column.get_x_bounds()
    }

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>) {
        self.sortable_column.set_x_bounds(x_bounds)
    }
}

/// A searchable, sortable table to manage processes.
pub struct ProcessManager {
    bounds: Rect,
    process_table: SortableTextTable<ProcessSortColumn>,
    sort_menu: SortMenu,
    search_block_bounds: Rect,

    search_input: TextInput,

    dd_multi: MultiKey,

    selected: ProcessManagerSelection,
    prev_selected: ProcessManagerSelection,

    in_tree_mode: bool,
    show_sort: bool,
    show_search: bool,

    search_modifiers: SearchModifiers,

    display_data: TextTableData,

    process_filter: Option<Result<Query, BottomError>>,

    block_border: Borders,

    width: LayoutRule,
    height: LayoutRule,

    show_scroll_index: bool,
}

impl ProcessManager {
    /// Creates a new [`ProcessManager`].
    pub fn new(process_defaults: &ProcessDefaults, config: &AppConfigFields) -> Self {
        let process_table_columns = vec![
            ProcessSortColumn::new(ProcessSortType::Pid),
            ProcessSortColumn::new(ProcessSortType::Name),
            ProcessSortColumn::new(ProcessSortType::Cpu),
            ProcessSortColumn::new(ProcessSortType::MemPercent),
            ProcessSortColumn::new(ProcessSortType::Rps),
            ProcessSortColumn::new(ProcessSortType::Wps),
            ProcessSortColumn::new(ProcessSortType::TotalRead),
            ProcessSortColumn::new(ProcessSortType::TotalWrite),
            #[cfg(target_family = "unix")]
            ProcessSortColumn::new(ProcessSortType::User),
            ProcessSortColumn::new(ProcessSortType::State),
        ];

        let mut manager = Self {
            bounds: Rect::default(),
            sort_menu: SortMenu::new(process_table_columns.len()).try_show_gap(config.table_gap),
            process_table: SortableTextTable::new(process_table_columns)
                .default_sort_index(2)
                .try_show_gap(config.table_gap),
            search_input: TextInput::default(),
            search_block_bounds: Rect::default(),
            dd_multi: MultiKey::register(vec!['d', 'd']), // TODO: [Optimization] Maybe use something static/const/arrayvec?...
            selected: ProcessManagerSelection::Processes,
            prev_selected: ProcessManagerSelection::Processes,
            in_tree_mode: false,
            show_sort: false,
            show_search: false,
            search_modifiers: SearchModifiers::default(),
            display_data: Default::default(),
            process_filter: None,
            block_border: Borders::ALL,
            width: LayoutRule::default(),
            height: LayoutRule::default(),
            show_scroll_index: false,
        };

        manager.set_tree_mode(process_defaults.is_tree);
        manager
    }

    /// Sets the block border style.
    pub fn basic_mode(mut self, basic_mode: bool) -> Self {
        if basic_mode {
            self.block_border = *crate::constants::SIDE_BORDERS;
        }

        self
    }

    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: LayoutRule) -> Self {
        self.height = height;
        self
    }

    fn set_tree_mode(&mut self, in_tree_mode: bool) {
        self.in_tree_mode = in_tree_mode;
    }

    /// Sets whether to show the scroll index.
    pub fn show_scroll_index(mut self, show_scroll_index: bool) -> Self {
        self.show_scroll_index = show_scroll_index;
        self
    }

    fn open_search(&mut self) -> ComponentEventResult {
        if let ProcessManagerSelection::Search = self.selected {
            ComponentEventResult::NoRedraw
        } else {
            self.show_search = true;
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Search;
            ComponentEventResult::Redraw
        }
    }

    fn open_sort(&mut self) -> ComponentEventResult {
        if let ProcessManagerSelection::Sort = self.selected {
            ComponentEventResult::NoRedraw
        } else {
            self.sort_menu
                .set_index(self.process_table.current_sorting_column_index());
            self.show_sort = true;
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Sort;
            ComponentEventResult::Redraw
        }
    }

    /// Returns whether the process manager is searching the current term with the restriction that it must
    /// match entire word.
    pub fn is_searching_whole_word(&self) -> bool {
        self.search_modifiers.enable_whole_word
    }

    /// Returns whether the process manager is searching the current term using regex.
    pub fn is_searching_with_regex(&self) -> bool {
        self.search_modifiers.enable_regex
    }

    /// Returns whether the process manager is searching the current term with the restriction that case-sensitivity
    /// matters.
    pub fn is_case_sensitive(&self) -> bool {
        self.search_modifiers.enable_case_sensitive
    }

    fn is_using_command(&self) -> bool {
        matches!(
            self.process_table.columns()[1].sort_type,
            ProcessSortType::Command
        )
    }

    fn toggle_command(&mut self) -> ComponentEventResult {
        if self.is_using_command() {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Name), 1);
        } else {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Command), 1);
        }

        // Invalidate row cache.
        self.process_table.invalidate_cached_columns();

        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    fn is_grouped(&self) -> bool {
        matches!(
            self.process_table.columns()[0].sort_type,
            ProcessSortType::Count
        )
    }

    fn toggle_grouped(&mut self) -> ComponentEventResult {
        if self.is_grouped() {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Pid), 0);

            self.process_table
                .add_column(ProcessSortColumn::new(ProcessSortType::State), 8);
            #[cfg(target_family = "unix")]
            {
                self.process_table
                    .add_column(ProcessSortColumn::new(ProcessSortType::User), 8);
            }
        } else {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Count), 0);

            #[cfg(target_family = "unix")]
            {
                self.process_table.remove_column(9, Some(2));
            }
            self.process_table.remove_column(8, Some(2));
        }

        // Invalidate row cache.
        self.process_table.invalidate_cached_columns();

        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    fn toggle_memory(&mut self) -> ComponentEventResult {
        if matches!(
            self.process_table.columns()[3].sort_type,
            ProcessSortType::MemPercent
        ) {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::Mem), 3);
        } else {
            self.process_table
                .set_column(ProcessSortColumn::new(ProcessSortType::MemPercent), 3);
        }

        // Invalidate row cache.
        self.process_table.invalidate_cached_columns(); // TODO: [Gotcha, Refactor] This should be automatically called somehow after sets/removes to avoid forgetting it - maybe do a queue system?

        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    fn hide_sort(&mut self) {
        self.show_sort = false;
        if let ProcessManagerSelection::Sort = self.selected {
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Processes;
        }
    }

    fn hide_search(&mut self) {
        self.show_search = false;
        if let ProcessManagerSelection::Search = self.selected {
            self.prev_selected = self.selected;
            self.selected = ProcessManagerSelection::Processes;
        }
    }

    /// Toggles the search case-sensitivity status for the [`ProcessManager`].
    fn toggle_search_case_sensitive(&mut self) -> ComponentEventResult {
        self.search_modifiers.toggle_case_sensitive();
        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    /// Toggle whether to search for the whole word for the [`ProcessManager`].
    fn toggle_search_whole_word(&mut self) -> ComponentEventResult {
        self.search_modifiers.toggle_whole_word();
        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    /// Toggle whether to search with regex for the [`ProcessManager`].
    fn toggle_search_regex(&mut self) -> ComponentEventResult {
        self.search_modifiers.toggle_regex();
        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    /// Toggles tree mode.
    fn toggle_tree_mode(&mut self) -> ComponentEventResult {
        self.in_tree_mode = !self.in_tree_mode;
        ComponentEventResult::Signal(ReturnSignal::Update)
    }
}

impl Component for ProcessManager {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        // "Global" handling:

        if let KeyCode::Esc = event.code {
            match self.selected {
                ProcessManagerSelection::Processes => {
                    if self.show_sort {
                        self.hide_sort();
                        return ComponentEventResult::Redraw;
                    } else if self.show_search {
                        self.hide_search();
                        return ComponentEventResult::Redraw;
                    }
                }
                ProcessManagerSelection::Sort if self.show_sort => {
                    self.hide_sort();
                    return ComponentEventResult::Redraw;
                }
                ProcessManagerSelection::Search if self.show_search => {
                    self.hide_search();
                    return ComponentEventResult::Redraw;
                }
                _ => {}
            }
        }

        match self.selected {
            ProcessManagerSelection::Processes => {
                // Try to catch some stuff first...
                if event.modifiers.is_empty() {
                    match event.code {
                        KeyCode::Tab => {
                            // Handle grouping/ungrouping
                            return self.toggle_grouped();
                        }
                        KeyCode::Char('P') => {
                            // Show full command/process name
                            return self.toggle_command();
                        }
                        KeyCode::Char('d') => {
                            match self.dd_multi.input('d') {
                                MultiKeyResult::Completed => {
                                    // Kill the selected process(es)
                                }
                                MultiKeyResult::Accepted | MultiKeyResult::Rejected => {
                                    return ComponentEventResult::NoRedraw;
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            return self.open_search();
                        }
                        KeyCode::Char('%') => {
                            return self.toggle_memory();
                        }
                        KeyCode::Char('+') => {
                            // Expand a branch
                        }
                        KeyCode::Char('-') => {
                            // Collapse a branch
                        }
                        KeyCode::Char('t') | KeyCode::F(5) => {
                            return self.toggle_tree_mode();
                        }
                        KeyCode::Char('s') | KeyCode::F(6) => {
                            return self.open_sort();
                        }
                        KeyCode::F(9) => {
                            // Kill the selected process(es)
                        }
                        _ => {}
                    }
                } else if let KeyModifiers::CONTROL = event.modifiers {
                    if let KeyCode::Char('f') = event.code {
                        return self.open_search();
                    }
                } else if let KeyModifiers::SHIFT = event.modifiers {
                    if let KeyCode::Char('P') = event.code {
                        // Show full command/process name
                        return self.toggle_command();
                    }
                }

                self.process_table.handle_key_event(event)
            }
            ProcessManagerSelection::Sort => {
                if event.modifiers.is_empty() {
                    match event.code {
                        KeyCode::Enter => {
                            self.process_table
                                .set_sort_index(self.sort_menu.current_index());
                            return ComponentEventResult::Signal(ReturnSignal::Update);
                        }
                        KeyCode::Char('/') => {
                            return self.open_search();
                        }
                        _ => {}
                    }
                }

                self.sort_menu.handle_key_event(event)
            }
            ProcessManagerSelection::Search => {
                if event.modifiers.is_empty() {
                    match event.code {
                        KeyCode::F(1) => {
                            return self.toggle_search_case_sensitive();
                        }
                        KeyCode::F(2) => {
                            return self.toggle_search_whole_word();
                        }
                        KeyCode::F(3) => {
                            return self.toggle_search_regex();
                        }
                        _ => {}
                    }
                } else if let KeyModifiers::ALT = event.modifiers {
                    match event.code {
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            return self.toggle_search_case_sensitive();
                        }
                        KeyCode::Char('w') | KeyCode::Char('W') => {
                            return self.toggle_search_whole_word();
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            return self.toggle_search_regex();
                        }
                        _ => {}
                    }
                }

                let handle_output = self.search_input.handle_key_event(event);
                if let ComponentEventResult::Signal(ReturnSignal::Update) = handle_output {
                    if !self.search_input.query().is_empty() {
                        self.process_filter = Some(parse_query(
                            self.search_input.query(),
                            self.is_searching_whole_word(),
                            !self.is_case_sensitive(),
                            self.is_searching_with_regex(),
                        ));
                    } else {
                        self.process_filter = None;
                    }
                }

                handle_output
            }
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        match &event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.process_table.does_border_intersect_mouse(&event) {
                    if let ProcessManagerSelection::Processes = self.selected {
                        self.process_table.handle_mouse_event(event)
                    } else {
                        self.prev_selected = self.selected;
                        self.selected = ProcessManagerSelection::Processes;
                        match self.process_table.handle_mouse_event(event) {
                            ComponentEventResult::Unhandled
                            | ComponentEventResult::Redraw
                            | ComponentEventResult::NoRedraw => ComponentEventResult::Redraw,
                            ComponentEventResult::Signal(s) => ComponentEventResult::Signal(s),
                        }
                    }
                } else if self.sort_menu.does_border_intersect_mouse(&event) {
                    if let ProcessManagerSelection::Sort = self.selected {
                        self.sort_menu.handle_mouse_event(event)
                    } else {
                        self.prev_selected = self.selected;
                        self.selected = ProcessManagerSelection::Sort;
                        self.sort_menu.handle_mouse_event(event);
                        ComponentEventResult::Redraw
                    }
                } else if does_bound_intersect_coordinate(
                    event.column,
                    event.row,
                    self.search_block_bounds,
                ) {
                    if let ProcessManagerSelection::Search = self.selected {
                        self.search_input.handle_mouse_event(event)
                    } else {
                        self.prev_selected = self.selected;
                        self.selected = ProcessManagerSelection::Search;
                        self.search_input.handle_mouse_event(event);
                        ComponentEventResult::Redraw
                    }
                } else {
                    ComponentEventResult::Unhandled
                }
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => match self.selected {
                ProcessManagerSelection::Processes => self.process_table.handle_mouse_event(event),
                ProcessManagerSelection::Sort => self.sort_menu.handle_mouse_event(event),
                ProcessManagerSelection::Search => self.search_input.handle_mouse_event(event),
            },
            _ => ComponentEventResult::Unhandled,
        }
    }
}

impl Widget for ProcessManager {
    fn get_pretty_name(&self) -> &'static str {
        "Processes"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
        expanded: bool,
    ) {
        let draw_area = if self.show_search {
            let search_constraints: [Constraint; 2] = [
                Constraint::Min(0),
                if self.block_border.contains(Borders::TOP) {
                    Constraint::Length(5)
                } else {
                    Constraint::Length(3)
                },
            ];
            const INTERNAL_SEARCH_CONSTRAINTS: [Constraint; 3] = [Constraint::Length(1); 3];

            let vertical_split_area = Layout::default()
                .margin(0)
                .direction(Direction::Vertical)
                .constraints(search_constraints)
                .split(area);

            let is_search_selected =
                selected && matches!(self.selected, ProcessManagerSelection::Search);

            // TODO: [Redesign] this currently uses a separate box - maybe fold this into the main box?
            let search_block = BlockBuilder::new("")
                .selected(is_search_selected)
                .hide_title(true)
                .build(painter, vertical_split_area[1]);

            self.search_block_bounds = vertical_split_area[1];

            let internal_split_area = Layout::default()
                .margin(0)
                .direction(Direction::Vertical)
                .constraints(INTERNAL_SEARCH_CONSTRAINTS)
                .split(search_block.inner(vertical_split_area[1]));

            if internal_split_area[0].height > 0 {
                self.search_input.draw_text_input(
                    painter,
                    f,
                    internal_split_area[0],
                    is_search_selected,
                );
            }

            if internal_split_area[1].height > 0 {
                if let Some(Err(err)) = &self.process_filter {
                    f.render_widget(
                        Paragraph::new(tui::text::Span::styled(
                            err.to_string(),
                            painter.colours.invalid_query_style,
                        )),
                        internal_split_area[1],
                    );
                }
            }

            if internal_split_area[2].height > 0 {
                let case_text: Lazy<String> = Lazy::new(|| {
                    format!(
                        "Case({})",
                        if cfg!(target_os = "macos") {
                            "F1"
                        } else {
                            "Alt+C"
                        }
                    )
                });

                let whole_word_text: Lazy<String> = Lazy::new(|| {
                    format!(
                        "Whole({})",
                        if cfg!(target_os = "macos") {
                            "F2"
                        } else {
                            "Alt+W"
                        }
                    )
                });

                let regex_text: Lazy<String> = Lazy::new(|| {
                    format!(
                        "Regex({})",
                        if cfg!(target_os = "macos") {
                            "F3"
                        } else {
                            "Alt+R"
                        }
                    )
                });

                let case_style = if self.is_case_sensitive() {
                    painter.colours.currently_selected_text_style
                } else {
                    painter.colours.text_style
                };

                let whole_word_style = if self.is_searching_whole_word() {
                    painter.colours.currently_selected_text_style
                } else {
                    painter.colours.text_style
                };

                let regex_style = if self.is_searching_with_regex() {
                    painter.colours.currently_selected_text_style
                } else {
                    painter.colours.text_style
                };

                f.render_widget(
                    Paragraph::new(Spans::from(vec![
                        Span::styled(&*case_text, case_style),
                        Span::raw("  "), // TODO: [Drawing] Smartly space it out in the future...
                        Span::styled(&*whole_word_text, whole_word_style),
                        Span::raw("  "),
                        Span::styled(&*regex_text, regex_style),
                    ])),
                    internal_split_area[2],
                )
            }

            f.render_widget(search_block, vertical_split_area[1]);

            vertical_split_area[0]
        } else {
            area
        };

        let draw_area = if self.show_sort {
            const SORT_CONSTRAINTS: [Constraint; 2] = [Constraint::Length(10), Constraint::Min(0)];

            let horizontal_split_area = Layout::default()
                .margin(0)
                .direction(Direction::Horizontal)
                .constraints(SORT_CONSTRAINTS)
                .split(draw_area);

            let sort_block = self
                .block()
                .selected(selected && matches!(self.selected, ProcessManagerSelection::Sort))
                .hide_title(true);
            self.sort_menu.draw_sort_menu(
                painter,
                f,
                self.process_table.columns(),
                sort_block,
                horizontal_split_area[0],
            );

            horizontal_split_area[1]
        } else {
            draw_area
        };

        let process_selected =
            selected && matches!(self.selected, ProcessManagerSelection::Processes);
        let process_block = self
            .block()
            .selected(process_selected)
            .borders(self.block_border)
            .show_esc(expanded && !self.show_sort && !self.show_search);

        self.process_table.draw_tui_table(
            painter,
            f,
            &self.display_data,
            process_block,
            draw_area,
            process_selected,
            self.show_scroll_index,
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        let mut id_pid_map: HashMap<String, ProcessHarvest>;

        let filtered_iter = data_collection.process_harvest.iter().filter(|process| {
            if let Some(Ok(query)) = &self.process_filter {
                query.check(process, self.is_using_command())
            } else {
                true
            }
        });

        let filtered_grouped_iter = if self.is_grouped() {
            id_pid_map = HashMap::new();
            filtered_iter.for_each(|process_harvest| {
                let id = if self.is_using_command() {
                    &process_harvest.command
                } else {
                    &process_harvest.name
                };

                if let Some(grouped_process_harvest) = id_pid_map.get_mut(id) {
                    grouped_process_harvest.cpu_usage_percent += process_harvest.cpu_usage_percent;
                    grouped_process_harvest.mem_usage_bytes += process_harvest.mem_usage_bytes;
                    grouped_process_harvest.mem_usage_percent += process_harvest.mem_usage_percent;
                    grouped_process_harvest.read_bytes_per_sec +=
                        process_harvest.read_bytes_per_sec;
                    grouped_process_harvest.write_bytes_per_sec +=
                        process_harvest.write_bytes_per_sec;
                    grouped_process_harvest.total_read_bytes += process_harvest.total_read_bytes;
                    grouped_process_harvest.total_write_bytes += process_harvest.total_write_bytes;
                } else {
                    id_pid_map.insert(id.clone(), process_harvest.clone());
                }
            });

            Either::Left(id_pid_map.values())
        } else {
            Either::Right(filtered_iter)
        };

        let filtered_sorted_iter = if let ProcessSortType::Count =
            self.process_table.current_sorting_column().sort_type
        {
            let mut v = filtered_grouped_iter.collect::<Vec<_>>();
            v.sort_by_cached_key(|k| {
                if self.is_using_command() {
                    data_collection
                        .process_cmd_pid_map
                        .get(&k.command)
                        .map(|v| v.len())
                        .unwrap_or(0)
                } else {
                    data_collection
                        .process_name_pid_map
                        .get(&k.name)
                        .map(|v| v.len())
                        .unwrap_or(0)
                }
            });
            Either::Left(v.into_iter())
        } else {
            Either::Right(filtered_grouped_iter.sorted_by(
                match self.process_table.current_sorting_column().sort_type {
                    ProcessSortType::Pid => {
                        |a: &&ProcessHarvest, b: &&ProcessHarvest| a.pid.cmp(&b.pid)
                    }
                    ProcessSortType::Count => {
                        // This case should be impossible by the above check.
                        unreachable!()
                    }
                    ProcessSortType::Name => {
                        |a: &&ProcessHarvest, b: &&ProcessHarvest| a.name.cmp(&b.name)
                    }
                    ProcessSortType::Command => {
                        |a: &&ProcessHarvest, b: &&ProcessHarvest| a.command.cmp(&b.command)
                    }
                    ProcessSortType::Cpu => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        FloatOrd(a.cpu_usage_percent).cmp(&FloatOrd(b.cpu_usage_percent))
                    },
                    ProcessSortType::Mem => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.mem_usage_bytes.cmp(&b.mem_usage_bytes)
                    },
                    ProcessSortType::MemPercent => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        FloatOrd(a.mem_usage_percent).cmp(&FloatOrd(b.mem_usage_percent))
                    },
                    ProcessSortType::Rps => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.read_bytes_per_sec.cmp(&b.read_bytes_per_sec)
                    },
                    ProcessSortType::Wps => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.write_bytes_per_sec.cmp(&b.write_bytes_per_sec)
                    },
                    ProcessSortType::TotalRead => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.total_read_bytes.cmp(&b.total_read_bytes)
                    },
                    ProcessSortType::TotalWrite => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.total_write_bytes.cmp(&b.total_write_bytes)
                    },
                    ProcessSortType::User => {
                        #[cfg(target_family = "unix")]
                        {
                            |a: &&ProcessHarvest, b: &&ProcessHarvest| a.user.cmp(&b.user)
                        }
                        #[cfg(not(target_family = "unix"))]
                        {
                            |_a: &&ProcessHarvest, _b: &&ProcessHarvest| std::cmp::Ordering::Equal
                        }
                    }
                    ProcessSortType::State => |a: &&ProcessHarvest, b: &&ProcessHarvest| {
                        a.process_state.cmp(&b.process_state)
                    },
                },
            ))
        };

        self.display_data = if let SortStatus::SortDescending = self
            .process_table
            .current_sorting_column()
            .sortable_column
            .sorting_status()
        {
            Either::Left(filtered_sorted_iter.rev())
        } else {
            Either::Right(filtered_sorted_iter)
        }
        .map(|process| {
            self.process_table
                .columns()
                .iter()
                .map(|column| match &column.sort_type {
                    ProcessSortType::Pid => (process.pid.to_string().into(), None, None),
                    ProcessSortType::Count => (
                        if self.is_using_command() {
                            data_collection
                                .process_cmd_pid_map
                                .get(&process.command)
                                .map(|v| v.len())
                                .unwrap_or(0)
                                .to_string()
                                .into()
                        } else {
                            data_collection
                                .process_name_pid_map
                                .get(&process.name)
                                .map(|v| v.len())
                                .unwrap_or(0)
                                .to_string()
                                .into()
                        },
                        None,
                        None,
                    ),
                    ProcessSortType::Name => (process.name.clone().into(), None, None),
                    ProcessSortType::Command => (process.command.clone().into(), None, None),
                    ProcessSortType::Cpu => (
                        format!("{:.1}%", process.cpu_usage_percent).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::Mem => (
                        get_string_with_bytes(process.mem_usage_bytes).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::MemPercent => (
                        format!("{:.1}%", process.mem_usage_percent).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::Rps => (
                        get_string_with_bytes(process.read_bytes_per_sec).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::Wps => (
                        get_string_with_bytes(process.write_bytes_per_sec).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::TotalRead => (
                        get_string_with_bytes(process.total_read_bytes).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::TotalWrite => (
                        get_string_with_bytes(process.total_write_bytes).into(),
                        None,
                        None,
                    ),
                    ProcessSortType::User => (process.user.clone(), None, None),
                    ProcessSortType::State => (
                        process.process_state.clone().into(),
                        None, // Currently disabled; what happens if you try to sort in the shortened form?
                        None,
                    ),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }

    fn handle_widget_selection_left(&mut self) -> SelectionAction {
        if self.show_sort {
            if let ProcessManagerSelection::Processes = self.selected {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Sort;
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }

    fn handle_widget_selection_right(&mut self) -> SelectionAction {
        if self.show_sort {
            if let ProcessManagerSelection::Sort = self.selected {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Processes;
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }

    fn handle_widget_selection_up(&mut self) -> SelectionAction {
        if self.show_search {
            if let ProcessManagerSelection::Search = self.selected {
                let prev = self.prev_selected;
                self.prev_selected = self.selected;
                if self.show_sort && prev == ProcessManagerSelection::Sort {
                    self.selected = ProcessManagerSelection::Sort;
                } else {
                    self.selected = ProcessManagerSelection::Processes;
                }
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }

    fn handle_widget_selection_down(&mut self) -> SelectionAction {
        if self.show_search {
            if let ProcessManagerSelection::Processes = self.selected {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Search;
                SelectionAction::Handled
            } else if self.show_sort && self.selected == ProcessManagerSelection::Sort {
                self.prev_selected = self.selected;
                self.selected = ProcessManagerSelection::Search;
                SelectionAction::Handled
            } else {
                SelectionAction::NotHandled
            }
        } else {
            SelectionAction::NotHandled
        }
    }
}
