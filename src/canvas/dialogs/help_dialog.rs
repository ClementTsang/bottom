use std::cmp::{max, min};

use tui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::App,
    canvas::Painter,
    constants::{self, HELP_TEXT},
};

const HELP_BASE: &str = " Help ── Esc to close ";

// TODO: [REFACTOR] Make generic dialog boxes to build off of instead?
impl Painter {
    fn help_text_lines(&self) -> Vec<Line<'_>> {
        let mut styled_help_spans = Vec::new();

        // Init help text:
        HELP_TEXT.iter().enumerate().for_each(|(itx, section)| {
            let mut section = section.iter();

            if itx > 0 {
                if let Some(header) = section.next() {
                    styled_help_spans.push(Span::default());
                    styled_help_spans.push(Span::styled(*header, self.colours.table_header_style));
                }
            }

            section.for_each(|&text| {
                styled_help_spans.push(Span::styled(text, self.colours.text_style))
            });
        });

        styled_help_spans.into_iter().map(Line::from).collect()
    }

    pub fn draw_help_dialog(&self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect) {
        let styled_help_text = self.help_text_lines();

        let help_title = Line::from(vec![
            Span::styled(" Help ", self.colours.widget_title_style),
            Span::styled(
                format!(
                    "─{}─ Esc to close ",
                    "─".repeat(
                        usize::from(draw_loc.width).saturating_sub(HELP_BASE.chars().count() + 2)
                    )
                ),
                self.colours.border_style,
            ),
        ]);

        let block = Block::default()
            .title(help_title)
            .style(self.colours.border_style)
            .borders(Borders::ALL)
            .border_style(self.colours.border_style);

        if app_state.should_get_widget_bounds() {
            // We must also recalculate how many lines are wrapping to properly get
            // scrolling to work on small terminal sizes... oh joy.

            app_state.help_dialog_state.height = block.inner(draw_loc).height;

            let mut overflow_buffer = 0;
            let paragraph_width = max(draw_loc.width.saturating_sub(2), 1);
            let mut prev_section_len = 0;

            constants::HELP_TEXT
                .iter()
                .enumerate()
                .for_each(|(itx, section)| {
                    let mut buffer = 0;

                    if itx == 0 {
                        section.iter().for_each(|text_line| {
                            buffer += UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16
                                / paragraph_width;
                        });

                        app_state.help_dialog_state.index_shortcuts[itx] = 0;
                    } else {
                        section.iter().for_each(|text_line| {
                            buffer += UnicodeWidthStr::width(*text_line).saturating_sub(1) as u16
                                / paragraph_width;
                        });

                        app_state.help_dialog_state.index_shortcuts[itx] =
                            app_state.help_dialog_state.index_shortcuts[itx - 1]
                                + 1
                                + prev_section_len;
                    }
                    prev_section_len = section.len() as u16 + buffer;
                    overflow_buffer += buffer;
                });

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
            Paragraph::new(styled_help_text.clone())
                .block(block)
                .style(self.colours.text_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .scroll((
                    app_state
                        .help_dialog_state
                        .scroll_state
                        .current_scroll_index,
                    0,
                )),
            draw_loc,
        );
    }
}
