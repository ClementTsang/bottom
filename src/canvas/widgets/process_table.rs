use crate::{
    app::{self, App},
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
    widgets::{Block, Borders, Paragraph, Row, Table, Text},
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
            let search_height = if draw_border { 5 } else { 3 };
            if process_widget_state.is_search_enabled() {
                let processes_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(search_height)].as_ref())
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
                let is_on_widget = widget_id == app_state.current_widget.widget_id;

                let position = get_start_position(
                    draw_loc.height.saturating_sub(self.table_height_offset) as u64,
                    &proc_widget_state.scroll_state.scroll_direction,
                    &mut proc_widget_state.scroll_state.previous_scroll_position,
                    proc_widget_state.scroll_state.current_scroll_position,
                    app_state.is_force_redraw,
                );

                // Sanity check
                let start_position = if position >= process_data.len() as u64 {
                    process_data.len().saturating_sub(1) as u64
                } else {
                    position
                };

                let sliced_vec = &process_data[start_position as usize..];
                let proc_table_state = &mut proc_widget_state.scroll_state.table_state;
                proc_table_state.select(Some(
                    (proc_widget_state.scroll_state.current_scroll_position - start_position)
                        as usize,
                ));
                let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                    0
                } else {
                    app_state.app_config_fields.table_gap
                };

                // Draw!
                let is_proc_widget_grouped = proc_widget_state.is_grouped;
                let process_rows = sliced_vec.iter().map(|process| {
                    Row::Data(
                        vec![
                            if is_proc_widget_grouped {
                                process.group_pids.len().to_string()
                            } else {
                                process.pid.to_string()
                            },
                            process.name.clone(),
                            format!("{:.1}%", process.cpu_usage),
                            format!("{:.1}%", process.mem_usage),
                            process.read_per_sec.to_string(),
                            process.write_per_sec.to_string(),
                            process.total_read.to_string(),
                            process.total_write.to_string(),
                            process.process_states.to_string(),
                        ]
                        .into_iter(),
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
                let rps = "R/s".to_string();
                let wps = "W/s".to_string();
                let total_read = "Read".to_string();
                let total_write = "Write".to_string();
                // let process_state = "State".to_string();

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

                let process_headers = [
                    pid_or_name,
                    name,
                    cpu,
                    mem,
                    rps,
                    wps,
                    total_read,
                    total_write,
                    // process_state,
                ];
                let process_headers_lens: Vec<usize> = process_headers
                    .iter()
                    .map(|entry| entry.len())
                    .collect::<Vec<_>>();

                // Calculate widths
                let width = f64::from(draw_loc.width);
                let width_ratios = [0.1, 0.2, 0.1, 0.1, 0.1, 0.1, 0.15, 0.15];
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
                        format!(
                            " Processes ─{}─ Esc to go back ",
                            "─".repeat(
                                usize::from(draw_loc.width)
                                    .saturating_sub(TITLE_BASE.chars().count() + 2)
                            )
                        )
                    } else {
                        " Processes ".to_string()
                    }
                } else {
                    String::default()
                };

                let (border_and_title_style, highlight_style) = if is_on_widget {
                    (
                        self.colours.highlighted_border_style,
                        self.colours.currently_selected_text_style,
                    )
                } else {
                    (self.colours.border_style, self.colours.text_style)
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

                f.render_stateful_widget(
                    Table::new(process_headers.iter(), process_rows)
                        .block(process_block)
                        .header_style(self.colours.table_header_style)
                        .highlight_style(highlight_style)
                        .style(self.colours.text_style)
                        .widths(
                            &(intrinsic_widths
                                .iter()
                                .map(|calculated_width| {
                                    Constraint::Length(*calculated_width as u16)
                                })
                                .collect::<Vec<_>>()),
                        )
                        .header_gap(table_gap),
                    margined_draw_loc[0],
                    proc_table_state,
                );
            }
        }
    }

    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
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
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let num_columns = draw_loc.width as usize;
            let search_title = "> ";

            let num_chars_for_text = search_title.len();
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
                app_state.is_force_redraw,
            );

            let mut search_text = vec![Text::styled(
                search_title,
                if is_on_widget {
                    self.colours.table_header_style
                } else {
                    self.colours.text_style
                },
            )];
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

            let option_text = vec![
                Text::raw("\n"),
                Text::styled(
                    format!("Case({})", if self.is_mac_os { "F1" } else { "Alt+C" }),
                    case_style,
                ),
                Text::raw("  "),
                Text::styled(
                    format!("Whole({})", if self.is_mac_os { "F2" } else { "Alt+W" }),
                    whole_word_style,
                ),
                Text::raw("  "),
                Text::styled(
                    format!("Regex({})", if self.is_mac_os { "F3" } else { "Alt+R" }),
                    regex_style,
                ),
            ];

            search_text.extend(query_with_cursor);
            search_text.push(Text::styled(
                format!(
                    "\n{}",
                    if let Some(err) = &proc_widget_state
                        .process_search_state
                        .search_state
                        .error_message
                    {
                        err.as_str()
                    } else {
                        ""
                    }
                ),
                self.colours.invalid_query_style,
            ));
            search_text.extend(option_text);

            let current_border_style = if proc_widget_state
                .process_search_state
                .search_state
                .is_invalid_search
            {
                self.colours.invalid_query_style
            } else if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let title = if draw_border {
                const TITLE_BASE: &str = " Esc to close ";

                let repeat_num =
                    usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2);
                format!("{} Esc to close ", "─".repeat(repeat_num))
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

            f.render_widget(
                Paragraph::new(search_text.iter())
                    .block(process_search_block)
                    .style(self.colours.text_style)
                    .alignment(Alignment::Left)
                    .wrap(false),
                margined_draw_loc[0],
            );
        }
    }
}
