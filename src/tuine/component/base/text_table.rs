pub mod table_column;
mod table_scroll_state;

use std::{borrow::Cow, cmp::min, panic::Location};

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
    tuine::{DrawContext, Event, Status, TmpComponent, ViewContext},
};

pub use self::table_column::{TextColumn, TextColumnConstraint};
use self::table_scroll_state::ScrollState as TextTableState;

#[derive(Clone, Debug, Default)]
pub struct StyleSheet {
    text: Style,
    selected_text: Style,
    table_header: Style,
}

/// A sortable, scrollable table for text data.
pub struct TextTable<'a, Message> {
    test_state: &'a mut TextTableState,
    state: TextTableState,
    column_widths: Vec<u16>,
    columns: Vec<TextColumn>,
    show_gap: bool,
    show_selected_entry: bool,
    rows: Vec<Row<'a>>,
    style_sheet: StyleSheet,
    sortable: bool,
    table_gap: u16,
    on_select: Option<Box<dyn Fn(usize) -> Message>>,
    on_selected_click: Option<Box<dyn Fn(usize) -> Message>>,
}

impl<'a, Message> TextTable<'a, Message> {
    #[track_caller]
    pub fn new<S: Into<Cow<'static, str>>>(ctx: &mut ViewContext<'_>, columns: Vec<S>) -> Self {
        let test_state = ctx.state::<TextTableState>(Location::caller());

        Self {
            test_state,
            state: TextTableState::default(),
            column_widths: vec![0; columns.len()],
            columns: columns
                .into_iter()
                .map(|name| TextColumn::new(name))
                .collect(),
            show_gap: true,
            show_selected_entry: true,
            rows: Vec::default(),
            style_sheet: StyleSheet::default(),
            sortable: false,
            table_gap: 0,
            on_select: None,
            on_selected_click: None,
        }
    }

    /// Sets the row to display in the table.
    ///
    /// Defaults to displaying no data if not set.
    pub fn rows(mut self, rows: Vec<Row<'a>>) -> Self {
        self.rows = rows;
        self
    }

    /// Whether to try to show a gap between the table headers and data.
    /// Note that if there isn't enough room, the gap will still be hidden.
    ///
    /// Defaults to `true` if not set.
    pub fn show_gap(mut self, show_gap: bool) -> Self {
        self.show_gap = show_gap;
        self
    }

    /// Whether to highlight the selected entry.
    ///
    /// Defaults to `true` if not set.
    pub fn show_selected_entry(mut self, show_selected_entry: bool) -> Self {
        self.show_selected_entry = show_selected_entry;
        self
    }

    /// Whether the table should display as sortable.
    ///
    /// Defaults to `false` if not set.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// What to do when selecting an entry. Expects a boxed function that takes in
    /// the currently selected index and returns a [`Message`].
    ///
    /// Defaults to `None` if not set.
    pub fn on_select(mut self, on_select: Option<Box<dyn Fn(usize) -> Message>>) -> Self {
        self.on_select = on_select;
        self
    }

    /// What to do when clicking on an entry that is already selected.
    ///
    /// Defaults to `None` if not set.
    pub fn on_selected_click(
        mut self, on_selected_click: Option<Box<dyn Fn(usize) -> Message>>,
    ) -> Self {
        self.on_selected_click = on_selected_click;
        self
    }

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

impl<'a, Message> TmpComponent<Message> for TextTable<'a, Message> {
    fn draw<B>(&mut self, context: DrawContext<'_>, frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        let rect = context.rect();

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
            let start = self
                .state
                .display_start_index(rect, scrollable_height as usize);
            let end = min(self.state.num_items(), start + scrollable_height as usize);

            self.rows[start..end].to_vec()
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

        frame.render_stateful_widget(table.widths(&widths), rect, self.state.tui_state());
    }

    fn on_event(&mut self, area: Rect, event: Event, messages: &mut Vec<Message>) -> Status {
        use crate::tuine::MouseBoundIntersect;
        use crossterm::event::{MouseButton, MouseEventKind};

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
                if mouse_event.does_mouse_intersect_bounds(area) {
                    match mouse_event.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let y = mouse_event.row - area.top();

                            if self.sortable && y == 0 {
                                todo!()
                            } else if y > self.table_gap {
                                let visual_index = usize::from(y - self.table_gap);
                                self.state.set_visual_index(visual_index)
                            } else {
                                Status::Ignored
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            let status = self.state.move_down(1);
                            if let Some(on_select) = &self.on_select {
                                messages.push(on_select(self.state.current_index()));
                            }
                            status
                        }
                        MouseEventKind::ScrollUp => {
                            let status = self.state.move_up(1);
                            if let Some(on_select) = &self.on_select {
                                messages.push(on_select(self.state.current_index()));
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
mod tests {}
