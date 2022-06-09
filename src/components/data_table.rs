use std::{
    cmp::{max, min},
    iter::once,
    marker::PhantomData,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    text::Text,
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{
    app::layout_manager::BottomWidget,
    constants::{SIDE_BORDERS, TABLE_GAP_HEIGHT_LIMIT},
    utils::gen_util::truncate_text,
};

pub mod data_column;
pub use data_column::*;

pub mod styling;
pub use styling::*;

pub mod props;
pub use props::DataTableProps;

pub mod state;
pub use state::{DataTableState, ScrollDirection};

pub mod draw;
pub use draw::*;

pub trait DataTableInner<DataType> {
    /// Builds a [`Row`] given data.
    fn to_data_row<'a>(&self, data: &'a DataType, columns: &[DataTableColumn]) -> Row<'a>;

    /// Returns the desired column widths in light of having seen data.
    fn column_widths(&self, data: &[DataType]) -> Vec<u16>;
}

/// A [`DataTable`] is a component that displays data in a tabular form.
///
/// Note that the data is not guaranteed to be sorted, or managed in any way.
/// FIXME: Add note about using sortable tables.
pub struct DataTable<DataType, T: DataTableInner<DataType>> {
    pub columns: Vec<DataTableColumn>,
    pub state: DataTableState,
    pub props: DataTableProps,
    pub inner: T,
    _pd: PhantomData<DataType>,
}

impl<DataType, T: DataTableInner<DataType>> DataTable<DataType, T> {
    pub fn new<C: Into<Vec<DataTableColumn>>>(columns: C, props: DataTableProps, inner: T) -> Self {
        Self {
            columns: columns.into(),
            state: DataTableState::default(),
            props,
            inner,
            _pd: PhantomData,
        }
    }

    /// Sets the scroll position to the first value.
    pub fn set_scroll_first(&mut self) {
        self.state.set_scroll_first();
    }

    /// Sets the scroll position to the last value.
    pub fn set_scroll_last(&mut self, num_entries: usize) {
        self.state.set_scroll_last(num_entries);
    }

    /// Updates the scroll position to be valid for the number of entries.
    pub fn update_num_entries(&mut self, num_entries: usize) {
        self.state.update_num_entries(num_entries);
    }

    /// Updates the scroll position if possible by a positive/negative offset. If there is a
    /// valid change, this function will also return the new position wrapped in an [`Option`].
    pub fn update_scroll_position(&mut self, change: i64, num_entries: usize) -> Option<usize> {
        self.state.update_scroll_position(change, num_entries)
    }

    /// Returns tui-rs' internal selection.
    pub fn tui_selected(&self) -> Option<usize> {
        self.state.table_state.selected()
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

            if let Some(title) = self.props.generate_title(
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

    pub fn draw<'d, B: Backend>(
        &mut self, f: &mut Frame<'_, B>, draw_info: &DrawInfo, data: &[DataType],
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
                // FIXME: This is currently kinda hardcoded in terms of calculations!
                let col_widths = self.inner.column_widths(data);

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

                self.columns
                    .calculate_column_widths(inner_width, self.props.left_to_right);

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
                    self.state.display_row_start_index = get_start_position(
                        num_rows,
                        &self.state.scroll_direction,
                        self.state.display_row_start_index,
                        self.state.current_scroll_position,
                        draw_info.force_redraw,
                    );
                    let start = self.state.display_row_start_index;
                    let end = min(data.len(), start + num_rows);
                    self.state.table_state.select(Some(
                        self.state.current_scroll_position.saturating_sub(start),
                    ));

                    data[start..end]
                        .iter()
                        .map(|row| self.inner.to_data_row(row, columns))
                };

                let headers = Self::build_header(columns)
                    .style(draw_info.styling.header_style)
                    .bottom_margin(table_gap);

                let widget = {
                    let highlight_style = if draw_info.is_on_widget()
                        || self.props.show_current_entry_when_unfocused
                    {
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

    /// Constructs the table header.
    fn build_header(columns: &[DataTableColumn]) -> Row<'_> {
        Row::new(columns.iter().filter_map(|c| {
            if c.calculated_width == 0 {
                None
            } else {
                Some(truncate_text(c.header.clone(), c.calculated_width.into()))
            }
        }))
    }
}

#[cfg(test)]
mod test {
    // FIXME: Do all testing!
}
