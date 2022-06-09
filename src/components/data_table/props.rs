use std::borrow::Cow;

use concat_string::concat_string;
use tui::text::{Span, Spans};
use unicode_segmentation::UnicodeSegmentation;

use crate::components::data_table::DrawInfo;

pub struct DataTableProps {
    /// An optional title for the table.
    pub title: Option<Cow<'static, str>>,

    /// The size of the gap between the header and rows.
    pub table_gap: u16,

    /// Whether this table determines column widths from left to right.
    pub left_to_right: bool,

    /// Whether this table is a basic table.
    pub is_basic: bool,

    /// Whether to show the table scroll position.
    pub show_table_scroll_position: bool,

    /// Whether to show the current entry as highlighted when not focused.
    pub show_current_entry_when_unfocused: bool,
}

impl DataTableProps {
    /// Generates a title, given the available space.
    pub fn generate_title<'a>(
        &self, draw_info: &'a DrawInfo, current_index: usize, total_items: usize,
    ) -> Option<Spans<'a>> {
        self.title.as_ref().map(|title| {
            let draw_loc = draw_info.loc;
            let title_style = draw_info.styling.title_style;
            let border_style = if draw_info.is_on_widget() {
                draw_info.styling.highlighted_border_style
            } else {
                draw_info.styling.border_style
            };

            let title = if self.show_table_scroll_position {
                let pos = current_index.to_string();
                let tot = total_items.to_string();
                let title_string = concat_string!(title, "(", pos, " of ", tot, ") ");

                if title_string.len() + 2 <= draw_loc.width.into() {
                    title_string
                } else {
                    title.to_string()
                }
            } else {
                title.to_string()
            };

            if draw_info.is_expanded() {
                let title_base = concat_string!(title, "── Esc to go back ");
                let lines = "─".repeat(usize::from(draw_loc.width).saturating_sub(
                    UnicodeSegmentation::graphemes(title_base.as_str(), true).count() + 2,
                ));
                let esc = concat_string!("─", lines, "─ Esc to go back ");
                Spans::from(vec![
                    Span::styled(title, title_style),
                    Span::styled(esc, border_style),
                ])
            } else {
                Spans::from(Span::styled(title, title_style))
            }
        })
    }
}
