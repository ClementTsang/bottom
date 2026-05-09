use tui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

pub struct SearchInputConfig<'a> {
    pub query: &'a str,
    pub cursor_index: usize,
    pub is_focused: bool,
    pub prefix: &'a str,
    pub hint: Option<&'a str>,
}

pub struct SearchInputStyles {
    pub prefix_style: Style,
    pub query_style: Style,
    pub cursor_style: Style,
    pub hint_style: Style,
}

fn build_query_spans(
    query: &str, cursor_index: usize, is_focused: bool, query_style: Style, cursor_style: Style,
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
        // TODO: There's a bug right now if you move the cursor, this shifts because of the extra space at the end.
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
