use std::{
    cmp::{max, min},
    iter::once,
};

use concat_string::concat_string;
use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Row, Table},
};

use super::{
    CalculateColumnWidths, ColumnHeader, ColumnWidthBounds, DataTable, DataTableColumn, DataToCell,
    SortType,
};
use crate::{
    app::layout_manager::BottomWidget,
    canvas::{Painter, drawing_utils::widget_block},
    constants::TABLE_GAP_HEIGHT_LIMIT,
    utils::strings::truncate_to_text,
};

pub enum SelectionState {
    NotSelected,
    Selected,
    Expanded,
}

impl SelectionState {
    pub fn new(is_expanded: bool, is_on_widget: bool) -> Self {
        if is_expanded {
            SelectionState::Expanded
        } else if is_on_widget {
            SelectionState::Selected
        } else {
            SelectionState::NotSelected
        }
    }
}

/// A [`DrawInfo`] is information required on each draw call.
pub struct DrawInfo {
    pub loc: Rect,
    pub force_redraw: bool,
    pub recalculate_column_widths: bool,
    pub selection_state: SelectionState,
}

impl DrawInfo {
    pub fn is_on_widget(&self) -> bool {
        matches!(self.selection_state, SelectionState::Selected)
            || matches!(self.selection_state, SelectionState::Expanded)
    }

    pub fn is_expanded(&self) -> bool {
        matches!(self.selection_state, SelectionState::Expanded)
    }
}

impl<DataType, H, S, C> DataTable<DataType, H, S, C>
where
    DataType: DataToCell<H>,
    H: ColumnHeader,
    S: SortType,
    C: DataTableColumn<H>,
{
    fn block<'a>(&self, draw_info: &'a DrawInfo, data_len: usize) -> Block<'a> {
        let is_selected = match draw_info.selection_state {
            SelectionState::NotSelected => false,
            SelectionState::Selected | SelectionState::Expanded => true,
        };

        let border_style = if is_selected {
            self.styling.highlighted_border_style
        } else {
            self.styling.border_style
        };

        let mut block = widget_block(self.props.is_basic, is_selected, self.styling.border_type)
            .border_style(border_style);

        if let Some((left_title, right_title)) = self.generate_title(draw_info, data_len) {
            if !self.props.is_basic {
                block = block.title_top(left_title);
            }

            if let Some(right_title) = right_title {
                block = block.title_top(right_title);
            }
        }

        block
    }

    /// Generates a title, given the available space.
    fn generate_title(
        &self, draw_info: &'_ DrawInfo, total_items: usize,
    ) -> Option<(Line<'static>, Option<Line<'static>>)> {
        self.props.title.as_ref().map(|title| {
            let current_index = self.state.current_index.saturating_add(1);
            let draw_loc = draw_info.loc;
            let title_style = self.styling.title_style;

            let title = if self.props.show_table_scroll_position {
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

            let left_title = Line::from(Span::styled(title, title_style)).left_aligned();

            let right_title = if draw_info.is_expanded() {
                Some(Line::from(" Esc to go back ").right_aligned())
            } else {
                None
            };

            (left_title, right_title)
        })
    }

    pub fn draw(
        &mut self, f: &mut Frame<'_>, draw_info: &DrawInfo, widget: Option<&mut BottomWidget>,
        painter: &Painter,
    ) {
        let draw_loc = draw_info.loc;
        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(u16::from(self.props.is_basic && !draw_info.is_on_widget()))
            .direction(Direction::Horizontal)
            .split(draw_loc)[0];

        let block = self.block(draw_info, self.data.len());

        let (inner_width, inner_height) = {
            let inner_rect = block.inner(margined_draw_loc);
            self.state.inner_rect = inner_rect;
            (inner_rect.width, inner_rect.height)
        };

        if inner_width == 0 || inner_height == 0 {
            f.render_widget(block, margined_draw_loc);
        } else {
            // Calculate widths
            if draw_info.recalculate_column_widths {
                let col_widths = DataType::column_widths(&self.data, &self.columns);

                self.columns
                    .iter_mut()
                    .zip(&col_widths)
                    .for_each(|(column, &width)| {
                        let header_len = column.header_len() as u16;
                        if let ColumnWidthBounds::Soft {
                            desired,
                            max_percentage: _,
                        } = &mut column.bounds_mut()
                        {
                            *desired = max(header_len, width);
                        }
                    });

                self.state.calculated_widths = self
                    .columns
                    .calculate_column_widths(inner_width, self.props.left_to_right);

                // Update draw loc in widget map
                if let Some(widget) = widget {
                    widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                    widget.bottom_right_corner =
                        Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
                }
            }

            let show_header = inner_height > 1;
            let header_height = u16::from(show_header);
            let table_gap = if !show_header || draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                self.props.table_gap
            };

            if !self.data.is_empty() || !self.first_draw {
                if self.first_draw {
                    // TODO: Doing it this way is fine, but it could be done better (e.g. showing
                    // custom no results/entries message)
                    self.first_draw = false;
                    if let Some(first_index) = self.first_index {
                        self.set_position(first_index);
                    }
                }

                let columns = &self.columns;
                let rows = {
                    let num_rows =
                        usize::from(inner_height.saturating_sub(table_gap + header_height));
                    self.state
                        .get_start_position(num_rows, draw_info.force_redraw);
                    let start = self.state.display_start_index;
                    let end = min(self.data.len(), start + num_rows);
                    self.state
                        .table_state
                        .select(Some(self.state.current_index.saturating_sub(start)));

                    self.data[start..end].iter().map(|data_row| {
                        let row = Row::new(
                            columns
                                .iter()
                                .zip(&self.state.calculated_widths)
                                .filter_map(|(column, &width)| {
                                    data_row.to_cell_text(column.inner(), width).map(|content| {
                                        let content = truncate_to_text(&content, width.get());

                                        if let Some(style) =
                                            data_row.style_cell(column.inner(), painter)
                                        {
                                            Cell::new(content).style(style)
                                        } else {
                                            Cell::new(content)
                                        }
                                    })
                                }),
                        );

                        data_row.style_row(row, painter)
                    })
                };

                let headers = self
                    .sort_type
                    .build_header(columns, &self.state.calculated_widths)
                    .style(self.styling.header_style)
                    .bottom_margin(table_gap);

                let widget = {
                    let highlight_style = if draw_info.is_on_widget()
                        || self.props.show_current_entry_when_unfocused
                    {
                        self.styling.highlighted_text_style
                    } else {
                        self.styling.text_style
                    };
                    let mut table = Table::new(
                        rows,
                        self.state.calculated_widths.iter().map(|nzu| nzu.get()),
                    )
                    .block(block)
                    .row_highlight_style(highlight_style)
                    .style(self.styling.text_style);

                    if show_header {
                        table = table.header(headers);
                    }

                    table
                };

                let table_state = &mut self.state.table_state;
                f.render_stateful_widget(widget, margined_draw_loc, table_state);
            } else {
                let table = Table::new(
                    once(Row::new(Text::raw("No data"))),
                    [Constraint::Percentage(100)],
                )
                .block(block)
                .style(self.styling.text_style);
                f.render_widget(table, margined_draw_loc);
            }
        }
    }
}
