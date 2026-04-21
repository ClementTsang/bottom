use tui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

pub struct SearchInputConfig<'a> {
    pub query: &'a str,
    pub cursor_index: usize,
    pub is_focused: bool,
    pub prefix: &'a str,
    pub hint: Option<&'a str>,
}

pub struct SearchInputStyles {
    pub prefix_style: tui::style::Style,
    pub query_style: tui::style::Style,
    pub cursor_style: tui::style::Style,
    pub hint_style: tui::style::Style,
}

pub fn build_query_spans(
    query: &str, cursor_index: usize, is_focused: bool, query_style: tui::style::Style,
    cursor_style: tui::style::Style,
) -> Vec<Span<'static>> {
    if !is_focused {
        return vec![Span::styled(query.to_string(), query_style)];
    }

    let mut spans = Vec::new();
    let cursor_pos = cursor_index.min(query.len());

    for (idx, ch) in query.chars().enumerate() {
        let span = if idx == cursor_pos {
            Span::styled(ch.to_string(), cursor_style)
        } else {
            Span::styled(ch.to_string(), query_style)
        };
        spans.push(span);
    }

    // If cursor is at end of query, show cursor as space with cursor style
    if cursor_pos == query.len() {
        spans.push(Span::styled(" ", cursor_style));
    }

    spans
}

pub fn build_grapheme_query_spans(
    graphemes: &[(usize, &str, std::ops::Range<usize>)], cursor_index: usize,
    display_start_index: usize, is_focused: bool, available_width: usize,
    query_style: tui::style::Style, cursor_style: tui::style::Style,
) -> Vec<Span<'static>> {
    if !is_focused {
        let query_str = graphemes
            .iter()
            .map(|(_, g, _)| *g)
            .collect::<Vec<_>>()
            .join("");
        return vec![Span::styled(query_str, query_style)];
    }

    let mut spans = Vec::new();
    let mut current_width = 0;

    for (index, grapheme, lengths) in graphemes {
        if *index < display_start_index {
            continue;
        }
        if current_width > available_width {
            break;
        }

        let styled = if *index == cursor_index {
            Span::styled(grapheme.to_string(), cursor_style)
        } else {
            Span::styled(grapheme.to_string(), query_style)
        };
        spans.push(styled);
        current_width += lengths.end - lengths.start;
    }

    if cursor_index == graphemes.len() {
        spans.push(Span::styled(" ", cursor_style));
    }

    spans
}

pub fn render_search_input(
    f: &mut Frame<'_>, rect: Rect, config: SearchInputConfig<'_>, styles: SearchInputStyles,
) {
    let mut input_vec = vec![Span::styled(config.prefix, styles.prefix_style)];

    let query_spans = build_query_spans(
        config.query,
        config.cursor_index,
        config.is_focused,
        styles.query_style,
        styles.cursor_style,
    );
    input_vec.extend(query_spans);

    if let Some(hint) = config.hint {
        input_vec.push(Span::styled("  ", styles.query_style));
        input_vec.push(Span::styled(hint, styles.hint_style));
    }

    let input_line = Line::from(input_vec);

    f.render_widget(
        Paragraph::new(input_line)
            .style(styles.query_style)
            .alignment(tui::layout::Alignment::Left),
        rect,
    );
}
