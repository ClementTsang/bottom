use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::{App, AppSearchState},
    canvas::{
        components::data_table::{DrawInfo, SelectionState},
        Painter,
    },
    constants::*,
};

const SORT_MENU_WIDTH: u16 = 7;

impl Painter {
    /// Draws and handles all process-related drawing.  Use this.
    /// - `widget_id` here represents the widget ID of the process widget
    ///   itself!
    pub fn draw_process(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(proc_widget_state) = app_state.states.proc_state.widget_states.get(&widget_id) {
            let search_height = if draw_border { 5 } else { 3 };
            let is_sort_open = proc_widget_state.is_sort_open;

            let mut proc_draw_loc = draw_loc;
            if proc_widget_state.is_search_enabled() {
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

                self.draw_sort_table(f, app_state, processes_chunk[0], widget_id + 2);
            }

            self.draw_processes_table(f, app_state, proc_draw_loc, widget_id);
        }

        if let Some(proc_widget_state) = app_state
            .states
            .proc_state
            .widget_states
            .get_mut(&widget_id)
        {
            // Reset redraw marker.
            if proc_widget_state.force_rerender {
                proc_widget_state.force_rerender = false;
            }
        }
    }

    /// Draws the process sort box.
    /// - `widget_id` represents the widget ID of the process widget itself.an
    fn draw_processes_table(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(proc_widget_state) = app_state
            .states
            .proc_state
            .widget_states
            .get_mut(&widget_id)
        {
            let recalculate_column_widths =
                should_get_widget_bounds || proc_widget_state.force_rerender;

            let is_on_widget = widget_id == app_state.current_widget.widget_id;

            let draw_info = DrawInfo {
                loc: draw_loc,
                force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                selection_state: SelectionState::new(app_state.is_expanded, is_on_widget),
            };

            proc_widget_state.table.draw(
                f,
                &draw_info,
                app_state.widget_map.get_mut(&widget_id),
                self,
            );
        }
    }

    /// Draws the process search field.
    /// - `widget_id` represents the widget ID of the search box itself --- NOT
    ///   the process widget state that is stored.
    fn draw_search_field(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        fn build_query_span(
            search_state: &AppSearchState, available_width: usize, is_on_widget: bool,
            currently_selected_text_style: Style, text_style: Style,
        ) -> Vec<Span<'_>> {
            let start_index = search_state.display_start_char_index;
            let cursor_index = search_state.grapheme_cursor.cur_cursor();
            let mut current_width = 0;
            let query = search_state.current_search_query.as_str();

            if is_on_widget {
                let mut res = Vec::with_capacity(available_width);
                for ((index, grapheme), lengths) in
                    UnicodeSegmentation::grapheme_indices(query, true)
                        .zip(search_state.size_mappings.values())
                {
                    if index < start_index {
                        continue;
                    } else if current_width > available_width {
                        break;
                    } else {
                        let styled = if index == cursor_index {
                            Span::styled(grapheme, currently_selected_text_style)
                        } else {
                            Span::styled(grapheme, text_style)
                        };

                        res.push(styled);
                        current_width += lengths.end - lengths.start;
                    }
                }

                if cursor_index == query.len() {
                    res.push(Span::styled(" ", currently_selected_text_style))
                }

                res
            } else {
                // This is easier - we just need to get a range of graphemes, rather than
                // dealing with possibly inserting a cursor (as none is shown!)

                vec![Span::styled(query.to_string(), text_style)]
            }
        }

        if let Some(proc_widget_state) = app_state
            .states
            .proc_state
            .widget_states
            .get_mut(&(widget_id - 1))
        {
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let num_columns = usize::from(draw_loc.width);
            const SEARCH_TITLE: &str = "> ";
            let offset = if draw_border { 4 } else { 2 }; // width of 3 removed for >_|
            let available_width = if num_columns > (offset + 3) {
                num_columns - offset
            } else {
                num_columns
            };

            proc_widget_state
                .proc_search
                .search_state
                .get_start_position(available_width, app_state.is_force_redraw);

            // TODO: [CURSOR] blinking cursor?
            let query_with_cursor = build_query_span(
                &proc_widget_state.proc_search.search_state,
                available_width,
                is_on_widget,
                self.colours.selected_text_style,
                self.colours.text_style,
            );

            let mut search_text = vec![Line::from({
                let mut search_vec = vec![Span::styled(
                    SEARCH_TITLE,
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
            let case_style = if !proc_widget_state.proc_search.is_ignoring_case {
                self.colours.selected_text_style
            } else {
                self.colours.text_style
            };

            let whole_word_style = if proc_widget_state.proc_search.is_searching_whole_word {
                self.colours.selected_text_style
            } else {
                self.colours.text_style
            };

            let regex_style = if proc_widget_state.proc_search.is_searching_with_regex {
                self.colours.selected_text_style
            } else {
                self.colours.text_style
            };

            // TODO: [MOUSE] Mouse support for these in search
            // TODO: [MOVEMENT] Movement support for these in search
            let (case, whole, regex) = {
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "macos")] {
                        ("Case(F1)", "Whole(F2)", "Regex(F3)")
                    } else {
                        ("Case(Alt+C)", "Whole(Alt+W)", "Regex(Alt+R)")
                    }
                }
            };
            let option_text = Line::from(vec![
                Span::styled(case, case_style),
                Span::raw("  "),
                Span::styled(whole, whole_word_style),
                Span::raw("  "),
                Span::styled(regex, regex_style),
            ]);

            search_text.push(Line::from(Span::styled(
                if let Some(err) = &proc_widget_state.proc_search.search_state.error_message {
                    err.as_str()
                } else {
                    ""
                },
                self.colours.invalid_query_style,
            )));
            search_text.push(option_text);

            let current_border_style =
                if proc_widget_state.proc_search.search_state.is_invalid_search {
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
                .horizontal_margin(u16::from(!(is_on_widget || draw_border)))
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
    /// - `widget_id` represents the widget ID of the sort box itself --- NOT
    ///   the process widget state that is stored.
    fn draw_sort_table(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(pws) = app_state
            .states
            .proc_state
            .widget_states
            .get_mut(&(widget_id - 2))
        {
            let recalculate_column_widths = should_get_widget_bounds || pws.force_rerender;

            let is_on_widget = widget_id == app_state.current_widget.widget_id;

            let draw_info = DrawInfo {
                loc: draw_loc,
                force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                selection_state: SelectionState::new(app_state.is_expanded, is_on_widget),
            };

            pws.sort_table.draw(
                f,
                &draw_info,
                app_state.widget_map.get_mut(&widget_id),
                self,
            );
        }
    }
}
