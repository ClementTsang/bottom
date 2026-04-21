use std::cmp::{max, min};

use tui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::App,
    canvas::{Painter, components::search_input, drawing_utils::dialog_block},
    constants::{self, HELP_TEXT},
};

// TODO: [REFACTOR] Make generic dialog boxes to build off of instead?
impl Painter {
    fn help_text_lines(&self, app_state: &App) -> Vec<Line<'_>> {
        let mut lines: Vec<Line<'_>> = Vec::new();

        let query = app_state
            .help_dialog_state
            .search_query
            .trim()
            .to_lowercase();
        let is_filtering = !query.is_empty();

        HELP_TEXT.iter().enumerate().for_each(|(itx, section)| {
            let mut iter = section.iter();

            // Section 0 (intro) - hide entirely when filtering; render fully otherwise
            if itx == 0 {
                if !is_filtering {
                    for &text in iter {
                        lines.push(Line::from(Span::styled(text, self.styles.text_style)));
                    }
                }
                return;
            }

            // Non-root sections: pull out header; render header only if section has matches in search
            let header_opt = iter.next();
            let header_str = header_opt.copied();

            if is_filtering {
                // Check if header matches the query
                let header_matches = header_str
                    .map(|h| h.to_lowercase().contains(&query))
                    .unwrap_or(false);

                // Collect matching body lines
                let mut matched_body: Vec<Line<'_>> = Vec::new();
                for &text in iter {
                    let lower = text.to_lowercase();
                    if let Some(pos) = lower.find(&query) {
                        let pre = &text[..pos];
                        let mat_end = pos + query.len();
                        let mat_str = &text[pos..mat_end];
                        let post = &text[mat_end..];
                        matched_body.push(Line::from(vec![
                            Span::styled(pre, self.styles.text_style),
                            Span::styled(mat_str, self.styles.table_header_style),
                            Span::styled(post, self.styles.text_style),
                        ]));
                    }
                }

                if !matched_body.is_empty() || header_matches {
                    // Spacer + header (same appearance as non-search)
                    if let Some(header) = header_str {
                        lines.push(Line::from(Span::default()));
                        if header_matches {
                            let lower = header.to_lowercase();
                            if let Some(pos) = lower.find(&query) {
                                let pre = &header[..pos];
                                let mat_end = pos + query.len();
                                let mat_str = &header[pos..mat_end];
                                let post = &header[mat_end..];
                                lines.push(Line::from(vec![
                                    Span::styled(pre, self.styles.table_header_style),
                                    Span::styled(mat_str, self.styles.text_style),
                                    Span::styled(post, self.styles.table_header_style),
                                ]));
                            }
                        } else {
                            lines.push(Line::from(Span::styled(
                                header,
                                self.styles.table_header_style,
                            )));
                        }
                    }
                    // Push matching body lines
                    lines.extend(matched_body);
                }
            } else {
                // Non-search: show header and all body lines
                if let Some(header) = header_str {
                    lines.push(Line::from(Span::default()));
                    lines.push(Line::from(Span::styled(
                        header,
                        self.styles.table_header_style,
                    )));
                }
                for &text in iter {
                    lines.push(Line::from(Span::styled(text, self.styles.text_style)));
                }
            }
        });

        lines
    }

    pub fn draw_help_dialog(&self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect) {
        let styled_help_text = self.help_text_lines(app_state);

        let block = dialog_block(self.styles.border_type, self.styles.border_style)
            .title_top(Line::styled(" Help ", self.styles.widget_title_style))
            .title_top(
                Line::styled(" Esc to close ", self.styles.widget_title_style).right_aligned(),
            );

        if app_state.should_get_widget_bounds() {
            // We must also recalculate how many lines are wrapping to properly get
            // scrolling to work on small terminal sizes... oh joy.

            // Split into content and input areas; use content area for height and scrolling math.
            let content_area = if app_state.help_dialog_state.is_searching {
                tui::layout::Layout::default()
                    .direction(tui::layout::Direction::Vertical)
                    .constraints([
                        tui::layout::Constraint::Min(1),
                        tui::layout::Constraint::Length(1),
                    ])
                    .areas::<2>(draw_loc)[0]
            } else {
                draw_loc
            };

            app_state.help_dialog_state.height = block.inner(content_area).height;

            let mut overflow_buffer = 0;
            let paragraph_width = max(draw_loc.width.saturating_sub(2), 1) as usize;
            let mut prev_section_len = 0;

            if app_state.help_dialog_state.search_query.is_empty() {
                constants::HELP_TEXT
                    .iter()
                    .enumerate()
                    .for_each(|(itx, section)| {
                        let mut buffer = 0;

                        section.iter().for_each(|text_line| {
                            buffer += UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16
                                / paragraph_width as u16;
                        });

                        if itx == 0 {
                            app_state.help_dialog_state.index_shortcuts[itx] = 0;
                        } else {
                            app_state.help_dialog_state.index_shortcuts[itx] =
                                app_state.help_dialog_state.index_shortcuts[itx - 1]
                                    + 1
                                    + prev_section_len;
                        }
                        prev_section_len = section.len() as u16 + buffer;
                        overflow_buffer += buffer;
                    });
            } else {
                // When filtering in search mode, calculate wrapping more accurately
                for line in &styled_help_text {
                    // Get the width by extracting text from all spans
                    let line_text = line
                        .spans
                        .iter()
                        .map(|s| s.content.as_ref())
                        .collect::<String>();
                    let width = UnicodeWidthStr::width(line_text.as_str());
                    if width > paragraph_width {
                        overflow_buffer += (width.saturating_sub(1) / paragraph_width) as u16;
                    }
                }
            }

            let max_scroll_index = &mut app_state.help_dialog_state.scroll_state.max_scroll_index;
            *max_scroll_index = (styled_help_text.len() as u16 + 3 + overflow_buffer)
                .saturating_sub(draw_loc.height + 1);

            // Fix the scroll index if it is over-scrolled
            let index = &mut app_state
                .help_dialog_state
                .scroll_state
                .current_scroll_index;

            *index = min(*index, *max_scroll_index);
        }

        // Split into content and input areas for rendering
        let content_area = if app_state.help_dialog_state.is_searching {
            tui::layout::Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    tui::layout::Constraint::Min(1),
                    tui::layout::Constraint::Length(1),
                ])
                .areas::<2>(draw_loc)[0]
        } else {
            draw_loc
        };

        // Render help content
        f.render_widget(
            Paragraph::new(styled_help_text.clone())
                .block(block)
                .style(self.styles.text_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .scroll((
                    app_state
                        .help_dialog_state
                        .scroll_state
                        .current_scroll_index,
                    0,
                )),
            content_area,
        );

        // Render input pane only when searching with visible cursor
        if app_state.help_dialog_state.is_searching {
            let input_area = tui::layout::Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    tui::layout::Constraint::Min(1),
                    tui::layout::Constraint::Length(1),
                ])
                .areas::<2>(draw_loc)[1];

            search_input::render_search_input(
                f,
                input_area,
                search_input::SearchInputConfig {
                    query: app_state.help_dialog_state.search_query.as_str(),
                    cursor_index: app_state.help_dialog_state.search_cursor_index,
                    is_focused: true,
                    prefix: "Search: ",
                    hint: Some("Type to search, Esc to clear"),
                },
                search_input::SearchInputStyles {
                    prefix_style: self.styles.widget_title_style,
                    query_style: self.styles.text_style,
                    cursor_style: self.styles.selected_text_style,
                    hint_style: self.styles.table_header_style,
                },
            );
        }
    }
}
