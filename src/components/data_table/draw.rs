pub mod draw_info;
pub use draw_info::{DrawInfo, SelectionState};

use std::{
    cmp::{max, min},
    iter::once,
};

use concat_string::concat_string;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Row, Table},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::layout_manager::BottomWidget,
    constants::{SIDE_BORDERS, TABLE_GAP_HEIGHT_LIMIT},
    utils::gen_util::truncate_text,
};

use super::{ColumnWidthBounds, DataColumn, DataTable, ScrollDirection, ToDataRow};

// For now, the implementation lives here as just a basic impl. Ideally, I change this to a trait impl
// on some Draw trait in the future.
impl<RowType: ToDataRow> DataTable<RowType> {
    /// Generates a title for the [`TextTable`] widget, given the available space.
    fn generate_title<'a>(
        &self, draw_info: &'a DrawInfo, pos: usize, total: usize,
    ) -> Option<Spans<'a>> {
        self.props.title.as_ref().map(|title| {
            let draw_loc = draw_info.loc;
            let title_style = draw_info.styling.title_style;
            let border_style = if draw_info.is_on_widget() {
                draw_info.styling.highlighted_border_style
            } else {
                draw_info.styling.border_style
            };

            let title = if self.props.show_table_scroll_position {
                let title_string =
                    concat_string!(title, "(", pos.to_string(), " of ", total.to_string(), ") ");

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
                let esc = concat_string!(
                    "─",
                    "─".repeat(usize::from(draw_loc.width).saturating_sub(
                        UnicodeSegmentation::graphemes(title_base.as_str(), true).count() + 2
                    )),
                    "─ Esc to go back "
                );
                Spans::from(vec![
                    Span::styled(title, title_style),
                    Span::styled(esc, border_style),
                ])
            } else {
                Spans::from(Span::styled(title, title_style))
            }
        })
    }

    fn block<'a>(&self, draw_info: &'a DrawInfo, data_len: usize) -> Block<'a> {
        let border_style = match draw_info.selection_state {
            SelectionState::NotSelected => draw_info.styling.border_style,
            SelectionState::Selected | SelectionState::Expanded => {
                draw_info.styling.highlighted_border_style
            }
        };

        if !self.props.is_basic {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(title) = self.generate_title(
                draw_info,
                self.state.current_scroll_position.saturating_add(1),
                data_len,
            ) {
                block.title(title)
            } else {
                block
            }
        } else if draw_info.is_on_widget() {
            // Implies it is basic mode but selected.
            Block::default()
                .borders(SIDE_BORDERS)
                .border_style(border_style)
        } else {
            Block::default().borders(Borders::NONE)
        }
    }

    pub fn draw<B: Backend>(
        &mut self, f: &mut Frame<'_, B>, draw_info: &DrawInfo, data: &[RowType],
        widget: Option<&mut BottomWidget>,
    ) {
        let draw_horizontal = !self.props.is_basic || draw_info.is_on_widget();
        let draw_loc = draw_info.loc;
        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(if draw_horizontal { 0 } else { 1 })
            .direction(Direction::Horizontal)
            .split(draw_loc)[0];

        let block = self.block(draw_info, data.len());

        let (inner_width, inner_height) = {
            let inner_rect = block.inner(margined_draw_loc);
            (inner_rect.width, inner_rect.height)
        };

        if inner_width == 0 || inner_height == 0 {
            f.render_widget(block, margined_draw_loc);
        } else {
            // Calculate widths
            if draw_info.recalculate_column_widths {
                // NB: This is currently kinda hardcoded in terms of calculations!
                let col_widths = ToDataRow::column_widths(data);

                self.columns
                    .iter_mut()
                    .zip(&col_widths)
                    .for_each(|(column, width)| match &mut column.width_bounds {
                        ColumnWidthBounds::Soft { desired, .. } => {
                            *desired = max(column.header.len() as u16, *width);
                        }
                        ColumnWidthBounds::Hard(_) => {}
                        ColumnWidthBounds::HeaderWidth => {}
                    });

                self.calculate_column_widths(inner_width, self.props.left_to_right);

                // Update draw loc in widget map
                if let Some(widget) = widget {
                    widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                    widget.bottom_right_corner =
                        Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
                }
            }

            let show_header = inner_height > 1;
            let header_height = if show_header { 1 } else { 0 };
            let table_gap = if !show_header || draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                self.props.table_gap
            };

            let columns = &self.columns;
            if !data.is_empty() {
                let rows = {
                    let num_rows =
                        usize::from(inner_height.saturating_sub(table_gap + header_height));
                    let start = get_start_position(
                        num_rows,
                        &self.state.scroll_direction,
                        &mut self.state.display_row_start_index,
                        self.state.current_scroll_position,
                        draw_info.force_redraw,
                    );
                    let end = min(data.len(), start + num_rows);
                    self.state.table_state.select(Some(
                        self.state.current_scroll_position.saturating_sub(start),
                    ));

                    data[start..end].iter().map(|row| row.to_data_row(columns))
                };

                let headers = build_header(columns)
                    .style(draw_info.styling.header_style)
                    .bottom_margin(table_gap);

                let widget = {
                    let highlight_style = if draw_info.is_on_widget() {
                        draw_info.styling.highlighted_text_style
                    } else {
                        draw_info.styling.text_style
                    };
                    let mut table = Table::new(rows)
                        .block(block)
                        .highlight_style(highlight_style)
                        .style(draw_info.styling.text_style);

                    if show_header {
                        table = table.header(headers);
                    }

                    table
                };

                let table_state = &mut self.state.table_state;
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
                    table_state,
                );
            } else {
                let table = Table::new(once(Row::new(Text::raw("No data"))))
                    .block(block)
                    .style(draw_info.styling.text_style)
                    .widths(&[Constraint::Percentage(100)]);
                f.render_widget(table, margined_draw_loc);
            }
        }
    }
}

/// Gets the starting position of a table.
pub fn get_start_position(
    num_rows: usize, scroll_direction: &ScrollDirection, scroll_position_bar: &mut usize,
    currently_selected_position: usize, is_force_redraw: bool,
) -> usize {
    if is_force_redraw {
        *scroll_position_bar = 0;
    }

    match scroll_direction {
        ScrollDirection::Down => {
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
        ScrollDirection::Up => {
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

/// Constructs the table header.
fn build_header<'a>(columns: &'a [DataColumn]) -> Row<'a> {
    Row::new(columns.iter().filter_map(|c| {
        if c.calculated_width == 0 {
            None
        } else {
            Some(truncate_text(c.header.clone(), c.calculated_width.into()))
        }
    }))
}
