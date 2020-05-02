use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use unicode_segmentation::GraphemeCursor;

use tui::widgets::TableState;

use crate::{
    app::{layout_manager::BottomWidgetType, query::*},
    constants,
    data_harvester::processes,
    utils::error::{BottomError::*, Result},
};

#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    UP,
    // DOWN means scrolling down --- this usually INCREMENTS
    DOWN,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::DOWN
    }
}

#[derive(Debug)]
pub enum CursorDirection {
    LEFT,
    RIGHT,
}

/// AppScrollWidgetState deals with fields for a scrollable app's current state.
#[derive(Default)]
pub struct AppScrollWidgetState {
    pub current_scroll_position: u64,
    pub previous_scroll_position: u64,
    pub scroll_direction: ScrollDirection,
    pub table_state: TableState,
}

#[derive(Default)]
pub struct AppDeleteDialogState {
    pub is_showing_dd: bool,
    pub is_on_yes: bool, // Defaults to "No"
}

pub struct AppHelpDialogState {
    pub is_showing_help: bool,
    pub scroll_state: ParagraphScrollState,
    pub index_shortcuts: Vec<u16>,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
            scroll_state: ParagraphScrollState::default(),
            index_shortcuts: vec![0; constants::HELP_TEXT.len()],
        }
    }
}

/// AppSearchState deals with generic searching (I might do this in the future).
pub struct AppSearchState {
    pub is_enabled: bool,
    pub current_search_query: String,
    pub is_blank_search: bool,
    pub is_invalid_search: bool,
    pub grapheme_cursor: GraphemeCursor,
    pub cursor_direction: CursorDirection,
    pub cursor_bar: usize,
    /// This represents the position in terms of CHARACTERS, not graphemes
    pub char_cursor_position: usize,
    /// The query
    pub query: Option<Query>,
}

impl Default for AppSearchState {
    fn default() -> Self {
        AppSearchState {
            is_enabled: false,
            current_search_query: String::default(),
            is_invalid_search: false,
            is_blank_search: true,
            grapheme_cursor: GraphemeCursor::new(0, 0, true),
            cursor_direction: CursorDirection::RIGHT,
            cursor_bar: 0,
            char_cursor_position: 0,
            query: None,
        }
    }
}

impl AppSearchState {
    /// Returns a reset but still enabled app search state
    pub fn reset(&mut self) {
        *self = AppSearchState {
            is_enabled: self.is_enabled,
            ..AppSearchState::default()
        }
    }

    pub fn is_invalid_or_blank_search(&self) -> bool {
        self.is_blank_search || self.is_invalid_search
    }
}

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

pub struct ProcWidgetState {
    pub process_search_state: ProcessSearchState,
    pub is_grouped: bool,
    pub scroll_state: AppScrollWidgetState,
    pub process_sorting_type: processes::ProcessSorting,
    pub process_sorting_reverse: bool,
}

impl ProcWidgetState {
    pub fn init(
        is_case_sensitive: bool, is_match_whole_word: bool, is_use_regex: bool, is_grouped: bool,
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

        ProcWidgetState {
            process_search_state,
            is_grouped,
            scroll_state: AppScrollWidgetState::default(),
            process_sorting_type: processes::ProcessSorting::CPU,
            process_sorting_reverse: true,
        }
    }

    pub fn get_cursor_position(&self) -> usize {
        self.process_search_state
            .search_state
            .grapheme_cursor
            .cur_cursor()
    }

    pub fn get_char_cursor_position(&self) -> usize {
        self.process_search_state.search_state.char_cursor_position
    }

    pub fn is_search_enabled(&self) -> bool {
        self.process_search_state.search_state.is_enabled
    }

    pub fn get_current_search_query(&self) -> &String {
        &self.process_search_state.search_state.current_search_query
    }

    pub fn update_query(&mut self) {
        if self
            .process_search_state
            .search_state
            .current_search_query
            .is_empty()
        {
            self.process_search_state.search_state.is_invalid_search = false;
            self.process_search_state.search_state.is_blank_search = true;
        } else if let Ok(parsed_query) = self.parse_query() {
            self.process_search_state.search_state.query = Some(parsed_query);
            self.process_search_state.search_state.is_blank_search = false;
            self.process_search_state.search_state.is_invalid_search = false;
        } else {
            self.process_search_state.search_state.is_blank_search = false;
            self.process_search_state.search_state.is_invalid_search = true;
        }
        self.scroll_state.previous_scroll_position = 0;
        self.scroll_state.current_scroll_position = 0;
    }

    pub fn clear_search(&mut self) {
        self.process_search_state.search_state.reset();
    }

    pub fn search_walk_forward(&mut self, start_position: usize) {
        self.process_search_state
            .search_state
            .grapheme_cursor
            .next_boundary(
                &self.process_search_state.search_state.current_search_query[start_position..],
                start_position,
            )
            .unwrap();
    }

    pub fn search_walk_back(&mut self, start_position: usize) {
        self.process_search_state
            .search_state
            .grapheme_cursor
            .prev_boundary(
                &self.process_search_state.search_state.current_search_query[..start_position],
                0,
            )
            .unwrap();
    }

    /// The filtering function.  Based on the results of the query.
    pub fn matches_filter(&self) -> bool {
        // The way this will have to work is that given a "query" structure, we have
        // to filter based on it.

        false
    }

    /// In charge of parsing the given query.
    /// We are defining the following language for a query (case-insensitive prefixes):
    ///
    /// - Process names: No prefix required, can use regex, match word, or case.
    ///   Enclosing anything, including prefixes, in quotes, means we treat it as an entire process
    ///   rather than a prefix.
    /// - PIDs: Use prefix `pid`, can use regex or match word (case is irrelevant).
    /// - CPU: Use prefix `cpu`, cannot use r/m/c (regex, match word, case).  Can compare.
    /// - MEM: Use prefix `mem`, cannot use r/m/c.  Can compare.
    /// - STATE: Use prefix `state`, TODO when we update how state looks in 0.5 probably.
    /// - Read/s: Use prefix `r`.  Can compare.
    /// - Write/s: Use prefix `w`.  Can compare.
    /// - Total read: Use prefix `read`.  Can compare.
    /// - Total write: Use prefix `write`.  Can compare.
    ///
    /// For queries, whitespaces are our delimiters.  We will merge together any adjacent non-prefixed
    /// or quoted elements after splitting to treat as process names.
    /// Furthermore, we want to support boolean joiners like AND and OR, and brackets.
    fn parse_query(&self) -> Result<Query> {
        fn process_string_to_filter(query: &mut VecDeque<String>) -> Result<Query> {
            Ok(Query {
                query: process_and(query)?,
            })
        }

        fn process_and(query: &mut VecDeque<String>) -> Result<And> {
            let mut lhs = process_or(query)?;
            let mut rhs: Option<Box<Or>> = None;

            while let Some(queue_top) = query.front() {
                if queue_top.to_lowercase() == "and" {
                    query.pop_front();
                    rhs = Some(Box::new(process_or(query)?));

                    if let Some(queue_next) = query.front() {
                        if queue_next.to_lowercase() == "and" {
                            // Must merge LHS and RHS
                            lhs = Or {
                                lhs: Prefix {
                                    and: Some(Box::new(And { lhs, rhs })),
                                    regex_prefix: None,
                                    compare_prefix: None,
                                },
                                rhs: None,
                            };
                            rhs = None;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(And { lhs, rhs })
        }

        fn process_or(query: &mut VecDeque<String>) -> Result<Or> {
            let mut lhs = process_prefix(query)?;
            let mut rhs: Option<Box<Prefix>> = None;

            while let Some(queue_top) = query.front() {
                if queue_top.to_lowercase() == "or" {
                    query.pop_front();
                    rhs = Some(Box::new(process_prefix(query)?));

                    if let Some(queue_next) = query.front() {
                        if queue_next.to_lowercase() == "or" {
                            // Must merge LHS and RHS
                            lhs = Prefix {
                                and: Some(Box::new(And {
                                    lhs: Or { lhs, rhs },
                                    rhs: None,
                                })),
                                regex_prefix: None,
                                compare_prefix: None,
                            };
                            rhs = None;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(Or { lhs, rhs })
        }

        fn process_prefix(query: &mut VecDeque<String>) -> Result<Prefix> {
            if let Some(queue_top) = query.pop_front() {
                if queue_top == "(" {
                    // Get content within bracket; and check if paren is complete
                    let and = process_and(query)?;
                    if let Some(close_paren) = query.pop_front() {
                        if close_paren.to_lowercase() == ")" {
                            return Ok(Prefix {
                                and: Some(Box::new(and)),
                                regex_prefix: None,
                                compare_prefix: None,
                            });
                        } else {
                            return Err(QueryError("Missing closing parentheses".into()));
                        }
                    } else {
                        return Err(QueryError("Missing closing parentheses".into()));
                    }
                } else if queue_top == ")" {
                    // This is actually caught by the regex creation, but it seems a bit
                    // sloppy to leave that up to that to do so...

                    return Err(QueryError("Missing opening parentheses".into()));
                } else {
                    //  Get prefix type...
                    let prefix_type = queue_top.parse::<PrefixType>()?;
                    let content = if let PrefixType::Name = prefix_type {
                        Some(queue_top)
                    } else {
                        query.pop_front()
                    };

                    if let Some(content) = content {
                        match &prefix_type {
                            PrefixType::Name => {
                                return Ok(Prefix {
                                    and: None,
                                    regex_prefix: Some((prefix_type, StringQuery::Value(content))),
                                    compare_prefix: None,
                                })
                            }
                            PrefixType::Pid => {
                                // We have to check if someone put an "="...
                                if content == "=" {
                                    // Check next string if possible
                                    if let Some(queue_next) = query.pop_front() {
                                        return Ok(Prefix {
                                            and: None,
                                            regex_prefix: Some((
                                                prefix_type,
                                                StringQuery::Value(queue_next),
                                            )),
                                            compare_prefix: None,
                                        });
                                    }
                                } else {
                                    return Ok(Prefix {
                                        and: None,
                                        regex_prefix: Some((
                                            prefix_type,
                                            StringQuery::Value(content),
                                        )),
                                        compare_prefix: None,
                                    });
                                }
                            }
                            _ => {
                                // Now we gotta parse the content... yay.

                                let mut condition: Option<QueryComparison> = None;
                                let mut value: Option<f64> = None;

                                if content == "=" {
                                    // TODO: Do we want to allow just an empty space to work here too?  ie: cpu 5?
                                    condition = Some(QueryComparison::Equal);
                                    if let Some(queue_next) = query.pop_front() {
                                        value = queue_next.parse::<f64>().ok();
                                    }
                                } else if content == ">" || content == "<" {
                                    // We also have to check if the next string is an "="...
                                    if let Some(queue_next) = query.pop_front() {
                                        if queue_next == "=" {
                                            condition = Some(if content == ">" {
                                                QueryComparison::GreaterOrEqual
                                            } else {
                                                QueryComparison::LessOrEqual
                                            });
                                            if let Some(queue_next_next) = query.pop_front() {
                                                value = queue_next_next.parse::<f64>().ok();
                                            }
                                        } else {
                                            condition = Some(if content == ">" {
                                                QueryComparison::Greater
                                            } else {
                                                QueryComparison::Less
                                            });
                                            value = queue_next.parse::<f64>().ok();
                                        }
                                    }
                                }

                                if let Some(condition) = condition {
                                    if let Some(value) = value {
                                        return Ok(Prefix {
                                            and: None,
                                            regex_prefix: None,
                                            compare_prefix: Some((
                                                prefix_type,
                                                NumericalQuery { condition, value },
                                            )),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Err(QueryError("Failed to parse comparator.".into()))
        }

        let mut split_query = VecDeque::new();

        self.get_current_search_query()
            .split_whitespace()
            .for_each(|s| {
                // From https://stackoverflow.com/a/56923739 in order to get a split but include the parentheses
                let mut last = 0;
                for (index, matched) in s.match_indices(|x| ['=', '>', '<', '(', ')'].contains(&x))
                {
                    if last != index {
                        split_query.push_back(s[last..index].to_owned());
                    }
                    split_query.push_back(matched.to_owned());
                    last = index + matched.len();
                }
                if last < s.len() {
                    split_query.push_back(s[last..].to_owned());
                }
            });

        let mut process_filter = process_string_to_filter(&mut split_query)?;
        process_filter.process_regexes(
            self.process_search_state.is_searching_whole_word,
            self.process_search_state.is_ignoring_case,
            self.process_search_state.is_searching_with_regex,
        )?;

        Ok(process_filter)
    }
}

pub struct ProcState {
    pub widget_states: HashMap<u64, ProcWidgetState>,
    pub force_update: Option<u64>,
    pub force_update_all: bool,
}

impl ProcState {
    pub fn init(widget_states: HashMap<u64, ProcWidgetState>) -> Self {
        ProcState {
            widget_states,
            force_update: None,
            force_update_all: false,
        }
    }
}

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl NetWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}

pub struct NetState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, NetWidgetState>,
}

impl NetState {
    pub fn init(widget_states: HashMap<u64, NetWidgetState>) -> Self {
        NetState {
            force_update: None,
            widget_states,
        }
    }
}

pub struct CpuWidgetState {
    pub current_display_time: u64,
    pub is_legend_hidden: bool,
    pub is_showing_tray: bool,
    pub core_show_vec: Vec<bool>,
    pub num_cpus_shown: usize,
    pub autohide_timer: Option<Instant>,
    pub scroll_state: AppScrollWidgetState,
}

impl CpuWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            is_showing_tray: false,
            core_show_vec: Vec::new(),
            num_cpus_shown: 0,
            autohide_timer,
            scroll_state: AppScrollWidgetState::default(),
        }
    }
}

pub struct CpuState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, CpuWidgetState>,
    pub num_cpus_total: usize,
}

impl CpuState {
    pub fn init(widget_states: HashMap<u64, CpuWidgetState>) -> Self {
        CpuState {
            force_update: None,
            widget_states,
            num_cpus_total: 0,
        }
    }
}

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl MemWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}

pub struct MemState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, MemWidgetState>,
}

impl MemState {
    pub fn init(widget_states: HashMap<u64, MemWidgetState>) -> Self {
        MemState {
            force_update: None,
            widget_states,
        }
    }
}

pub struct TempWidgetState {
    pub scroll_state: AppScrollWidgetState,
}

impl TempWidgetState {
    pub fn init() -> Self {
        TempWidgetState {
            scroll_state: AppScrollWidgetState::default(),
        }
    }
}

pub struct TempState {
    pub widget_states: HashMap<u64, TempWidgetState>,
}

impl TempState {
    pub fn init(widget_states: HashMap<u64, TempWidgetState>) -> Self {
        TempState { widget_states }
    }
}

pub struct DiskWidgetState {
    pub scroll_state: AppScrollWidgetState,
}

impl DiskWidgetState {
    pub fn init() -> Self {
        DiskWidgetState {
            scroll_state: AppScrollWidgetState::default(),
        }
    }
}

pub struct DiskState {
    pub widget_states: HashMap<u64, DiskWidgetState>,
}

impl DiskState {
    pub fn init(widget_states: HashMap<u64, DiskWidgetState>) -> Self {
        DiskState { widget_states }
    }
}
pub struct BasicTableWidgetState {
    // Since this is intended (currently) to only be used for ONE widget, that's
    // how it's going to be written.  If we want to allow for multiple of these,
    // then we can expand outwards with a normal BasicTableState and a hashmap
    pub currently_displayed_widget_type: BottomWidgetType,
    pub currently_displayed_widget_id: u64,
    pub widget_id: i64,
}

#[derive(Default)]
pub struct BatteryWidgetState {
    pub currently_selected_battery_index: usize,
}

pub struct BatteryState {
    pub widget_states: HashMap<u64, BatteryWidgetState>,
}

impl BatteryState {
    pub fn init(widget_states: HashMap<u64, BatteryWidgetState>) -> Self {
        BatteryState { widget_states }
    }
}

#[derive(Default)]
pub struct ParagraphScrollState {
    pub current_scroll_index: u16,
    pub max_scroll_index: u16,
}
