use crossterm::event::{KeyEvent, MouseEvent};
use tui::layout::Rect;

use crate::app::{event::EventResult, Scrollable, Widget};

struct Column {
    name: &'static str,

    // TODO: I would remove these in the future, storing them here feels weird...
    desired_column_width: u16,
    calculated_column_width: u16,

    x_bounds: (u16, u16),
}

impl Column {}

/// The [`Widget::UpdateState`] of a [`TextTable`].
pub struct TextTableUpdateData {
    num_items: Option<usize>,
    columns: Option<Vec<Column>>,
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
}

impl TextTable {
    pub fn new(num_items: usize, columns: Vec<&'static str>) -> Self {
        Self {
            scrollable: Scrollable::new(num_items),
            columns: columns
                .into_iter()
                .map(|name| Column {
                    name,
                    desired_column_width: 0,
                    calculated_column_width: 0,
                    x_bounds: (0, 0),
                })
                .collect(),
            show_gap: true,
            bounds: Rect::default(),
            sort_index: 0,
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
}

impl Widget for TextTable {
    type UpdateData = TextTableUpdateData;

    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
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
                    self.sort_index = index;
                }
            }

            EventResult::Continue
        } else {
            self.scrollable.handle_mouse_event(event)
        }
    }

    fn update(&mut self, update_data: Self::UpdateData) {
        if let Some(num_items) = update_data.num_items {
            self.scrollable.update(num_items);
        }

        if let Some(columns) = update_data.columns {
            self.columns = columns;
            if self.columns.len() <= self.sort_index {
                self.sort_index = self.columns.len() - 1;
            }
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}
