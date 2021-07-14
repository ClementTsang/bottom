use crate::{
    app::AppState,
    canvas::{
        drawing_utils::{get_column_widths, get_search_start_position, get_start_position},
        Painter,
    },
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

use once_cell::sync::Lazy;

static PROCESS_HEADERS_HARD_WIDTH_NO_GROUP: Lazy<Vec<Option<u16>>> = Lazy::new(|| {
    vec![
        Some(7),
        None,
        Some(8),
        Some(8),
        Some(8),
        Some(8),
        Some(7),
        Some(8),
        #[cfg(target_family = "unix")]
        None,
        None,
    ]
});
static PROCESS_HEADERS_HARD_WIDTH_GROUPED: Lazy<Vec<Option<u16>>> = Lazy::new(|| {
    vec![
        Some(7),
        None,
        Some(8),
        Some(8),
        Some(8),
        Some(8),
        Some(7),
        Some(8),
    ]
});

static PROCESS_HEADERS_SOFT_WIDTH_MAX_GROUPED_COMMAND: Lazy<Vec<Option<f64>>> =
    Lazy::new(|| vec![None, Some(0.7), None, None, None, None, None, None]);
static PROCESS_HEADERS_SOFT_WIDTH_MAX_GROUPED_ELSE: Lazy<Vec<Option<f64>>> =
    Lazy::new(|| vec![None, Some(0.3), None, None, None, None, None, None]);

static PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_COMMAND: Lazy<Vec<Option<f64>>> = Lazy::new(|| {
    vec![
        None,
        Some(0.7),
        None,
        None,
        None,
        None,
        None,
        None,
        #[cfg(target_family = "unix")]
        Some(0.05),
        Some(0.2),
    ]
});
static PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_TREE: Lazy<Vec<Option<f64>>> = Lazy::new(|| {
    vec![
        None,
        Some(0.5),
        None,
        None,
        None,
        None,
        None,
        None,
        #[cfg(target_family = "unix")]
        Some(0.05),
        Some(0.2),
    ]
});
static PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_ELSE: Lazy<Vec<Option<f64>>> = Lazy::new(|| {
    vec![
        None,
        Some(0.3),
        None,
        None,
        None,
        None,
        None,
        None,
        #[cfg(target_family = "unix")]
        Some(0.05),
        Some(0.2),
    ]
});

pub trait ProcessTableWidget {
    /// Draws and handles all process-related drawing.  Use this.
    /// - `widget_id` here represents the widget ID of the process widget itself!
    fn draw_process_features<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the process widget itself.
    ///
    /// This should not be directly called.
    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    /// Draws the process search field.
    /// - `widget_id` represents the widget ID of the search box itself --- NOT the process widget
    /// state that is stored.
    ///
    /// This should not be directly called.
    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the sort box itself --- NOT the process widget
    /// state that is stored.
    ///
    /// This should not be directly called.
    fn draw_process_sort<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl ProcessTableWidget for Painter {
    fn draw_process_features<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
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
                    .constraints([Constraint::Min(0), Constraint::Length(search_height)])
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
                    .constraints([Constraint::Length(header_len + 4), Constraint::Min(0)])
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
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(proc_widget_state) = app_state.proc_state.widget_states.get_mut(&widget_id) {
            let recalculate_column_widths =
                should_get_widget_bounds || proc_widget_state.requires_redraw;
            if proc_widget_state.requires_redraw {
                proc_widget_state.requires_redraw = false;
            }

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

            let (border_style, highlight_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };

            let title_base = if app_state.app_config_fields.show_table_scroll_position {
                if let Some(finalized_process_data) = app_state
                    .canvas_data
                    .finalized_process_data_map
                    .get(&widget_id)
                {
                    let title = format!(
                        " Processes ({} of {}) ",
                        proc_widget_state
                            .scroll_state
                            .current_scroll_position
                            .saturating_add(1),
                        finalized_process_data.len()
                    );

                    if title.len() <= draw_loc.width as usize {
                        title
                    } else {
                        " Processes ".to_string()
                    }
                } else {
                    " Processes ".to_string()
                }
            } else {
                " Processes ".to_string()
            };

            let title = if app_state.is_expanded
                && !proc_widget_state
                    .process_search_state
                    .search_state
                    .is_enabled
                && !proc_widget_state.is_sort_open
            {
                const ESCAPE_ENDING: &str = "── Esc to go back ";

                let (chosen_title_base, expanded_title_base) = {
                    let temp_title_base = format!("{}{}", title_base, ESCAPE_ENDING);

                    if temp_title_base.len() > draw_loc.width as usize {
                        (
                            " Processes ".to_string(),
                            format!("{}{}", " Processes ".to_string(), ESCAPE_ENDING),
                        )
                    } else {
                        (title_base, temp_title_base)
                    }
                };

                Spans::from(vec![
                    Span::styled(chosen_title_base, self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to go back ",
                            "─".repeat(
                                usize::from(draw_loc.width).saturating_sub(
                                    UnicodeSegmentation::graphemes(
                                        expanded_title_base.as_str(),
                                        true
                                    )
                                    .count()
                                        + 2
                                )
                            )
                        ),
                        border_style,
                    ),
                ])
            } else {
                Spans::from(Span::styled(title_base, self.colours.widget_title_style))
            };

            let process_block = if draw_border {
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            } else if is_on_widget {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            if let Some(process_data) = &app_state
                .canvas_data
                .stringified_process_data_map
                .get(&widget_id)
            {
                let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                    0
                } else {
                    app_state.app_config_fields.table_gap
                };
                let position = get_start_position(
                    usize::from(
                        (draw_loc.height + (1 - table_gap))
                            .saturating_sub(self.table_height_offset),
                    ),
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
                let processed_sliced_vec = sliced_vec.iter().map(|(data, disabled)| {
                    (
                        data.iter()
                            .map(|(entry, _alternative)| entry)
                            .collect::<Vec<_>>(),
                        disabled,
                    )
                });

                let proc_table_state = &mut proc_widget_state.scroll_state.table_state;
                proc_table_state.select(Some(
                    proc_widget_state
                        .scroll_state
                        .current_scroll_position
                        .saturating_sub(start_position),
                ));

                // Draw!
                let process_headers = proc_widget_state.columns.get_column_headers(
                    &proc_widget_state.process_sorting_type,
                    proc_widget_state.is_process_sort_descending,
                );

                // Calculate widths
                // FIXME: See if we can move this into the recalculate block?  I want to move column widths into the column widths
                let hard_widths = if proc_widget_state.is_grouped {
                    &*PROCESS_HEADERS_HARD_WIDTH_GROUPED
                } else {
                    &*PROCESS_HEADERS_HARD_WIDTH_NO_GROUP
                };

                if recalculate_column_widths {
                    let mut column_widths = process_headers
                        .iter()
                        .map(|entry| UnicodeWidthStr::width(entry.as_str()) as u16)
                        .collect::<Vec<_>>();

                    let soft_widths_min = column_widths
                        .iter()
                        .map(|width| Some(*width))
                        .collect::<Vec<_>>();

                    proc_widget_state.table_width_state.desired_column_widths = {
                        for (row, _disabled) in processed_sliced_vec.clone() {
                            for (col, entry) in row.iter().enumerate() {
                                if let Some(col_width) = column_widths.get_mut(col) {
                                    let grapheme_len = UnicodeWidthStr::width(entry.as_str());
                                    if grapheme_len as u16 > *col_width {
                                        *col_width = grapheme_len as u16;
                                    }
                                }
                            }
                        }
                        column_widths
                    };

                    proc_widget_state.table_width_state.desired_column_widths = proc_widget_state
                        .table_width_state
                        .desired_column_widths
                        .iter()
                        .zip(hard_widths)
                        .map(|(current, hard)| {
                            if let Some(hard) = hard {
                                if *hard > *current {
                                    *hard
                                } else {
                                    *current
                                }
                            } else {
                                *current
                            }
                        })
                        .collect::<Vec<_>>();

                    let soft_widths_max = if proc_widget_state.is_grouped {
                        // Note grouped trees are not a thing.

                        if proc_widget_state.is_using_command {
                            &*PROCESS_HEADERS_SOFT_WIDTH_MAX_GROUPED_COMMAND
                        } else {
                            &*PROCESS_HEADERS_SOFT_WIDTH_MAX_GROUPED_ELSE
                        }
                    } else if proc_widget_state.is_using_command {
                        &*PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_COMMAND
                    } else if proc_widget_state.is_tree_mode {
                        &*PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_TREE
                    } else {
                        &*PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_ELSE
                    };

                    proc_widget_state.table_width_state.calculated_column_widths =
                        get_column_widths(
                            draw_loc.width,
                            &hard_widths,
                            &soft_widths_min,
                            soft_widths_max,
                            &(proc_widget_state
                                .table_width_state
                                .desired_column_widths
                                .iter()
                                .map(|width| Some(*width))
                                .collect::<Vec<_>>()),
                            true,
                        );

                    // debug!(
                    //     "DCW: {:?}",
                    //     proc_widget_state.table_width_state.desired_column_widths
                    // );
                    // debug!(
                    //     "CCW: {:?}",
                    //     proc_widget_state.table_width_state.calculated_column_widths
                    // );
                }

                let dcw = &proc_widget_state.table_width_state.desired_column_widths;
                let ccw = &proc_widget_state.table_width_state.calculated_column_widths;

                let process_rows = sliced_vec.iter().map(|(data, disabled)| {
                    let truncated_data = data.iter().zip(hard_widths).enumerate().map(
                        |(itx, ((entry, alternative), width))| {
                            if let (Some(desired_col_width), Some(calculated_col_width)) =
                                (dcw.get(itx), ccw.get(itx))
                            {
                                if width.is_none() {
                                    if *desired_col_width > *calculated_col_width
                                        && *calculated_col_width > 0
                                    {
                                        let graphemes =
                                            UnicodeSegmentation::graphemes(entry.as_str(), true)
                                                .collect::<Vec<&str>>();

                                        if let Some(alternative) = alternative {
                                            Text::raw(alternative)
                                        } else if graphemes.len() > *calculated_col_width as usize
                                            && *calculated_col_width > 1
                                        {
                                            // Truncate with ellipsis
                                            let first_n = graphemes
                                                [..(*calculated_col_width as usize - 1)]
                                                .concat();
                                            Text::raw(format!("{}…", first_n))
                                        } else {
                                            Text::raw(entry)
                                        }
                                    } else {
                                        Text::raw(entry)
                                    }
                                } else {
                                    Text::raw(entry)
                                }
                            } else {
                                Text::raw(entry)
                            }
                        },
                    );

                    if *disabled {
                        Row::new(truncated_data).style(self.colours.disabled_text_style)
                    } else {
                        Row::new(truncated_data)
                    }
                });

                f.render_stateful_widget(
                    Table::new(process_rows)
                        .header(
                            Row::new(process_headers)
                                .style(self.colours.table_header_style)
                                .bottom_margin(table_gap),
                        )
                        .block(process_block)
                        .highlight_style(highlight_style)
                        .style(self.colours.text_style)
                        .widths(
                            &(proc_widget_state
                                .table_width_state
                                .calculated_column_widths
                                .iter()
                                .map(|calculated_width| {
                                    Constraint::Length(*calculated_width as u16)
                                })
                                .collect::<Vec<_>>()),
                        ),
                    margined_draw_loc,
                    proc_table_state,
                );
            } else {
                f.render_widget(process_block, margined_draw_loc);
            }

            // Check if we need to update columnar bounds...
            if recalculate_column_widths
                || proc_widget_state.columns.column_header_x_locs.is_none()
                || proc_widget_state.columns.column_header_y_loc.is_none()
            {
                // y location is just the y location of the widget + border size (1 normally, 0 in basic)
                proc_widget_state.columns.column_header_y_loc =
                    Some(draw_loc.y + if draw_border { 1 } else { 0 });

                // x location is determined using the x locations of the widget; just offset from the left bound
                // as appropriate, and use the right bound as limiter.

                let mut current_x_left = draw_loc.x + 1;
                let max_x_right = draw_loc.x + draw_loc.width - 1;

                let mut x_locs = vec![];

                for width in proc_widget_state
                    .table_width_state
                    .calculated_column_widths
                    .iter()
                {
                    let right_bound = current_x_left + width;

                    if right_bound < max_x_right {
                        x_locs.push((current_x_left, right_bound));
                        current_x_left = right_bound + 1;
                    } else {
                        x_locs.push((current_x_left, max_x_right));
                        break;
                    }
                }

                proc_widget_state.columns.column_header_x_locs = Some(x_locs);
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
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        fn build_query<'a>(
            is_on_widget: bool, grapheme_indices: GraphemeIndices<'a>, start_position: usize,
            cursor_position: usize, query: &str, currently_selected_text_style: tui::style::Style,
            text_style: tui::style::Style,
        ) -> Vec<Span<'a>> {
            let mut current_grapheme_posn = 0;

            if is_on_widget {
                let mut res = grapheme_indices
                    .filter_map(|grapheme| {
                        current_grapheme_posn += UnicodeWidthStr::width(grapheme.1);

                        if current_grapheme_posn <= start_position {
                            None
                        } else {
                            let styled = if grapheme.0 == cursor_position {
                                Span::styled(grapheme.1, currently_selected_text_style)
                            } else {
                                Span::styled(grapheme.1, text_style)
                            };
                            Some(styled)
                        }
                    })
                    .collect::<Vec<_>>();

                if cursor_position == query.len() {
                    res.push(Span::styled(" ", currently_selected_text_style))
                }

                res
            } else {
                // This is easier - we just need to get a range of graphemes, rather than
                // dealing with possibly inserting a cursor (as none is shown!)

                vec![Span::styled(query.to_string(), text_style)]
            }
        }

        // TODO: Make the cursor scroll back if there's space!
        if let Some(proc_widget_state) =
            app_state.proc_state.widget_states.get_mut(&(widget_id - 1))
        {
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let num_columns = usize::from(draw_loc.width);
            let search_title = "> ";

            let num_chars_for_text = search_title.len();
            let cursor_position = proc_widget_state.get_search_cursor_position();
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

            // TODO: [CURSOR] blank cursor if not selected
            // TODO: [CURSOR] blinking cursor?
            let query_with_cursor = build_query(
                is_on_widget,
                grapheme_indices,
                start_position,
                cursor_position,
                query,
                self.colours.currently_selected_text_style,
                self.colours.text_style,
            );

            let mut search_text = vec![Spans::from({
                let mut search_vec = vec![Span::styled(
                    search_title,
                    if is_on_widget {
                        self.colours.table_header_style
                    } else {
                        self.colours.text_style
                    },
                )];
                search_vec.extend(query_with_cursor);

                search_vec
            })];

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

            // FIXME: [MOUSE] Mouse support for these in search
            // FIXME: [MOVEMENT] Movement support for these in search
            let option_text = Spans::from(vec![
                Span::styled(
                    format!("Case({})", if self.is_mac_os { "F1" } else { "Alt+C" }),
                    case_style,
                ),
                Span::raw("  "),
                Span::styled(
                    format!("Whole({})", if self.is_mac_os { "F2" } else { "Alt+W" }),
                    whole_word_style,
                ),
                Span::raw("  "),
                Span::styled(
                    format!("Regex({})", if self.is_mac_os { "F3" } else { "Alt+R" }),
                    regex_style,
                ),
            ]);

            search_text.push(Spans::from(Span::styled(
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
            )));
            search_text.push(option_text);

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

            let title = Span::styled(
                if draw_border {
                    const TITLE_BASE: &str = " Esc to close ";
                    let repeat_num =
                        usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2);
                    format!("{} Esc to close ", "─".repeat(repeat_num))
                } else {
                    String::new()
                },
                current_border_style,
            );

            let process_search_block = if draw_border {
                Block::default()
                    .title(title)
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
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

            f.render_widget(
                Paragraph::new(search_text)
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
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
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
                .map(|column_type| column_type.to_string())
                .collect::<Vec<_>>();

            let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                app_state.app_config_fields.table_gap
            };
            let position = get_start_position(
                usize::from(
                    (draw_loc.height + (1 - table_gap)).saturating_sub(self.table_height_offset),
                ),
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
                .map(|column| Row::new(vec![column.as_str()]));

            let column_state = &mut proc_widget_state.columns.column_state;
            column_state.select(Some(
                proc_widget_state
                    .columns
                    .current_scroll_position
                    .saturating_sub(start_position),
            ));
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

            let highlight_style = if is_on_widget {
                self.colours.currently_selected_text_style
            } else {
                self.colours.text_style
            };

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

            f.render_stateful_widget(
                Table::new(sort_options)
                    .header(
                        Row::new(vec!["Sort By"])
                            .style(self.colours.table_header_style)
                            .bottom_margin(table_gap),
                    )
                    .block(process_sort_block)
                    .highlight_style(highlight_style)
                    .style(self.colours.text_style)
                    .widths(&[Constraint::Percentage(100)]),
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
