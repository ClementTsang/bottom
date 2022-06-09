use std::{
    borrow::Cow,
    cmp::{max, min},
};

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
    app::{self, layout_manager::BottomWidget},
    components::old_text_table::SortOrder,
    constants::{SIDE_BORDERS, TABLE_GAP_HEIGHT_LIMIT},
    data_conversion::{TableData, TableRow},
};

use super::{
    CellContent, SortState, TableComponentColumn, TableComponentHeader, TableComponentState,
    WidthBounds,
};

pub struct TextTableTitle<'a> {
    pub title: Cow<'a, str>,
    pub is_expanded: bool,
}

pub struct TextTable<'a> {
    pub table_gap: u16,
    pub is_force_redraw: bool, // TODO: Is this force redraw thing needed? Or is there a better way?
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

    pub fn draw_old_text_table<B: Backend, H: TableComponentHeader>(
        &self, f: &mut Frame<'_, B>, draw_loc: Rect, state: &mut TableComponentState<H>,
        table_data: &TableData, btm_widget: Option<&mut BottomWidget>,
    ) {
        // TODO: This is a *really* ugly hack to get basic mode to hide the border when not selected, without shifting everything.
        let is_not_basic = self.is_on_widget || self.draw_border;
        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(if is_not_basic { 0 } else { 1 })
            .direction(Direction::Horizontal)
            .split(draw_loc)[0];

        let block = if self.draw_border {
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

        let inner_rect = block.inner(margined_draw_loc);
        let (inner_width, inner_height) = { (inner_rect.width, inner_rect.height) };

        if inner_width == 0 || inner_height == 0 {
            f.render_widget(block, margined_draw_loc);
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
                    .zip(&table_data.col_widths)
                    .for_each(|(column, data_width)| match &mut column.width_bounds {
                        WidthBounds::Soft {
                            min_width: _,
                            desired,
                            max_percentage: _,
                        } => {
                            *desired = max(
                                *desired,
                                max(column.header.header_text().len(), *data_width) as u16,
                            );
                        }
                        WidthBounds::CellWidth => {}
                        WidthBounds::Hard(_width) => {}
                    });

                state.calculate_column_widths(inner_width, self.left_to_right);

                if let SortState::Sortable(st) = &mut state.sort_state {
                    let row_widths = state
                        .columns
                        .iter()
                        .filter_map(|c| {
                            if c.calculated_width == 0 {
                                None
                            } else {
                                Some(c.calculated_width)
                            }
                        })
                        .collect::<Vec<_>>();

                    st.update_visual_index(inner_rect, &row_widths);
                }

                // Update draw loc in widget map
                if let Some(btm_widget) = btm_widget {
                    btm_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                    btm_widget.bottom_right_corner =
                        Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
                }
            }

            let columns = &state.columns;
            let header = build_header(columns, &state.sort_state)
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

            if !table_data.data.is_empty() {
                let widget = {
                    let mut table = Table::new(table_rows)
                        .block(block)
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
            } else {
                f.render_widget(block, margined_draw_loc);
            }
        }
    }
}

/// Constructs the table header.
fn build_header<'a, H: TableComponentHeader>(
    columns: &'a [TableComponentColumn<H>], sort_state: &SortState,
) -> Row<'a> {
    use itertools::Either;

    const UP_ARROW: &str = "▲";
    const DOWN_ARROW: &str = "▼";

    let iter = match sort_state {
        SortState::Unsortable => Either::Left(columns.iter().filter_map(|c| {
            if c.calculated_width == 0 {
                None
            } else {
                Some(truncate_text(
                    c.header.header_text(),
                    c.calculated_width.into(),
                    None,
                ))
            }
        })),
        SortState::Sortable(s) => {
            let order = &s.order;
            let index = s.current_index;

            let arrow = match order {
                SortOrder::Ascending => UP_ARROW,
                SortOrder::Descending => DOWN_ARROW,
            };

            Either::Right(columns.iter().enumerate().filter_map(move |(itx, c)| {
                if c.calculated_width == 0 {
                    None
                } else if itx == index {
                    Some(truncate_suffixed_text(
                        c.header.header_text(),
                        arrow,
                        c.calculated_width.into(),
                        None,
                    ))
                } else {
                    Some(truncate_text(
                        c.header.header_text(),
                        c.calculated_width.into(),
                        None,
                    ))
                }
            }))
        }
    };

    Row::new(iter)
}

/// Truncates text if it is too long, and adds an ellipsis at the end if needed.
fn truncate_text(content: &CellContent, width: usize, row_style: Option<Style>) -> Text<'_> {
    let (main_text, alt_text) = match content {
        CellContent::Simple(s) => (s, None),
        CellContent::HasAlt {
            alt: short,
            main: long,
        } => (long, Some(short)),
    };

    let mut text = {
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(main_text.as_ref(), true).collect();
        if graphemes.len() > width && width > 0 {
            if let Some(s) = alt_text {
                // If an alternative exists, use that.
                Text::raw(s.as_ref())
            } else {
                // Truncate with ellipsis
                let first_n = graphemes[..(width - 1)].concat();
                Text::raw(concat_string!(first_n, "…"))
            }
        } else {
            Text::raw(main_text.as_ref())
        }
    };

    if let Some(row_style) = row_style {
        text.patch_style(row_style);
    }

    text
}

fn truncate_suffixed_text<'a>(
    content: &'a CellContent, suffix: &str, width: usize, row_style: Option<Style>,
) -> Text<'a> {
    let (main_text, alt_text) = match content {
        CellContent::Simple(s) => (s, None),
        CellContent::HasAlt {
            alt: short,
            main: long,
        } => (long, Some(short)),
    };

    let mut text = {
        let suffixed = concat_string!(main_text, suffix);
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(suffixed.as_str(), true).collect();
        if graphemes.len() > width && width > 1 {
            if let Some(alt) = alt_text {
                // If an alternative exists, use that + arrow.
                Text::raw(concat_string!(alt, suffix))
            } else {
                // Truncate with ellipsis + arrow.
                let first_n = graphemes[..(width - 2)].concat();
                Text::raw(concat_string!(first_n, "…", suffix))
            }
        } else {
            Text::raw(suffixed)
        }
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
mod test {
    use super::*;

    #[test]
    fn test_get_start_position() {
        use crate::app::ScrollDirection::{self, Down, Up};

        #[track_caller]
        fn test_get(
            bar: usize, rows: usize, direction: ScrollDirection, selected: usize, force: bool,
            expected_posn: usize, expected_bar: usize,
        ) {
            let mut bar = bar;
            assert_eq!(
                get_start_position(rows, &direction, &mut bar, selected, force),
                expected_posn,
                "returned start position should match"
            );
            assert_eq!(bar, expected_bar, "bar positions should match");
        }

        // Scrolling down from start
        test_get(0, 10, Down, 0, false, 0, 0);

        // Simple scrolling down
        test_get(0, 10, Down, 1, false, 0, 0);

        // Scrolling down from the middle high up
        test_get(0, 10, Down, 4, false, 0, 0);

        // Scrolling down into boundary
        test_get(0, 10, Down, 10, false, 1, 1);
        test_get(0, 10, Down, 11, false, 2, 2);

        // Scrolling down from the with non-zero bar
        test_get(5, 10, Down, 14, false, 5, 5);

        // Force redraw scrolling down (e.g. resize)
        test_get(5, 15, Down, 14, true, 0, 0);

        // Test jumping down
        test_get(1, 10, Down, 19, true, 10, 10);

        // Scrolling up from bottom
        test_get(10, 10, Up, 19, false, 10, 10);

        // Simple scrolling up
        test_get(10, 10, Up, 18, false, 10, 10);

        // Scrolling up from the middle
        test_get(10, 10, Up, 10, false, 10, 10);

        // Scrolling up into boundary
        test_get(10, 10, Up, 9, false, 9, 9);

        // Force redraw scrolling up (e.g. resize)
        test_get(5, 10, Up, 14, true, 5, 5);

        // Test jumping up
        test_get(10, 10, Up, 0, false, 0, 0);
    }
}
