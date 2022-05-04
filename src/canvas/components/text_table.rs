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
    data_conversion::{CellContent, TableData, TableRow},
};

pub struct TextTableTitle<'a> {
    pub title: Cow<'a, str>,
    pub is_expanded: bool,
}

pub struct TextTable<'a> {
    pub table_gap: u16,
    pub is_force_redraw: bool,
    pub recalculate_column_widths: bool,

    /// The header style.
    pub header_style: Style,

    /// The border style.
    pub border_style: Style,

    /// The highlighted text style.
    pub highlighted_text_style: Style,

    /// The graph title and whether it is expanded (if there is one).
    pub title: Option<TextTableTitle<'a>>,

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
    fn generate_title(&self, draw_loc: Rect, pos: usize, total: usize) -> Option<Spans<'_>> {
        self.title
            .as_ref()
            .map(|TextTableTitle { title, is_expanded }| {
                let title = if self.show_table_scroll_position {
                    let title_string = concat_string!(
                        title,
                        "(",
                        pos.to_string(),
                        " of ",
                        total.to_string(),
                        ") "
                    );

                    if title_string.len() + 2 <= draw_loc.width.into() {
                        title_string
                    } else {
                        title.to_string()
                    }
                } else {
                    title.to_string()
                };

                if *is_expanded {
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
            })
    }
    pub fn draw_text_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, draw_loc: Rect, state: &mut TableComponentState,
        table_data: &TableData,
    ) {
        // TODO: This is a *really* ugly hack to get basic mode to hide the border when not selected, without shifting everything.
        let is_not_basic = self.is_on_widget || self.draw_border;
        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(if is_not_basic { 0 } else { 1 })
            .direction(Direction::Horizontal)
            .split(draw_loc)[0];

        let disk_block = if self.draw_border {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.border_style);

            if let Some(title) = self.generate_title(
                draw_loc,
                state.current_scroll_position.saturating_add(1),
                table_data.data.len(),
            ) {
                block.title(title)
            } else {
                block
            }
        } else if self.is_on_widget {
            Block::default()
                .borders(SIDE_BORDERS)
                .border_style(self.border_style)
        } else {
            Block::default().borders(Borders::NONE)
        };

        let (inner_width, inner_height) = {
            let inner = disk_block.inner(margined_draw_loc);
            (inner.width, inner.height)
        };

        if inner_width == 0 || inner_height == 0 {
            f.render_widget(disk_block, margined_draw_loc);
        } else {
            let show_header = inner_height > 1;
            let header_height = if show_header { 1 } else { 0 };
            let table_gap = if !show_header || draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                self.table_gap
            };

            let sliced_vec = {
                let num_rows = usize::from(inner_height.saturating_sub(table_gap + header_height));
                let start = get_start_position(
                    num_rows,
                    &state.scroll_direction,
                    &mut state.scroll_bar,
                    state.current_scroll_position,
                    self.is_force_redraw,
                );
                let end = min(table_data.data.len(), start + num_rows);
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

                state.calculate_column_widths(inner_width, self.left_to_right);
            }

            let columns = &state.columns;
            let header = Row::new(columns.iter().filter_map(|c| {
                if c.calculated_width == 0 {
                    None
                } else {
                    Some(truncate_text(&c.name, c.calculated_width.into(), None))
                }
            }))
            .style(self.header_style)
            .bottom_margin(table_gap);
            let table_rows = sliced_vec.iter().map(|row| {
                let (row, style) = match row {
                    TableRow::Raw(row) => (row, None),
                    TableRow::Styled(row, style) => (row, Some(*style)),
                };

                Row::new(row.iter().zip(columns).filter_map(|(cell, c)| {
                    if c.calculated_width == 0 {
                        None
                    } else {
                        Some(truncate_text(cell, c.calculated_width.into(), style))
                    }
                }))
            });

            let widget = {
                let mut table = Table::new(table_rows)
                    .block(disk_block)
                    .highlight_style(self.highlighted_text_style)
                    .style(self.text_style);

                if show_header {
                    table = table.header(header);
                }

                table
            };

            f.render_stateful_widget(
                widget.widths(
                    &(columns
                        .iter()
                        .filter_map(|c| {
                            if c.calculated_width == 0 {
                                None
                            } else {
                                Some(Constraint::Length(c.calculated_width))
                            }
                        })
                        .collect::<Vec<_>>()),
                ),
                margined_draw_loc,
                &mut state.table_state,
            );
        }
    }
}

/// Truncates text if it is too long, and adds an ellipsis at the end if needed.
fn truncate_text(content: &CellContent, width: usize, row_style: Option<Style>) -> Text<'_> {
    let (text, opt) = match content {
        CellContent::Simple(s) => (s, None),
        CellContent::HasAlt {
            alt: short,
            main: long,
        } => (long, Some(short)),
    };

    let graphemes = UnicodeSegmentation::graphemes(text.as_ref(), true).collect::<Vec<&str>>();
    let mut text = if graphemes.len() > width && width > 0 {
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
    };

    if let Some(row_style) = row_style {
        text.patch_style(row_style);
    }

    text
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
                *scroll_position_bar = currently_selected_position - num_rows + 1;
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
                *scroll_position_bar = currently_selected_position - num_rows + 1;
            }
            // Else, don't change what our start position is from whatever it is set to!
            *scroll_position_bar
        }
    }
}

#[cfg(test)]
mod test {}
