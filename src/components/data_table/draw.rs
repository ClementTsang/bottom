use std::cmp::{max, min};

use concat_string::concat_string;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Row, Table},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use super::{
    CalculateColumnWidths, ColumnHeader, ColumnWidthBounds, DataTable, DataTableColumn, DataToCell,
    SortType,
};
use crate::{
    app::layout_manager::BottomWidget,
    canvas::Painter,
    constants::{SIDE_BORDERS, TABLE_GAP_HEIGHT_LIMIT},
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
        let border_style = match draw_info.selection_state {
            SelectionState::NotSelected => self.styling.border_style,
            SelectionState::Selected | SelectionState::Expanded => {
                self.styling.highlighted_border_style
            }
        };

        if !self.props.is_basic {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(title) = self.generate_title(draw_info, data_len) {
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

    /// Generates a title, given the available space.
    pub fn generate_title<'a>(
        &self, draw_info: &'a DrawInfo, total_items: usize,
    ) -> Option<Line<'a>> {
        self.props.title.as_ref().map(|title| {
            let current_index = self.state.current_index.saturating_add(1);
            let draw_loc = draw_info.loc;
            let title_style = self.styling.title_style;
            let border_style = if draw_info.is_on_widget() {
                self.styling.highlighted_border_style
            } else {
                self.styling.border_style
            };

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

            if draw_info.is_expanded() {
                let title_base = concat_string!(title, "── Esc to go back ");
                let lines = "─".repeat(usize::from(draw_loc.width).saturating_sub(
                    UnicodeSegmentation::graphemes(title_base.as_str(), true).count() + 2,
                ));
                let esc = concat_string!("─", lines, "─ Esc to go back ");
                Line::from(vec![
                    Span::styled(title, title_style),
                    Span::styled(esc, border_style),
                ])
            } else {
                Line::from(Span::styled(title, title_style))
            }
        })
    }

    pub fn draw(
        &mut self, f: &mut Frame<'_>, draw_info: &DrawInfo, widget: Option<&mut BottomWidget>,
        painter: &Painter,
    ) {
        let draw_horizontal = !self.props.is_basic || draw_info.is_on_widget();
        let draw_loc = draw_info.loc;
        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(u16::from(!draw_horizontal))
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

            let columns = &self.columns;
            if !self.data.is_empty() || !self.first_draw {
                self.first_draw = false; // TODO: Doing it this way is fine, but it could be done better (e.g. showing custom no results/entries message)

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
                                    data_row.to_cell(column.inner(), width)
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
                    let mut table = Table::default()
                        .rows(rows)
                        .block(block)
                        .highlight_style(highlight_style)
                        .style(self.styling.text_style);

                    if show_header {
                        table = table.header(headers);
                    }

                    table
                };

                let table_state = &mut self.state.table_state;
                f.render_stateful_widget(
                    widget.widths(
                        &(self
                            .state
                            .calculated_widths
                            .iter()
                            .filter_map(|&width| {
                                if width == 0 {
                                    None
                                } else {
                                    Some(Constraint::Length(width))
                                }
                            })
                            .collect::<Vec<_>>()),
                    ),
                    margined_draw_loc,
                    table_state,
                );
            } else {
                let table = Table::new(
                    [Row::new(Text::raw("No data"))],
                    [Constraint::Percentage(100)],
                )
                .block(block)
                .style(self.styling.text_style);
                f.render_widget(table, margined_draw_loc);
            }
        }
    }
}
