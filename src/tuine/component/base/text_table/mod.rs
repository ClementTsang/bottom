pub mod table_column;
pub use self::table_column::{TextColumn, TextColumnConstraint};

mod table_scroll_state;
use self::table_scroll_state::ScrollState;

pub mod data_row;
pub use data_row::DataRow;

pub mod data_cell;
pub use data_cell::DataCell;

pub mod sort_type;
pub use sort_type::SortType;

pub mod props;
pub use props::TextTableProps;

use std::{cmp::min, panic::Location};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Row, Table},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    constants::TABLE_GAP_HEIGHT_LIMIT,
    tuine::{DrawContext, Event, Key, StateContext, StatefulComponent, Status, TmpComponent},
};

#[derive(Clone, Debug, Default)]
pub struct StyleSheet {
    text: Style,
    selected_text: Style,
    table_header: Style,
}

#[derive(PartialEq, Default)]
pub struct TextTableState {
    scroll: ScrollState,
    sort: SortType,
}

/// A sortable, scrollable table for text data.
pub struct TextTable<Message> {
    key: Key,
    column_widths: Vec<u16>,
    columns: Vec<TextColumn>,
    show_gap: bool,
    show_selected_entry: bool,
    rows: Vec<DataRow>,
    style_sheet: StyleSheet,
    table_gap: u16,
    on_select: Option<Box<dyn Fn(usize) -> Message>>,
    on_selected_click: Option<Box<dyn Fn(usize) -> Message>>,
}

impl<Message> TextTable<Message> {
    fn update_column_widths(&mut self, bounds: Rect) {
        let total_width = bounds.width;
        let mut width_remaining = bounds.width;

        let mut column_widths: Vec<u16> = self
            .columns
            .iter()
            .map(|column| {
                let width = match column.width_constraint {
                    TextColumnConstraint::Fill => {
                        let desired = column.name.graphemes(true).count().saturating_add(1) as u16;
                        min(desired, width_remaining)
                    }
                    TextColumnConstraint::Length(length) => min(length, width_remaining),
                    TextColumnConstraint::Percentage(percentage) => {
                        let length = total_width * percentage / 100;
                        min(length, width_remaining)
                    }
                    TextColumnConstraint::MaxLength(length) => {
                        let desired = column.name.graphemes(true).count().saturating_add(1) as u16;
                        min(min(length, desired), width_remaining)
                    }
                    TextColumnConstraint::MaxPercentage(percentage) => {
                        let desired = column.name.graphemes(true).count().saturating_add(1) as u16;
                        let length = total_width * percentage / 100;
                        min(min(desired, length), width_remaining)
                    }
                };
                width_remaining -= width;
                width
            })
            .collect();

        if !column_widths.is_empty() {
            let amount_per_slot = width_remaining / column_widths.len() as u16;
            width_remaining %= column_widths.len() as u16;
            for (index, width) in column_widths.iter_mut().enumerate() {
                if (index as u16) < width_remaining {
                    *width += amount_per_slot + 1;
                } else {
                    *width += amount_per_slot;
                }
            }
        }

        self.column_widths = column_widths;
    }
}

impl<Message> StatefulComponent<Message> for TextTable<Message> {
    type Properties = TextTableProps<Message>;

    type ComponentState = TextTableState;

    #[track_caller]
    fn build(ctx: &mut crate::tuine::ViewContext<'_>, mut props: Self::Properties) -> Self {
        let sort = props.sort;
        let (key, state) = ctx.register_and_mut_state_with_default::<_, Self::ComponentState, _>(
            Location::caller(),
            || TextTableState {
                scroll: Default::default(),
                sort,
            },
        );

        state.scroll.set_num_items(props.rows.len());
        props.try_sort_data(state.sort);

        TextTable {
            key,
            column_widths: props.column_widths,
            columns: props.columns,
            show_gap: props.show_gap,
            show_selected_entry: props.show_selected_entry,
            rows: props.rows,
            style_sheet: props.style_sheet,
            table_gap: props.table_gap,
            on_select: props.on_select,
            on_selected_click: props.on_selected_click,
        }
    }
}

impl<Message> TmpComponent<Message> for TextTable<Message> {
    fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        let rect = draw_ctx.global_rect();
        let state = state_ctx.mut_state::<TextTableState>(self.key);
        state.scroll.set_num_items(self.rows.len()); // FIXME: Not a fan of this system like this - should be easier to do.

        self.table_gap = if !self.show_gap
            || (self.rows.len() + 2 > rect.height.into() && rect.height < TABLE_GAP_HEIGHT_LIMIT)
        {
            0
        } else {
            1
        };

        let table_extras = 1 + self.table_gap;
        let scrollable_height = rect.height.saturating_sub(table_extras);
        self.update_column_widths(rect);

        // Calculate widths first, since we need them later.
        let widths = self
            .column_widths
            .iter()
            .map(|column| Constraint::Length(*column))
            .collect::<Vec<_>>();

        // Then calculate rows. We truncate the amount of data read based on height,
        // as well as truncating some entries based on available width.
        let data_slice = {
            // Note: `get_list_start` already ensures `start` is within the bounds of the number of items, so no need to check!
            let start = state
                .scroll
                .display_start_index(rect, scrollable_height as usize);
            let end = min(state.scroll.num_items(), start + scrollable_height as usize);

            self.rows.drain(start..end).into_iter().map(|row| {
                let r: Row<'_> = row.into();
                r
            })
        };

        // Now build up our headers...
        let header = Row::new(self.columns.iter().map(|column| column.name.clone()))
            .style(self.style_sheet.table_header)
            .bottom_margin(self.table_gap);

        let mut table = Table::new(data_slice)
            .header(header)
            .style(self.style_sheet.text);

        if self.show_selected_entry {
            table = table.highlight_style(self.style_sheet.selected_text);
        }

        frame.render_stateful_widget(table.widths(&widths), rect, state.scroll.tui_state());
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        use crate::tuine::MouseBoundIntersect;
        use crossterm::event::{MouseButton, MouseEventKind};

        let rect = draw_ctx.global_rect();
        let state = state_ctx.mut_state::<TextTableState>(self.key);

        match event {
            Event::Keyboard(key_event) => {
                if key_event.modifiers.is_empty() {
                    match key_event.code {
                        _ => Status::Ignored,
                    }
                } else {
                    Status::Ignored
                }
            }
            Event::Mouse(mouse_event) => {
                if mouse_event.does_mouse_intersect_bounds(rect) {
                    match mouse_event.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let y = mouse_event.row - rect.top();
                            if y == 0 {
                                let x = mouse_event.column - rect.left();
                                match state.sort {
                                    SortType::Unsortable => Status::Ignored,
                                    SortType::Ascending(column) | SortType::Descending(column) => {
                                        let mut cursor = 0;
                                        for (selected_column, width) in
                                            self.column_widths.iter().enumerate()
                                        {
                                            let end = cursor + width;

                                            if x >= cursor && x <= end {
                                                match state.sort {
                                                    SortType::Ascending(_) => {
                                                        if selected_column == column {
                                                            // FIXME: This should handle default sorting orders...
                                                            state.sort = SortType::Descending(
                                                                selected_column,
                                                            );
                                                        } else {
                                                            state.sort = SortType::Ascending(
                                                                selected_column,
                                                            );
                                                        }
                                                    }
                                                    SortType::Descending(_) => {
                                                        if selected_column == column {
                                                            // FIXME: This should handle default sorting orders...
                                                            state.sort = SortType::Ascending(
                                                                selected_column,
                                                            );
                                                        } else {
                                                            state.sort = SortType::Descending(
                                                                selected_column,
                                                            );
                                                        }
                                                    }
                                                    SortType::Unsortable => unreachable!(), // Should be impossible by above check.
                                                }

                                                return Status::Captured;
                                            } else {
                                                cursor += width;
                                            }
                                        }
                                        Status::Ignored
                                    }
                                }
                            } else if y > self.table_gap {
                                let visual_index = usize::from(y - self.table_gap);
                                match state.scroll.set_visual_index(visual_index) {
                                    Status::Captured => Status::Captured,
                                    Status::Ignored => {
                                        if let Some(on_selected_click) = &self.on_selected_click {
                                            messages.push(on_selected_click(
                                                state.scroll.current_index(),
                                            ));
                                            Status::Captured
                                        } else {
                                            Status::Ignored
                                        }
                                    }
                                }
                            } else {
                                Status::Ignored
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            let status = state.scroll.move_down(1);
                            if let Some(on_select) = &self.on_select {
                                messages.push(on_select(state.scroll.current_index()));
                            }
                            status
                        }
                        MouseEventKind::ScrollUp => {
                            let status = state.scroll.move_up(1);
                            if let Some(on_select) = &self.on_select {
                                messages.push(on_select(state.scroll.current_index()));
                            }
                            status
                        }
                        _ => Status::Ignored,
                    }
                } else {
                    Status::Ignored
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tuine::{
        text_table::SortType, StateMap, StatefulComponent, TextTableProps, ViewContext,
    };

    use super::{DataRow, TextTable};

    type Message = ();

    fn ctx<'a>(map: &'a mut StateMap) -> ViewContext<'a> {
        ViewContext::new(map)
    }

    #[test]
    fn sorting() {
        let rows = vec![
            DataRow::default().cell("A").cell(2),
            DataRow::default().cell("B").cell(3),
            DataRow::default().cell("C").cell(1),
        ];
        let row_length = rows.len();
        let index = 1;

        let mut map = StateMap::default();
        let ctx = &mut ctx(&mut map);
        let table: TextTable<Message> = TextTable::build(
            ctx,
            TextTableProps::new(vec!["Sensor", "Temp"])
                .default_sort(SortType::Ascending(index))
                .rows(rows),
        );

        assert_eq!(
            table.rows.len(),
            row_length,
            "The number of cells should be equal to the vector passed in."
        );
        let mut prev = &table.rows[0].cells()[index];
        for row in &table.rows[1..] {
            let curr = &row.cells()[index];
            assert!(
                prev <= curr,
                "The previous value should be less or equal to the current one."
            );
            prev = curr;
        }
    }

    #[test]
    fn resorting() {
        let rows = vec![
            DataRow::default().cell("A").cell(2),
            DataRow::default().cell("B").cell(3),
            DataRow::default().cell("C").cell(1),
        ];
        let row_length = rows.len();
        let index = 1;
        let new_index = 0;

        let mut map = StateMap::default();
        let ctx = &mut ctx(&mut map);
        let table: TextTable<Message> = TextTable::build(
            ctx,
            TextTableProps::new(vec!["Sensor", "Temp"])
                .default_sort(SortType::Ascending(index))
                .rows(rows)
                .default_sort(SortType::Ascending(new_index)),
        );

        assert_eq!(
            table.rows.len(),
            row_length,
            "The number of cells should be equal to the vector passed in."
        );
        let mut prev = &table.rows[0].cells()[new_index];
        for row in &table.rows[1..] {
            let curr = &row.cells()[new_index];
            assert!(
                prev <= curr,
                "The previous value should be less or equal to the current one."
            );
            prev = curr;
        }
    }

    #[test]
    fn reverse_sorting() {
        let rows = vec![
            DataRow::default().cell("A").cell(2),
            DataRow::default().cell("B").cell(3),
            DataRow::default().cell("C").cell(1),
        ];
        let row_length = rows.len();
        let index = 1;

        let mut map = StateMap::default();
        let ctx = &mut ctx(&mut map);
        let table: TextTable<Message> = TextTable::build(
            ctx,
            TextTableProps::new(vec!["Sensor", "Temp"])
                .default_sort(SortType::Descending(index))
                .rows(rows),
        );

        assert_eq!(
            table.rows.len(),
            row_length,
            "The number of cells should be equal to the vector passed in."
        );
        let mut prev = &table.rows[0].cells()[index];
        for row in &table.rows[1..] {
            let curr = &row.cells()[index];
            assert!(
                prev >= curr,
                "The previous value should be bigger or equal to the current one."
            );
            prev = curr;
        }
    }

    #[test]
    fn adding_row() {
        let rows = vec![
            DataRow::default().cell("A").cell(2),
            DataRow::default().cell("B").cell(3),
            DataRow::default().cell("C").cell(1),
        ];
        let row_length = rows.len();
        let index = 1;

        let mut map = StateMap::default();
        let ctx = &mut ctx(&mut map);
        let table: TextTable<Message> = TextTable::build(
            ctx,
            TextTableProps::new(vec!["Sensor", "Temp"])
                .rows(rows)
                .default_sort(SortType::Ascending(index))
                .row(DataRow::default().cell("X").cell(0)),
        );

        assert_eq!(
            table.rows.len(),
            row_length + 1,
            "The number of cells should be equal to the vector passed in."
        );
        let mut prev = &table.rows[0].cells()[index];
        for row in &table.rows[1..] {
            let curr = &row.cells()[index];
            assert!(
                prev <= curr,
                "The previous value should be less or equal to the current one."
            );
            prev = curr;
        }
    }

    #[test]
    fn no_sort() {
        let original_rows = vec![
            DataRow::default().cell("A").cell(2),
            DataRow::default().cell("B").cell(3),
            DataRow::default().cell("C").cell(1),
            DataRow::default().cell("X").cell(0),
        ];
        let rows = original_rows[0..3].to_vec();
        let row_length = original_rows.len();

        let mut map = StateMap::default();
        let ctx = &mut ctx(&mut map);
        let table: TextTable<Message> = TextTable::build(
            ctx,
            TextTableProps::new(vec!["Sensor", "Temp"])
                .rows(rows)
                .row(original_rows[3].clone()),
        );

        assert_eq!(
            table.rows.len(),
            row_length,
            "The number of cells should be equal to the vector passed in."
        );

        table
            .rows
            .into_iter()
            .zip(original_rows)
            .for_each(|(a_row, b_row)| {
                a_row
                    .cells()
                    .into_iter()
                    .zip(b_row.cells())
                    .for_each(|(a, b)| {
                        assert_eq!(a, b, "Each DataCell should be equal.");
                    });
            });
    }
}
