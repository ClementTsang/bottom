use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Table, TableState},
};

use crate::{
    app::{event::EventResult, Component, TextTable},
    canvas::Painter,
};

use super::text_table::{DesiredColumnWidth, SimpleColumn, TableColumn, TextTableData};

fn get_shortcut_name(e: &KeyEvent) -> String {
    let modifier = if e.modifiers.is_empty() {
        ""
    } else if let KeyModifiers::ALT = e.modifiers {
        "Alt+"
    } else if let KeyModifiers::SHIFT = e.modifiers {
        "Shift+"
    } else if let KeyModifiers::CONTROL = e.modifiers {
        "Ctrl+"
    } else {
        // For now, that's all we support, though combos/more could be added.
        ""
    };

    let key: Cow<'static, str> = match e.code {
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PgUp".into(),
        KeyCode::PageDown => "PgDown".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::BackTab => "BackTab".into(),
        KeyCode::Delete => "Del".into(),
        KeyCode::Insert => "Insert".into(),
        KeyCode::F(num) => format!("F{}", num).into(),
        KeyCode::Char(c) => format!("{}", c).into(),
        KeyCode::Null => "Null".into(),
        KeyCode::Esc => "Esc".into(),
    };

    format!("({}{})", modifier, key).into()
}

#[derive(Copy, Clone, Debug)]
pub enum SortStatus {
    NotSorting,
    SortAscending,
    SortDescending,
}

/// A trait for sortable columns.
pub trait SortableColumn {
    /// Returns the shortcut for the column, if it exists.
    fn shortcut(&self) -> &Option<(KeyEvent, String)>;

    /// Returns whether the column defaults to sorting in descending order or not.
    fn default_descending(&self) -> bool;

    /// Returns whether the column is currently selected for sorting, and if so,
    /// what direction.
    fn sorting_status(&self) -> SortStatus;

    /// Sets the sorting status.
    fn set_sorting_status(&mut self, sorting_status: SortStatus);

    fn display_name(&self) -> Cow<'static, str>;

    fn get_desired_width(&self) -> &DesiredColumnWidth;

    fn get_x_bounds(&self) -> Option<(u16, u16)>;

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>);
}

impl<T> TableColumn for T
where
    T: SortableColumn,
{
    fn display_name(&self) -> Cow<'static, str> {
        self.display_name()
    }

    fn get_desired_width(&self) -> &DesiredColumnWidth {
        self.get_desired_width()
    }

    fn get_x_bounds(&self) -> Option<(u16, u16)> {
        self.get_x_bounds()
    }

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>) {
        self.set_x_bounds(x_bounds)
    }
}

/// A [`SimpleSortableColumn`] represents some column in a [`SortableTextTable`].
#[derive(Debug)]
pub struct SimpleSortableColumn {
    pub shortcut: Option<(KeyEvent, String)>,
    pub default_descending: bool,
    pub internal: SimpleColumn,

    /// Whether this column is currently selected for sorting, and which direction.
    sorting_status: SortStatus,
}

impl SimpleSortableColumn {
    /// Creates a new [`SimpleSortableColumn`].
    fn new(
        full_name: Cow<'static, str>, shortcut: Option<(KeyEvent, String)>,
        default_descending: bool, desired_width: DesiredColumnWidth,
    ) -> Self {
        Self {
            shortcut,
            default_descending,
            internal: SimpleColumn::new(full_name, desired_width),
            sorting_status: SortStatus::NotSorting,
        }
    }

    /// Creates a new [`SortableColumn`] with a hard desired width. If none is specified,
    /// it will instead use the name's length + 1.
    pub fn new_hard(
        name: Cow<'static, str>, shortcut: Option<KeyEvent>, default_descending: bool,
        hard_length: Option<u16>,
    ) -> Self {
        let (full_name, shortcut) = if let Some(shortcut) = shortcut {
            let shortcut_name = get_shortcut_name(&shortcut);
            (
                format!("{}{}", name, shortcut_name).into(),
                Some((shortcut, shortcut_name)),
            )
        } else {
            (name, None)
        };
        let full_name_len = full_name.len();

        SimpleSortableColumn::new(
            full_name,
            shortcut,
            default_descending,
            DesiredColumnWidth::Hard(hard_length.unwrap_or(full_name_len as u16 + 1)),
        )
    }

    /// Creates a new [`SortableColumn`] with a flexible desired width.
    pub fn new_flex(
        name: Cow<'static, str>, shortcut: Option<KeyEvent>, default_descending: bool,
        max_percentage: f64,
    ) -> Self {
        let (full_name, shortcut) = if let Some(shortcut) = shortcut {
            let shortcut_name = get_shortcut_name(&shortcut);
            (
                format!("{}{}", name, shortcut_name).into(),
                Some((shortcut, shortcut_name)),
            )
        } else {
            (name, None)
        };
        let full_name_len = full_name.len();

        SimpleSortableColumn::new(
            full_name,
            shortcut,
            default_descending,
            DesiredColumnWidth::Flex {
                desired: full_name_len as u16,
                max_percentage,
            },
        )
    }
}

impl SortableColumn for SimpleSortableColumn {
    fn shortcut(&self) -> &Option<(KeyEvent, String)> {
        &self.shortcut
    }

    fn default_descending(&self) -> bool {
        self.default_descending
    }

    fn sorting_status(&self) -> SortStatus {
        self.sorting_status
    }

    fn set_sorting_status(&mut self, sorting_status: SortStatus) {
        self.sorting_status = sorting_status;
    }

    fn display_name(&self) -> Cow<'static, str> {
        const UP_ARROW: &str = "▲";
        const DOWN_ARROW: &str = "▼";
        format!(
            "{}{}",
            self.internal.display_name(),
            match &self.sorting_status {
                SortStatus::NotSorting => "",
                SortStatus::SortAscending => UP_ARROW,
                SortStatus::SortDescending => DOWN_ARROW,
            }
        )
        .into()
    }

    fn get_desired_width(&self) -> &DesiredColumnWidth {
        self.internal.get_desired_width()
    }

    fn get_x_bounds(&self) -> Option<(u16, u16)> {
        self.internal.get_x_bounds()
    }

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>) {
        self.internal.set_x_bounds(x_bounds)
    }
}

/// A sortable, scrollable table with columns.
pub struct SortableTextTable<S = SimpleSortableColumn>
where
    S: SortableColumn,
{
    /// Which index we're sorting by.
    sort_index: usize,

    /// The underlying [`TextTable`].
    pub table: TextTable<S>,
}

impl<S> SortableTextTable<S>
where
    S: SortableColumn,
{
    pub fn new(columns: Vec<S>) -> Self {
        let mut st = Self {
            sort_index: 0,
            table: TextTable::new(columns),
        };
        st.set_sort_index(0);
        st
    }

    pub fn default_ltr(mut self, ltr: bool) -> Self {
        self.table = self.table.default_ltr(ltr);
        self
    }

    pub fn default_sort_index(mut self, index: usize) -> Self {
        self.set_sort_index(index);
        self
    }

    pub fn current_index(&self) -> usize {
        self.table.current_index()
    }

    pub fn columns(&self) -> &[S] {
        &self.table.columns
    }

    pub fn set_column(&mut self, column: S, index: usize) {
        self.table.set_column(index, column)
    }

    fn set_sort_index(&mut self, new_index: usize) {
        if new_index == self.sort_index {
            if let Some(column) = self.table.columns.get_mut(self.sort_index) {
                match column.sorting_status() {
                    SortStatus::NotSorting => {
                        if column.default_descending() {
                            column.set_sorting_status(SortStatus::SortDescending);
                        } else {
                            column.set_sorting_status(SortStatus::SortAscending);
                        }
                    }
                    SortStatus::SortAscending => {
                        column.set_sorting_status(SortStatus::SortDescending);
                    }
                    SortStatus::SortDescending => {
                        column.set_sorting_status(SortStatus::SortAscending);
                    }
                }
            }
        } else {
            if let Some(column) = self.table.columns.get_mut(self.sort_index) {
                column.set_sorting_status(SortStatus::NotSorting);
            }

            if let Some(column) = self.table.columns.get_mut(new_index) {
                if column.default_descending() {
                    column.set_sorting_status(SortStatus::SortDescending);
                } else {
                    column.set_sorting_status(SortStatus::SortAscending);
                }
            }

            self.sort_index = new_index;
        }
    }

    /// Draws a [`Table`] given the [`TextTable`] and the given data.
    ///
    /// Note if the number of columns don't match in the [`TextTable`] and data,
    /// it will only create as many columns as it can grab data from both sources from.
    pub fn draw_tui_table<B: Backend>(
        &mut self, painter: &Painter, f: &mut tui::Frame<'_, B>, data: &TextTableData,
        block: Block<'_>, block_area: Rect, show_selected_entry: bool,
    ) {
        self.table
            .draw_tui_table(painter, f, data, block, block_area, show_selected_entry);
    }
}

impl<S> Component for SortableTextTable<S>
where
    S: SortableColumn,
{
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        for (index, column) in self.table.columns.iter().enumerate() {
            if let &Some((shortcut, _)) = column.shortcut() {
                if shortcut == event {
                    self.set_sort_index(index);
                    return EventResult::Redraw;
                }
            }
        }

        self.table.scrollable.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
            if !self.does_intersect_mouse(&event) {
                return EventResult::NoRedraw;
            }

            // Note these are representing RELATIVE coordinates! They *need* the above intersection check for validity!
            let x = event.column - self.table.bounds.left();
            let y = event.row - self.table.bounds.top();

            if y == 0 {
                for (index, column) in self.table.columns.iter().enumerate() {
                    if let Some((start, end)) = column.get_x_bounds() {
                        if x >= start && x <= end {
                            self.set_sort_index(index);
                            return EventResult::Redraw;
                        }
                    }
                }
            }

            self.table.scrollable.handle_mouse_event(event)
        } else {
            self.table.scrollable.handle_mouse_event(event)
        }
    }

    fn bounds(&self) -> Rect {
        self.table.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.table.bounds = new_bounds;
    }
}
