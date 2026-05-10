use std::cmp::{max, min};

use tui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Padding, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::App,
    canvas::{
        Painter,
        components::{
            scroll_bar::{ScrollBarArgs, dialog_scroll_bar_area, draw_scroll_bar},
            search_input,
        },
        drawing_utils::dialog_block,
    },
    constants::{self, HELP_TEXT},
};

/// Append a highlighted match to `lines`, and return whether a match was found.
fn add_highlight_match<'a>(
    query: &str, target: &'a str, lines: &mut Vec<Line<'a>>, normal_style: Style,
    match_style: Style,
) -> bool {
    let lower = target.to_lowercase();

    if let Some(pos) = lower.find(query) {
        let match_end = pos + query.len();
        lines.push(Line::from(vec![
            Span::styled(&target[..pos], normal_style),
            Span::styled(&target[pos..match_end], match_style),
            Span::styled(&target[match_end..], normal_style),
        ]));
        true
    } else {
        false
    }
}

// TODO: [REFACTOR] Make generic dialog boxes to build off of instead?
impl Painter {
    fn help_text_lines(&self, app_state: &App) -> Vec<Line<'_>> {
        let mut lines: Vec<Line<'_>> = Vec::new();

        let query = app_state
            .help_dialog_state
            .search_input_state
            .current_query()
            .trim()
            .to_lowercase();

        let is_filtering = !query.is_empty();
        if is_filtering {
            let selected_header_style = self
                .styles
                .selected_text_style
                .add_modifier(self.styles.table_header_style.add_modifier);

            HELP_TEXT.iter().enumerate().for_each(|(itx, section)| {
                let mut iter = section.iter();

                if itx == 0 {
                    return;
                }

                let header_str = iter.next().copied();

                let mut header_line: Vec<Line<'_>> = Vec::new();
                let header_matches = if let Some(h) = header_str {
                    add_highlight_match(
                        &query,
                        h,
                        &mut header_line,
                        self.styles.table_header_style,
                        selected_header_style,
                    )
                } else {
                    false
                };

                let mut matched_body: Vec<Line<'_>> = Vec::new();
                for &text in iter {
                    add_highlight_match(
                        &query,
                        text,
                        &mut matched_body,
                        self.styles.text_style,
                        self.styles.selected_text_style,
                    );
                }

                if !matched_body.is_empty() || header_matches {
                    if let Some(header) = header_str {
                        // Don't insert the space if there's nothing above anyway.
                        if !lines.is_empty() {
                            lines.push(Line::from(Span::default()));
                        }

                        if header_matches {
                            lines.extend(header_line);
                        } else {
                            lines.push(Line::from(Span::styled(
                                header,
                                self.styles.table_header_style,
                            )));
                        }
                    }

                    lines.extend(matched_body);
                }
            });
        } else {
            HELP_TEXT.iter().enumerate().for_each(|(itx, section)| {
                let mut iter = section.iter();

                if itx == 0 {
                    for &text in iter {
                        lines.push(Line::from(Span::styled(text, self.styles.text_style)));
                    }
                    return;
                }

                let header_opt = iter.next();
                let header_str = header_opt.copied();

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
            });
        }

        lines
    }

    pub fn draw_help_dialog(&self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect) {
        let styled_help_text = self.help_text_lines(app_state);

        // Reserve one column on the right for the scroll bar.
        let block = dialog_block(self.styles.border_type, self.styles.border_style)
            .title_top(Line::styled(" Help ", self.styles.widget_title_style))
            .title_top(
                Line::styled(" Esc to close ", self.styles.widget_title_style).right_aligned(),
            )
            .padding(Padding::right(1));

        let [content_area, input_area] = if app_state.help_dialog_state.is_searching() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .areas::<2>(draw_loc)
        } else {
            [draw_loc, Rect::default()]
        };

        if app_state.should_get_widget_bounds() {
            // We must also recalculate how many lines are wrapping to properly get
            // scrolling to work on small terminal sizes... oh joy.

            let inner = block.inner(content_area);
            app_state.help_dialog_state.height = inner.height;

            let mut overflow_buffer = 0;
            let paragraph_width: usize = max(inner.width, 1).into();
            let mut prev_section_len = 0;

            if app_state
                .help_dialog_state
                .search_input_state
                .current_query()
                .is_empty()
            {
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
                for line in &styled_help_text {
                    let width: usize = line
                        .spans
                        .iter()
                        .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                        .sum();

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

        f.render_widget(
            Paragraph::new(styled_help_text)
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

        if app_state.help_dialog_state.is_searching() {
            search_input::render_search_input(
                f,
                input_area,
                search_input::SearchInputState {
                    input_field_state: &app_state.help_dialog_state.search_input_state,
                    is_focused: true,
                    prefix: "Search: ",
                    hint: Some("Type to search, Esc to close"),
                },
                search_input::SearchInputStyles {
                    prefix_style: self.styles.widget_title_style,
                    text_style: self.styles.text_style,
                    cursor_style: self.styles.selected_text_style,
                    hint_style: self.styles.text_style.dim(),
                },
            );
        }

        let scrollbar_area = dialog_scroll_bar_area(draw_loc);
        let content_length = app_state
            .help_dialog_state
            .scroll_state
            .max_scroll_index
            .into();
        let viewport_length = app_state.help_dialog_state.height.into();
        let position = app_state
            .help_dialog_state
            .scroll_state
            .current_scroll_index
            .into();

        draw_scroll_bar(
            f,
            scrollbar_area,
            ScrollBarArgs {
                content_length,
                viewport_length,
                position,
                style: self.styles.text_style,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use tui::style::Style;

    use super::*;

    fn check_spans(line: &Line<'_>, expected: [&str; 3]) {
        assert_eq!(line.spans[0].content, expected[0]);
        assert_eq!(line.spans[1].content, expected[1]);
        assert_eq!(line.spans[2].content, expected[2]);
    }

    #[test]
    fn test_no_match() {
        let mut lines = Vec::new();
        let matched = add_highlight_match(
            "xyz",
            "hello world",
            &mut lines,
            Style::default(),
            Style::default(),
        );
        assert!(!matched);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_match_in_middle() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "world",
            "hello world!",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["hello ", "world", "!"]);
    }

    #[test]
    fn test_match_at_start() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "hello",
            "hello world",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["", "hello", " world"]);
    }

    #[test]
    fn test_match_at_end() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "world",
            "hello world",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["hello ", "world", ""]);
    }

    #[test]
    fn test_full_match() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "hello",
            "hello",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["", "hello", ""]);
    }

    #[test]
    fn test_case_insensitive() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "cpu",
            "CPU widget",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["", "CPU", " widget"]);
    }

    #[test]
    fn test_unicode_in_target() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "éléphants",
            "J'adore les éléphants et les lapins.",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["J'adore les ", "éléphants", " et les lapins."]);
    }

    #[test]
    fn test_unicode() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "你",
            "Hi 你好!🇨🇦",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["Hi ", "你", "好!🇨🇦"]);
    }

    #[test]
    fn test_unicode_2() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "🇨🇦",
            "Hi 你好!🇨🇦",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["Hi 你好!", "🇨🇦", ""]);
    }

    /// Shows that we only match the first occurrence in the line. This is expected behaviour as we're passing
    /// in separate strings.
    #[test]
    fn test_unicode_3() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "🇨🇦",
            "Hi 🇨🇦 你好!🇨🇦",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["Hi ", "🇨🇦", " 你好!🇨🇦"]);
    }

    #[test]
    fn test_unicode_case_insensitive() {
        let mut lines = Vec::new();
        assert!(add_highlight_match(
            "éléphants",
            "J'adore les Éléphants et les lapins.",
            &mut lines,
            Style::default(),
            Style::default()
        ));
        check_spans(&lines[0], ["J'adore les ", "Éléphants", " et les lapins."]);
    }

    #[test]
    fn test_accumulates_across_calls() {
        let mut lines = Vec::new();
        add_highlight_match("a", "cat", &mut lines, Style::default(), Style::default()); // match
        add_highlight_match("a", "dog", &mut lines, Style::default(), Style::default()); // no match
        add_highlight_match("a", "rat", &mut lines, Style::default(), Style::default()); // match
        assert_eq!(lines.len(), 2);
    }
}
