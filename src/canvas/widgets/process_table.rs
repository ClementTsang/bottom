use crate::{
    app::App,
    canvas::{
        components::{TextTable, TextTableTitle},
        drawing_utils::get_search_start_position,
        Painter,
    },
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

const PROCESS_HEADERS_HARD_WIDTH_NO_GROUP: &[Option<u16>] = &[
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
];
const PROCESS_HEADERS_HARD_WIDTH_GROUPED: &[Option<u16>] = &[
    Some(7),
    None,
    Some(8),
    Some(8),
    Some(8),
    Some(8),
    Some(7),
    Some(8),
];

const PROCESS_HEADERS_SOFT_WIDTH_MAX_GROUPED_COMMAND: &[Option<f64>] =
    &[None, Some(0.7), None, None, None, None, None, None];
const PROCESS_HEADERS_SOFT_WIDTH_MAX_GROUPED_ELSE: &[Option<f64>] =
    &[None, Some(0.3), None, None, None, None, None, None];

const PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_COMMAND: &[Option<f64>] = &[
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
];
const PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_TREE: &[Option<f64>] = &[
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
];
const PROCESS_HEADERS_SOFT_WIDTH_MAX_NO_GROUP_ELSE: &[Option<f64>] = &[
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
];

impl Painter {
    /// Draws and handles all process-related drawing.  Use this.
    /// - `widget_id` here represents the widget ID of the process widget itself!
    pub fn draw_process_widget<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(process_widget_state) = app_state.proc_state.widget_states.get(&widget_id) {
            let search_height = if draw_border { 5 } else { 3 };
            let is_sort_open = process_widget_state.is_sort_open;
            const SORT_MENU_WIDTH: u16 = 8;

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
                    .constraints([Constraint::Length(SORT_MENU_WIDTH + 4), Constraint::Min(0)])
                    .split(proc_draw_loc);
                proc_draw_loc = processes_chunk[1];

                self.draw_sort_table(f, app_state, processes_chunk[0], draw_border, widget_id + 2);
            }

            self.draw_processes_table(f, app_state, proc_draw_loc, draw_border, widget_id);
        }
    }

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the process widget itself.
    ///
    /// This should not be directly called.
    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(proc_widget_state) = app_state.proc_state.widget_states.get_mut(&widget_id) {
            let recalculate_column_widths =
                should_get_widget_bounds || proc_widget_state.requires_redraw;

            // Reset redraw marker.
            // TODO: this should ideally be handled generically in the future.
            if proc_widget_state.requires_redraw {
                proc_widget_state.requires_redraw = false;
            }

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let (border_style, highlighted_text_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };

            TextTable {
                table_gap: app_state.app_config_fields.table_gap,
                is_force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                header_style: self.colours.table_header_style,
                border_style,
                highlighted_text_style,
                title: Some(TextTableTitle {
                    title: " Processes ".into(),
                    is_expanded: app_state.is_expanded,
                }),
                is_on_widget,
                draw_border,
                show_table_scroll_position: app_state.app_config_fields.show_table_scroll_position,
                title_style: self.colours.widget_title_style,
                text_style: self.colours.text_style,
                left_to_right: false,
            }
            .draw_text_table(f, draw_loc, &mut proc_widget_state.table_state, todo!());

            // FIXME: [Proc] Handle this, and the above TODO
            // // Check if we need to update columnar bounds...
            // if recalculate_column_widths
            //     || proc_widget_state.columns.column_header_x_locs.is_none()
            //     || proc_widget_state.columns.column_header_y_loc.is_none()
            // {
            //     // y location is just the y location of the widget + border size (1 normally, 0 in basic)
            //     proc_widget_state.columns.column_header_y_loc =
            //         Some(draw_loc.y + if draw_border { 1 } else { 0 });

            //     // x location is determined using the x locations of the widget; just offset from the left bound
            //     // as appropriate, and use the right bound as limiter.

            //     let mut current_x_left = draw_loc.x + 1;
            //     let max_x_right = draw_loc.x + draw_loc.width - 1;

            //     let mut x_locs = vec![];

            //     for width in proc_widget_state
            //         .table_width_state
            //         .calculated_column_widths
            //         .iter()
            //     {
            //         let right_bound = current_x_left + width;

            //         if right_bound < max_x_right {
            //             x_locs.push((current_x_left, right_bound));
            //             current_x_left = right_bound + 1;
            //         } else {
            //             x_locs.push((current_x_left, max_x_right));
            //             break;
            //         }
            //     }

            //     proc_widget_state.columns.column_header_x_locs = Some(x_locs);
            // }

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                    widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                    widget.bottom_right_corner =
                        Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
                }
            }
        }
    }

    /// Draws the process search field.
    /// - `widget_id` represents the widget ID of the search box itself --- NOT the process widget
    /// state that is stored.
    ///
    /// This should not be directly called.
    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
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
                &proc_widget_state.search_state.search_state.cursor_direction,
                &mut proc_widget_state.search_state.search_state.cursor_bar,
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
            let case_style = if !proc_widget_state.search_state.is_ignoring_case {
                self.colours.currently_selected_text_style
            } else {
                self.colours.text_style
            };

            let whole_word_style = if proc_widget_state.search_state.is_searching_whole_word {
                self.colours.currently_selected_text_style
            } else {
                self.colours.text_style
            };

            let regex_style = if proc_widget_state.search_state.is_searching_with_regex {
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
                if let Some(err) = &proc_widget_state.search_state.search_state.error_message {
                    err.as_str()
                } else {
                    ""
                },
                self.colours.invalid_query_style,
            )));
            search_text.push(option_text);

            let current_border_style = if proc_widget_state
                .search_state
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
                    .borders(SIDE_BORDERS)
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

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the sort box itself --- NOT the process widget
    /// state that is stored.
    ///
    /// This should not be directly called.
    fn draw_sort_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        // FIXME: [Proc] Redo drawing sort table!
        // let is_on_widget = widget_id == app_state.current_widget.widget_id;

        // if let Some(proc_widget_state) =
        //     app_state.proc_state.widget_states.get_mut(&(widget_id - 2))
        // {
        //     let current_scroll_position = proc_widget_state.columns.current_scroll_position;
        //     let sort_string = proc_widget_state
        //         .columns
        //         .ordered_columns
        //         .iter()
        //         .filter(|column_type| {
        //             proc_widget_state
        //                 .columns
        //                 .column_mapping
        //                 .get(column_type)
        //                 .unwrap()
        //                 .enabled
        //         })
        //         .map(|column_type| column_type.to_string())
        //         .collect::<Vec<_>>();

        //     let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
        //         0
        //     } else {
        //         app_state.app_config_fields.table_gap
        //     };
        //     let position = get_start_position(
        //         usize::from(
        //             (draw_loc.height + (1 - table_gap)).saturating_sub(self.table_height_offset),
        //         ),
        //         &proc_widget_state.columns.scroll_direction,
        //         &mut proc_widget_state.columns.previous_scroll_position,
        //         current_scroll_position,
        //         app_state.is_force_redraw,
        //     );

        //     // Sanity check
        //     let start_position = if position >= sort_string.len() {
        //         sort_string.len().saturating_sub(1)
        //     } else {
        //         position
        //     };

        //     let sliced_vec = &sort_string[start_position..];

        //     let sort_options = sliced_vec
        //         .iter()
        //         .map(|column| Row::new(vec![column.as_str()]));

        //     let column_state = &mut proc_widget_state.columns.column_state;
        //     column_state.select(Some(
        //         proc_widget_state
        //             .columns
        //             .current_scroll_position
        //             .saturating_sub(start_position),
        //     ));
        //     let current_border_style = if proc_widget_state
        //         .search_state
        //         .search_state
        //         .is_invalid_search
        //     {
        //         self.colours.invalid_query_style
        //     } else if is_on_widget {
        //         self.colours.highlighted_border_style
        //     } else {
        //         self.colours.border_style
        //     };

        //     let process_sort_block = if draw_border {
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_style(current_border_style)
        //     } else if is_on_widget {
        //         Block::default()
        //             .borders(SIDE_BORDERS)
        //             .border_style(current_border_style)
        //     } else {
        //         Block::default().borders(Borders::NONE)
        //     };

        //     let highlight_style = if is_on_widget {
        //         self.colours.currently_selected_text_style
        //     } else {
        //         self.colours.text_style
        //     };

        //     let margined_draw_loc = Layout::default()
        //         .constraints([Constraint::Percentage(100)])
        //         .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
        //         .direction(Direction::Horizontal)
        //         .split(draw_loc)[0];

        //     f.render_stateful_widget(
        //         Table::new(sort_options)
        //             .header(
        //                 Row::new(vec!["Sort By"])
        //                     .style(self.colours.table_header_style)
        //                     .bottom_margin(table_gap),
        //             )
        //             .block(process_sort_block)
        //             .highlight_style(highlight_style)
        //             .style(self.colours.text_style)
        //             .widths(&[Constraint::Percentage(100)]),
        //         margined_draw_loc,
        //         column_state,
        //     );

        //     if app_state.should_get_widget_bounds() {
        //         // Update draw loc in widget map
        //         if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
        //             widget.top_left_corner = Some((margined_draw_loc.x, margined_draw_loc.y));
        //             widget.bottom_right_corner = Some((
        //                 margined_draw_loc.x + margined_draw_loc.width,
        //                 margined_draw_loc.y + margined_draw_loc.height,
        //             ));
        //         }
        //     }
        // }
    }
}
