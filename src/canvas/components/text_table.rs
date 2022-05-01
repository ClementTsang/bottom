use std::{borrow::Cow, cmp::min};

use concat_string::concat_string;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Row, Table},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::{self, TableComponentState},
    constants::{SIDE_BORDERS, TABLE_GAP_HEIGHT_LIMIT},
    data_conversion::{CellContent, TableData},
};

pub struct TextTable<'a> {
    pub table_gap: u16,
    pub table_height_offset: u16,
    pub is_force_redraw: bool,
    pub recalculate_column_widths: bool,

    /// The header style.
    pub header_style: Style,

    /// The border style.
    pub border_style: Style,

    /// The highlighted text style.
    pub highlighted_text_style: Style,

    /// The graph title.
    pub title: Cow<'a, str>,

    /// Whether this graph is expanded.
    pub is_expanded: bool,

    /// Whether this widget is selected.
    pub is_on_widget: bool,

    /// Whether to draw all borders.
    pub draw_border: bool,

    /// Whether to show the scroll position.
    pub show_table_scroll_position: bool,

    /// The title style.
    pub title_style: Style,

    /// The text style.
    pub text_style: Style,

    /// Whether to determine widths from left to right.
    pub left_to_right: bool,
}

impl<'a> TextTable<'a> {
    /// Generates a title for the [`TextTable`] widget, given the available space.
    fn generate_title(&self, draw_loc: Rect, pos: usize, total: usize) -> Spans<'_> {
        let title = if self.show_table_scroll_position {
            let title_string = concat_string!(
                self.title,
                "(",
                pos.to_string(),
                " of ",
                total.to_string(),
                ") "
            );

            if title_string.len() + 2 <= draw_loc.width.into() {
                title_string
            } else {
                self.title.to_string()
            }
        } else {
            self.title.to_string()
        };

        if self.is_expanded {
            let title_base = concat_string!(title, "── Esc to go back ");
            let esc = concat_string!(
                "─",
                "─".repeat(usize::from(draw_loc.width).saturating_sub(
                    UnicodeSegmentation::graphemes(title_base.as_str(), true).count() + 2
                )),
                "─ Esc to go back "
            );
            Spans::from(vec![
                Span::styled(title, self.title_style),
                Span::styled(esc, self.border_style),
            ])
        } else {
            Spans::from(Span::styled(title, self.title_style))
        }
    }
    pub fn draw_text_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, draw_loc: Rect, state: &mut TableComponentState,
        table_data: &TableData,
    ) -> Rect {
        let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
            0
        } else {
            self.table_gap
        };

        let sliced_vec = {
            let num_rows = usize::from(
                (draw_loc.height + 1 - table_gap).saturating_sub(self.table_height_offset),
            );
            let start = get_start_position(
                num_rows,
                &state.scroll_direction,
                &mut state.scroll_bar,
                state.current_scroll_position,
                self.is_force_redraw,
            );
            let end = min(table_data.data.len(), start + num_rows + 1);
            state
                .table_state
                .select(Some(state.current_scroll_position.saturating_sub(start)));
            &table_data.data[start..end]
        };

        // Calculate widths
        if self.recalculate_column_widths {
            state
                .columns
                .iter_mut()
                .zip(&table_data.row_widths)
                .for_each(|(column, data_width)| match &mut column.width_bounds {
                    app::WidthBounds::Soft {
                        min_width: _,
                        desired,
                        max_percentage: _,
                    } => {
                        *desired = std::cmp::max(column.name.len(), *data_width) as u16;
                    }
                    app::WidthBounds::Hard(_width) => {}
                });

            state.calculate_column_widths(draw_loc.width, self.left_to_right);
        }

        let columns = &state.columns;
        let widths = &state.calculated_widths;
        // TODO: Maybe truncate this too?
        let header = Row::new(columns.iter().map(|c| Text::raw(c.name.as_ref())))
            .style(self.header_style)
            .bottom_margin(table_gap);
        let disk_rows = sliced_vec.iter().map(|row| {
            Row::new(
                row.iter()
                    .zip(widths)
                    .map(|(cell, width)| truncate_text(cell, (*width).into())),
            )
        });

        let title = self.generate_title(
            draw_loc,
            state.current_scroll_position.saturating_add(1),
            table_data.data.len(),
        );

        let disk_block = if self.draw_border {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(self.border_style)
        } else if self.is_on_widget {
            Block::default()
                .borders(SIDE_BORDERS)
                .border_style(self.border_style)
        } else {
            Block::default().borders(Borders::NONE)
        };

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(if self.is_on_widget || self.draw_border {
                0
            } else {
                1
            })
            .direction(Direction::Horizontal)
            .split(draw_loc)[0];

        // Draw!
        f.render_stateful_widget(
            Table::new(disk_rows)
                .block(disk_block)
                .header(header)
                .highlight_style(self.highlighted_text_style)
                .style(self.text_style)
                .widths(
                    &(widths
                        .iter()
                        .map(|w| Constraint::Length(*w))
                        .collect::<Vec<_>>()),
                ),
            margined_draw_loc,
            &mut state.table_state,
        );

        margined_draw_loc
    }
}

/// Truncates text if it is too long, and adds an ellipsis at the end if needed.
fn truncate_text(content: &CellContent, width: usize) -> Text<'_> {
    let (text, opt) = match content {
        CellContent::Simple(s) => (s, None),
        CellContent::HasShort { short, long } => (long, Some(short)),
    };

    let graphemes = UnicodeSegmentation::graphemes(text.as_ref(), true).collect::<Vec<&str>>();
    if graphemes.len() > width && width > 0 {
        if let Some(s) = opt {
            // If an alternative exists, use that.
            Text::raw(s.as_ref())
        } else {
            // Truncate with ellipsis
            let first_n = graphemes[..(width - 1)].concat();
            Text::raw(concat_string!(first_n, "…"))
        }
    } else {
        Text::raw(text.as_ref())
    }
}

/// Gets the starting position of a table.
pub fn get_start_position(
    num_rows: usize, scroll_direction: &app::ScrollDirection, scroll_position_bar: &mut usize,
    currently_selected_position: usize, is_force_redraw: bool,
) -> usize {
    if is_force_redraw {
        *scroll_position_bar = 0;
    }

    match scroll_direction {
        app::ScrollDirection::Down => {
            if currently_selected_position < *scroll_position_bar + num_rows {
                // If, using previous_scrolled_position, we can see the element
                // (so within that and + num_rows) just reuse the current previously scrolled position
                *scroll_position_bar
            } else if currently_selected_position >= num_rows {
                // Else if the current position past the last element visible in the list, omit
                // until we can see that element
                *scroll_position_bar = currently_selected_position - num_rows;
                *scroll_position_bar
            } else {
                // Else, if it is not past the last element visible, do not omit anything
                0
            }
        }
        app::ScrollDirection::Up => {
            if currently_selected_position <= *scroll_position_bar {
                // If it's past the first element, then show from that element downwards
                *scroll_position_bar = currently_selected_position;
            } else if currently_selected_position >= *scroll_position_bar + num_rows {
                *scroll_position_bar = currently_selected_position - num_rows;
            }
            // Else, don't change what our start position is from whatever it is set to!
            *scroll_position_bar
        }
    }
}

#[cfg(test)]
mod test {}
