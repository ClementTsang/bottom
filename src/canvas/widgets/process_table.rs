use crate::{
    app::{data_harvester::processes::ProcessSorting, App},
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
    /// Draws and handles all process-related drawing.  Use this.
    /// - `widget_id` here represents the widget ID of the process widget itself!
    fn draw_process_features<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the process widget itself.
    ///
    /// This should not be directly called.
    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the search box itself --- NOT the process widget
    /// state that is stored.
    ///
    /// This should not be directly called.
    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the sort box itself --- NOT the process widget
    /// state that is stored.
    ///
    /// This should not be directly called.
    fn draw_process_sort<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl ProcessTableWidget for Painter {
    fn draw_process_features<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(process_widget_state) = app_state.proc_state.widget_states.get(&widget_id) {
            let search_height = if draw_border { 5 } else { 3 };
            let is_sort_open = process_widget_state.is_sort_open;
            let header_len = process_widget_state.columns.longest_header_len;

            let mut proc_draw_loc = draw_loc;
            if process_widget_state.is_search_enabled() {
                let processes_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(search_height)].as_ref())
                    .split(draw_loc);
                proc_draw_loc = processes_chunk[0];

                self.draw_search_field(
                    f,
                    app_state,
                    processes_chunk[1],
                    draw_border,
                    widget_id + 1,
                );
            }

            if is_sort_open {
                let processes_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(header_len + 4), Constraint::Min(0)].as_ref())
                    .split(proc_draw_loc);
                proc_draw_loc = processes_chunk[1];

                self.draw_process_sort(
                    f,
                    app_state,
                    processes_chunk[0],
                    draw_border,
                    widget_id + 2,
                );
            }

            self.draw_processes_table(f, app_state, proc_draw_loc, draw_border, widget_id);
        }
    }

    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(proc_widget_state) = app_state.proc_state.widget_states.get_mut(&widget_id) {
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

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

                let position = get_start_position(
                    usize::from(draw_loc.height.saturating_sub(self.table_height_offset)),
                    &proc_widget_state.scroll_state.scroll_direction,
                    &mut proc_widget_state.scroll_state.previous_scroll_position,
                    proc_widget_state.scroll_state.current_scroll_position,
                    app_state.is_force_redraw,
                );

                // Sanity check
                let start_position = if position >= process_data.len() {
                    process_data.len().saturating_sub(1)
                } else {
                    position
                };

                let sliced_vec = &process_data[start_position..];
                let proc_table_state = &mut proc_widget_state.scroll_state.table_state;
                proc_table_state.select(Some(
                    proc_widget_state.scroll_state.current_scroll_position - start_position,
                ));
                let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                    0
                } else {
                    app_state.app_config_fields.table_gap
                };

                // Draw!
                let is_proc_widget_grouped = proc_widget_state.is_grouped;
                let is_using_command = proc_widget_state.is_using_command;
                let mem_enabled = proc_widget_state.columns.is_enabled(&ProcessSorting::Mem);
                let process_rows = sliced_vec.iter().map(|process| {
                    Row::Data(
                        vec![
                            if is_proc_widget_grouped {
                                process.group_pids.len().to_string()
                            } else {
                                process.pid.to_string()
                            },
                            if is_using_command {
                                process.command.clone()
                            } else {
                                process.name.clone()
                            },
                            format!("{:.1}%", process.cpu_percent_usage),
                            if mem_enabled {
                                format!("{:.0}{}", process.mem_usage_str.0, process.mem_usage_str.1)
                            } else {
                                format!("{:.1}%", process.mem_percent_usage)
                            },
                            process.read_per_sec.to_string(),
                            process.write_per_sec.to_string(),
                            process.total_read.to_string(),
                            process.total_write.to_string(),
                            process.process_state.to_string(),
                        ]
                        .into_iter(),
                    )
                });

                let process_headers = proc_widget_state.columns.get_column_headers(
                    &proc_widget_state.process_sorting_type,
                    proc_widget_state.process_sorting_reverse,
                );

                let process_headers_lens: Vec<usize> = process_headers
                    .iter()
                    .map(|entry| entry.len())
                    .collect::<Vec<_>>();

                // Calculate widths
                let width = f64::from(draw_loc.width);

                // TODO: This is a ugly work-around for now.
                let width_ratios = if proc_widget_state.is_grouped {
                    if proc_widget_state.is_using_command {
                        vec![0.05, 0.7, 0.05, 0.05, 0.0375, 0.0375, 0.0375, 0.0375]
                    } else {
                        vec![0.1, 0.2, 0.1, 0.1, 0.1, 0.1, 0.15, 0.15]
                    }
                } else if proc_widget_state.is_using_command {
                    vec![0.05, 0.7, 0.05, 0.05, 0.03, 0.03, 0.03, 0.03]
                } else {
                    vec![0.1, 0.2, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
                };
                let variable_intrinsic_results = get_variable_intrinsic_widths(
                    width as u16,
                    &width_ratios,
                    &process_headers_lens,
                );
                let intrinsic_widths =
                    &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

                let (border_and_title_style, highlight_style) = if is_on_widget {
                    (
                        self.colours.highlighted_border_style,
                        self.colours.currently_selected_text_style,
                    )
                } else {
                    (self.colours.border_style, self.colours.text_style)
                };

                let title = if draw_border {
                    if app_state.is_expanded
                        && !proc_widget_state
                            .process_search_state
                            .search_state
                            .is_enabled
                        && !proc_widget_state.is_sort_open
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

                let title_style = if app_state.is_expanded {
                    border_and_title_style
                } else {
                    self.colours.widget_title_style
                };

                let process_block = if draw_border {
                    Block::default()
                        .title(&title)
                        .title_style(title_style)
                        .borders(Borders::ALL)
                        .border_style(border_and_title_style)
                } else if is_on_widget {
                    Block::default()
                        .borders(*SIDE_BORDERS)
                        .border_style(self.colours.highlighted_border_style)
                } else {
                    Block::default().borders(Borders::NONE)
                };

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
                    margined_draw_loc,
                    proc_table_state,
                );
            }

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                    widget.top_left_corner = Some((margined_draw_loc.x, margined_draw_loc.y));
                    widget.bottom_right_corner = Some((
                        margined_draw_loc.x + margined_draw_loc.width,
                        margined_draw_loc.y + margined_draw_loc.height,
                    ));
                }
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
            let num_columns = usize::from(draw_loc.width);
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

            let query = proc_widget_state.get_current_search_query().as_str();
            let grapheme_indices = UnicodeSegmentation::grapheme_indices(query, true);

            let query_with_cursor = build_query(
                is_on_widget,
                grapheme_indices,
                start_position,
                cursor_position,
                query,
                self.colours.currently_selected_text_style,
                self.colours.text_style,
            );

            let mut search_text = {
                let mut search_vec = vec![Text::styled(
                    search_title,
                    if is_on_widget {
                        self.colours.table_header_style
                    } else {
                        self.colours.text_style
                    },
                )];
                search_vec.extend(query_with_cursor);
                search_vec
            };

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

            search_text.push(Text::raw("\n"));
            search_text.push(Text::styled(
                if let Some(err) = &proc_widget_state
                    .process_search_state
                    .search_state
                    .error_message
                {
                    err.as_str()
                } else {
                    ""
                },
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
                .split(draw_loc)[0];

            f.render_widget(
                Paragraph::new(search_text.iter())
                    .block(process_search_block)
                    .style(self.colours.text_style)
                    .alignment(Alignment::Left),
                margined_draw_loc,
            );

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                    widget.top_left_corner = Some((margined_draw_loc.x, margined_draw_loc.y));
                    widget.bottom_right_corner = Some((
                        margined_draw_loc.x + margined_draw_loc.width,
                        margined_draw_loc.y + margined_draw_loc.height,
                    ));
                }
            }
        }
    }

    fn draw_process_sort<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let is_on_widget = widget_id == app_state.current_widget.widget_id;

        if let Some(proc_widget_state) =
            app_state.proc_state.widget_states.get_mut(&(widget_id - 2))
        {
            let current_scroll_position = proc_widget_state.columns.current_scroll_position;
            let sort_string = proc_widget_state
                .columns
                .ordered_columns
                .iter()
                .filter(|column_type| {
                    proc_widget_state
                        .columns
                        .column_mapping
                        .get(&column_type)
                        .unwrap()
                        .enabled
                })
                .enumerate()
                .map(|(itx, column_type)| {
                    if current_scroll_position == itx {
                        (
                            column_type.to_string(),
                            self.colours.currently_selected_text_style,
                        )
                    } else {
                        (column_type.to_string(), self.colours.text_style)
                    }
                })
                .collect::<Vec<_>>();

            let position = get_start_position(
                usize::from(draw_loc.height.saturating_sub(self.table_height_offset)),
                &proc_widget_state.columns.scroll_direction,
                &mut proc_widget_state.columns.previous_scroll_position,
                current_scroll_position,
                app_state.is_force_redraw,
            );

            // Sanity check
            let start_position = if position >= sort_string.len() {
                sort_string.len().saturating_sub(1)
            } else {
                position
            };

            let sliced_vec = &sort_string[start_position..];

            let sort_options = sliced_vec
                .iter()
                .map(|(column, style)| Row::StyledData(vec![column].into_iter(), *style));

            let column_state = &mut proc_widget_state.columns.column_state;
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

            let process_sort_block = if draw_border {
                Block::default()
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
                .split(draw_loc)[0];

            f.render_stateful_widget(
                Table::new(["Sort By"].iter(), sort_options)
                    .block(process_sort_block)
                    .header_style(self.colours.table_header_style)
                    .widths(&[Constraint::Percentage(100)])
                    .header_gap(1),
                margined_draw_loc,
                column_state,
            );

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                    widget.top_left_corner = Some((margined_draw_loc.x, margined_draw_loc.y));
                    widget.bottom_right_corner = Some((
                        margined_draw_loc.x + margined_draw_loc.width,
                        margined_draw_loc.y + margined_draw_loc.height,
                    ));
                }
            }
        }
    }
}
