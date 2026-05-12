use tui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::utils::input::InputFieldState;

pub struct SearchInputState<'a> {
    pub input_field_state: &'a InputFieldState,
    pub is_focused: bool,
    pub prefix: &'a str,
    pub hint: Option<&'a str>,
}

pub struct SearchInputStyles {
    pub prefix_style: Style,
    pub text_style: Style,
    pub cursor_style: Style,
    pub hint_style: Style,
}

/// Build a query span from a [`InputFieldState`]. `available_width` is the terminal column width.
pub fn build_query_spans(
    input_field_state: &InputFieldState, available_width: usize, is_on_widget: bool,
    cursor_style: Style, text_style: Style,
) -> Vec<Span<'_>> {
    let query = input_field_state.current_query();

    if !is_on_widget {
        // This is easier - we just need to get a range of graphemes, rather than
        // dealing with possibly inserting a cursor (as none is shown!)
        return vec![Span::styled(query.to_string(), text_style)];
    }

    let start_index = input_field_state.display_start_index();
    let cursor_index = input_field_state.cursor_index();
    let mut current_width = 0;
    let mut res = Vec::with_capacity(available_width);

    for (index, grapheme, lengths) in input_field_state.graphemes_with_ranges() {
        if index < start_index {
            continue;
        } else if current_width > available_width {
            break;
        } else {
            let styled = if index == cursor_index {
                Span::styled(grapheme, cursor_style)
            } else {
                Span::styled(grapheme, text_style)
            };

            res.push(styled);

            // The lengths are the width of the graphemes terminal-wise.
            current_width += lengths.end - lengths.start;
        }
    }

    if cursor_index == query.len() {
        res.push(Span::styled(" ", cursor_style))
    }

    res
}

pub fn render_search_input(
    f: &mut Frame<'_>, rect: Rect, state: SearchInputState<'_>, styles: SearchInputStyles,
) {
    let mut input_vec = vec![Span::styled(state.prefix, styles.prefix_style)];
    let query_spans = build_query_spans(
        state.input_field_state,
        rect.width.into(),
        state.is_focused,
        styles.cursor_style,
        styles.text_style,
    );

    input_vec.extend(query_spans);

    if let Some(hint) = state.hint {
        if state.input_field_state.current_query().is_empty() {
            input_vec.push(Span::styled(" ", styles.text_style));
            input_vec.push(Span::styled(hint, styles.hint_style));
        }
    }

    let input_line = Line::from(input_vec);

    f.render_widget(
        Paragraph::new(input_line)
            .style(styles.text_style)
            .alignment(Alignment::Left),
        rect,
    );
}
