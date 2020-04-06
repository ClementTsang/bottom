use std::cmp::{max, min};

use crate::{
    app::{self, App, ProcWidgetState},
    canvas::{
        drawing_utils::{
            get_search_start_position, get_start_position, get_variable_intrinsic_widths,
        },
        Painter,
    },
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget},
};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

pub trait ProcessTableWidget {
    fn draw_process_and_search<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl ProcessTableWidget for Painter {
    fn draw_process_and_search<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(process_widget_state) = app_state.proc_state.widget_states.get(&widget_id) {
            let search_width = if draw_border { 5 } else { 3 };
            if process_widget_state.is_search_enabled() {
                let processes_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(search_width)].as_ref())
                    .split(draw_loc);

                self.draw_processes_table(f, app_state, processes_chunk[0], draw_border, widget_id);
                self.draw_search_field(
                    f,
                    app_state,
                    processes_chunk[1],
                    draw_border,
                    widget_id + 1,
                );
            } else {
                self.draw_processes_table(f, app_state, draw_loc, draw_border, widget_id);
            }
        }
    }

    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(proc_widget_state) = app_state.proc_state.widget_states.get_mut(&widget_id) {
            if let Some(process_data) = &app_state
                .canvas_data
                .finalized_process_data_map
                .get(&widget_id)
            {
                // Admittedly this is kinda a hack... but we need to:
                // * Scroll
                // * Show/hide elements based on scroll position
                //
                // As such, we use a process_counter to know when we've
                // hit the process we've currently scrolled to.
                // We also need to move the list - we can
                // do so by hiding some elements!
                let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
                let is_on_widget = widget_id == app_state.current_widget.widget_id;

                let position = get_start_position(
                    num_rows,
                    &proc_widget_state.scroll_state.scroll_direction,
                    &mut proc_widget_state.scroll_state.previous_scroll_position,
                    proc_widget_state.scroll_state.current_scroll_position,
                    app_state.is_resized,
                );

                // Sanity check
                let start_position = if position >= process_data.len() as u64 {
                    std::cmp::max(0, process_data.len() as i64 - 1) as u64
                } else {
                    position
                };

                let sliced_vec = &process_data[start_position as usize..];
                let mut process_counter: i64 = 0;

                // Draw!
                let process_rows = sliced_vec.iter().map(|process| {
                    let stringified_process_vec: Vec<String> = vec![
                        if proc_widget_state.is_grouped {
                            process.group_pids.len().to_string()
                        } else {
                            process.pid.to_string()
                        },
                        process.name.clone(),
                        format!("{:.1}%", process.cpu_usage),
                        format!("{:.1}%", process.mem_usage),
                    ];
                    Row::StyledData(
                        stringified_process_vec.into_iter(),
                        if is_on_widget {
                            if process_counter as u64
                                == proc_widget_state.scroll_state.current_scroll_position
                                    - start_position
                            {
                                process_counter = -1;
                                self.colours.currently_selected_text_style
                            } else {
                                if process_counter >= 0 {
                                    process_counter += 1;
                                }
                                self.colours.text_style
                            }
                        } else {
                            self.colours.text_style
                        },
                    )
                });

                use app::data_harvester::processes::ProcessSorting;
                let mut pid_or_name = if proc_widget_state.is_grouped {
                    "Count"
                } else {
                    "PID(p)"
                }
                .to_string();
                let mut name = "Name(n)".to_string();
                let mut cpu = "CPU%(c)".to_string();
                let mut mem = "Mem%(m)".to_string();

                let direction_val = if proc_widget_state.process_sorting_reverse {
                    "▼".to_string()
                } else {
                    "▲".to_string()
                };

                match proc_widget_state.process_sorting_type {
                    ProcessSorting::CPU => cpu += &direction_val,
                    ProcessSorting::MEM => mem += &direction_val,
                    ProcessSorting::PID => pid_or_name += &direction_val,
                    ProcessSorting::NAME => name += &direction_val,
                };

                let process_headers = [pid_or_name, name, cpu, mem];
                let process_headers_lens: Vec<usize> = process_headers
                    .iter()
                    .map(|entry| entry.len())
                    .collect::<Vec<_>>();

                // Calculate widths
                let width = f64::from(draw_loc.width);
                let width_ratios = [0.2, 0.4, 0.2, 0.2];
                let variable_intrinsic_results = get_variable_intrinsic_widths(
                    width as u16,
                    &width_ratios,
                    &process_headers_lens,
                );
                let intrinsic_widths =
                    &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

                let title = if draw_border {
                    if app_state.is_expanded
                        && !proc_widget_state
                            .process_search_state
                            .search_state
                            .is_enabled
                    {
                        const TITLE_BASE: &str = " Processes ── Esc to go back ";
                        let repeat_num = max(
                            0,
                            draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                        );
                        let result_title = format!(
                            " Processes ─{}─ Esc to go back ",
                            "─".repeat(repeat_num as usize)
                        );

                        result_title
                    } else {
                        " Processes ".to_string()
                    }
                } else {
                    String::default()
                };

                let border_and_title_style = if is_on_widget {
                    self.colours.highlighted_border_style
                } else {
                    self.colours.border_style
                };

                let process_block = if draw_border {
                    Block::default()
                        .title(&title)
                        .title_style(if app_state.is_expanded {
                            border_and_title_style
                        } else {
                            self.colours.widget_title_style
                        })
                        .borders(Borders::ALL)
                        .border_style(border_and_title_style)
                } else if is_on_widget {
                    Block::default()
                        .borders(*SIDE_BORDERS)
                        .border_style(self.colours.highlighted_border_style)
                } else {
                    Block::default().borders(Borders::NONE)
                };

                let margined_draw_loc = Layout::default()
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                    .direction(Direction::Horizontal)
                    .split(draw_loc);

                Table::new(process_headers.iter(), process_rows)
                    .block(process_block)
                    .header_style(self.colours.table_header_style)
                    .widths(
                        &(intrinsic_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    )
                    .render(f, margined_draw_loc[0]);
            }
        }
    }

    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        fn get_prompt_text<'a>(proc_widget_state: &ProcWidgetState) -> &'a str {
            let pid_search_text = "Search by PID (Tab for Name): ";
            let name_search_text = "Search by Name (Tab for PID): ";
            let grouped_search_text = "Search by Name: ";

            if proc_widget_state.is_grouped {
                grouped_search_text
            } else if proc_widget_state.process_search_state.is_searching_with_pid {
                pid_search_text
            } else {
                name_search_text
            }
        }

        fn build_query<'a>(
            is_on_widget: bool, grapheme_indices: GraphemeIndices<'a>, start_position: usize,
            cursor_position: usize, query: &str, currently_selected_text_style: tui::style::Style,
            text_style: tui::style::Style,
        ) -> Vec<Text<'a>> {
            let mut current_grapheme_posn = 0;

            if is_on_widget {
                let mut res = grapheme_indices
                    .filter_map(|grapheme| {
                        current_grapheme_posn += UnicodeWidthStr::width(grapheme.1);

                        if current_grapheme_posn <= start_position {
                            None
                        } else {
                            let styled = if grapheme.0 == cursor_position {
                                Text::styled(grapheme.1, currently_selected_text_style)
                            } else {
                                Text::styled(grapheme.1, text_style)
                            };
                            Some(styled)
                        }
                    })
                    .collect::<Vec<_>>();

                if cursor_position >= query.len() {
                    res.push(Text::styled(" ", currently_selected_text_style))
                }

                res
            } else {
                // This is easier - we just need to get a range of graphemes, rather than
                // dealing with possibly inserting a cursor (as none is shown!)
                grapheme_indices
                    .filter_map(|grapheme| {
                        current_grapheme_posn += UnicodeWidthStr::width(grapheme.1);
                        if current_grapheme_posn <= start_position {
                            None
                        } else {
                            let styled = Text::styled(grapheme.1, text_style);
                            Some(styled)
                        }
                    })
                    .collect::<Vec<_>>()
            }
        }

        if let Some(proc_widget_state) =
            app_state.proc_state.widget_states.get_mut(&(widget_id - 1))
        {
            let chosen_text = get_prompt_text(&proc_widget_state);

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let num_columns = draw_loc.width as usize;
            let small_mode = chosen_text.len() != min(num_columns / 2, chosen_text.len());
            let search_title: &str = if !small_mode {
                chosen_text
            } else if chosen_text.is_empty() {
                ""
            } else if proc_widget_state.process_search_state.is_searching_with_pid {
                "p> "
            } else {
                "n> "
            };

            let num_chars_for_text = search_title.len();

            let mut search_text = vec![Text::styled(search_title, self.colours.table_header_style)];

            let cursor_position = proc_widget_state.get_cursor_position();
            let current_cursor_position = proc_widget_state.get_char_cursor_position();

            let start_position: usize = get_search_start_position(
                num_columns - num_chars_for_text - 5,
                &proc_widget_state
                    .process_search_state
                    .search_state
                    .cursor_direction,
                &mut proc_widget_state
                    .process_search_state
                    .search_state
                    .cursor_bar,
                current_cursor_position,
                app_state.is_resized,
            );

            let query = proc_widget_state.get_current_search_query().as_str();
            let grapheme_indices = UnicodeSegmentation::grapheme_indices(query, true);
            let query_with_cursor: Vec<Text<'_>> = build_query(
                is_on_widget,
                grapheme_indices,
                start_position,
                cursor_position,
                query,
                self.colours.currently_selected_text_style,
                self.colours.text_style,
            );

            // Text options shamelessly stolen from VS Code.
            let case_style = if !proc_widget_state.process_search_state.is_ignoring_case {
                self.colours.currently_selected_text_style
            } else {
                self.colours.text_style
            };

            let whole_word_style = if proc_widget_state
                .process_search_state
                .is_searching_whole_word
            {
                self.colours.currently_selected_text_style
            } else {
                self.colours.text_style
            };

            let regex_style = if proc_widget_state
                .process_search_state
                .is_searching_with_regex
            {
                self.colours.currently_selected_text_style
            } else {
                self.colours.text_style
            };

            let mut option_text = vec![];
            let case_text = format!(
                "{}({})",
                if small_mode { "Case" } else { "Match Case " },
                if self.is_mac_os { "F1" } else { "Alt+C" },
            );

            let whole_text = format!(
                "{}({})",
                if small_mode {
                    "Whole"
                } else {
                    "Match Whole Word "
                },
                if self.is_mac_os { "F2" } else { "Alt+W" },
            );

            let regex_text = format!(
                "{}({})",
                if small_mode { "Regex" } else { "Use Regex " },
                if self.is_mac_os { "F3" } else { "Alt+R" },
            );

            let option_row = vec![
                Text::raw("\n\n"),
                Text::styled(&case_text, case_style),
                Text::raw(if small_mode { "  " } else { "     " }),
                Text::styled(&whole_text, whole_word_style),
                Text::raw(if small_mode { "  " } else { "     " }),
                Text::styled(&regex_text, regex_style),
            ];
            option_text.extend(option_row);

            search_text.extend(query_with_cursor);
            search_text.extend(option_text);

            let current_border_style = if proc_widget_state
                .process_search_state
                .search_state
                .is_invalid_search
            {
                *INVALID_REGEX_STYLE
            } else if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let title = if draw_border {
                const TITLE_BASE: &str = " Esc to close ";

                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                format!("{} Esc to close ", "─".repeat(repeat_num as usize))
            } else {
                String::new()
            };

            let process_search_block = if draw_border {
                Block::default()
                    .title(&title)
                    .title_style(current_border_style)
                    .borders(Borders::ALL)
                    .border_style(current_border_style)
            } else if is_on_widget {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(current_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc);

            Paragraph::new(search_text.iter())
                .block(process_search_block)
                .style(self.colours.text_style)
                .alignment(Alignment::Left)
                .wrap(false)
                .render(f, margined_draw_loc[0]);
        }
    }
}
