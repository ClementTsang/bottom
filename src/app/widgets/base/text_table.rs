use crossterm::event::{KeyEvent, MouseEvent};
use tui::layout::Rect;

use crate::app::{event::EventResult, Component, Scrollable};

/// A [`Column`] represents some column in a [`TextTable`].
pub struct Column {
    pub name: &'static str,
    pub shortcut: Option<KeyEvent>,
    pub default_descending: bool,

    // TODO: I would remove these in the future, storing them here feels weird...
    pub desired_column_width: u16,
    pub calculated_column_width: u16,
    pub x_bounds: (u16, u16),
}

impl Column {
    /// Creates a new [`Column`], given a name and optional shortcut.
    pub fn new(name: &'static str, shortcut: Option<KeyEvent>, default_descending: bool) -> Self {
        Self {
            name,
            desired_column_width: 0,
            calculated_column_width: 0,
            x_bounds: (0, 0),
            shortcut,
            default_descending,
        }
    }
}

/// A sortable, scrollable table with columns.
pub struct TextTable {
    /// Controls the scrollable state.
    scrollable: Scrollable,

    /// The columns themselves.
    columns: Vec<Column>,

    /// Whether to show a gap between the column headers and the columns.
    show_gap: bool,

    /// The bounding box of the [`TextTable`].
    bounds: Rect, // TODO: Consider moving bounds to something else???

    /// Which index we're sorting by.
    sort_index: usize,

    /// Whether we're sorting by ascending order.
    sort_ascending: bool,
}

impl TextTable {
    pub fn new(num_items: usize, columns: Vec<(&'static str, Option<KeyEvent>, bool)>) -> Self {
        Self {
            scrollable: Scrollable::new(num_items),
            columns: columns
                .into_iter()
                .map(|(name, shortcut, default_descending)| Column {
                    name,
                    desired_column_width: 0,
                    calculated_column_width: 0,
                    x_bounds: (0, 0),
                    shortcut,
                    default_descending,
                })
                .collect(),
            show_gap: true,
            bounds: Rect::default(),
            sort_index: 0,
            sort_ascending: true,
        }
    }

    pub fn try_show_gap(mut self, show_gap: bool) -> Self {
        self.show_gap = show_gap;
        self
    }

    pub fn sort_index(mut self, sort_index: usize) -> Self {
        self.sort_index = sort_index;
        self
    }

    pub fn update_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    pub fn update_calculated_column_bounds(&mut self, calculated_bounds: &[u16]) {
        self.columns
            .iter_mut()
            .zip(calculated_bounds.iter())
            .for_each(|(column, bound)| column.calculated_column_width = *bound);
    }

    pub fn desired_column_bounds(&self) -> Vec<u16> {
        self.columns
            .iter()
            .map(|column| column.desired_column_width)
            .collect()
    }

    pub fn column_names(&self) -> Vec<&'static str> {
        self.columns.iter().map(|column| column.name).collect()
    }

    pub fn update_num_items(&mut self, num_items: usize) {
        self.scrollable.update_num_items(num_items);
    }

    pub fn update_a_column(&mut self, index: usize, column: Column) {
        if let Some(c) = self.columns.get_mut(index) {
            *c = column;
        }
    }

    pub fn update_columns(&mut self, columns: Vec<Column>) {
        self.columns = columns;
        if self.columns.len() <= self.sort_index {
            self.sort_index = self.columns.len() - 1;
        }
    }
}

impl Component for TextTable {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        for (index, column) in self.columns.iter().enumerate() {
            if let Some(shortcut) = column.shortcut {
                if shortcut == event {
                    if self.sort_index == index {
                        // Just flip the sort if we're already sorting by this.
                        self.sort_ascending = !self.sort_ascending;
                    } else {
                        self.sort_index = index;
                        self.sort_ascending = !column.default_descending;
                    }
                    return EventResult::Redraw;
                }
            }
        }

        self.scrollable.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        // Note these are representing RELATIVE coordinates!
        let x = event.column - self.bounds.left();
        let y = event.row - self.bounds.top();

        if y == 0 {
            for (index, column) in self.columns.iter().enumerate() {
                let (start, end) = column.x_bounds;
                if start >= x && end <= y {
                    if self.sort_index == index {
                        // Just flip the sort if we're already sorting by this.
                        self.sort_ascending = !self.sort_ascending;
                    } else {
                        self.sort_index = index;
                        self.sort_ascending = !column.default_descending;
                    }
                }
            }

            EventResult::NoRedraw
        } else {
            self.scrollable.handle_mouse_event(event)
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}
