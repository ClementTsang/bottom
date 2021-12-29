use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    app::{
        event::{ComponentEventResult, ReturnSignal},
        widgets::tui_stuff::BlockBuilder,
        Component, TextTable,
    },
    canvas::Painter,
};

use super::text_table::{DesiredColumnWidth, SimpleColumn, TableColumn, TextTableDataRef};

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

    format!("({}{})", modifier, key)
}

#[derive(Copy, Clone, Debug)]
pub enum SortStatus {
    NotSorting,
    SortAscending,
    SortDescending,
}

/// A trait for sortable columns.
pub trait SortableColumn {
    /// Returns the original name of the column.
    fn original_name(&self) -> &Cow<'static, str>;

    /// Returns the shortcut for the column, if it exists.
    fn shortcut(&self) -> &Option<(KeyEvent, String)>;

    /// Returns whether the column defaults to sorting in descending order or not.
    fn default_descending(&self) -> bool;

    /// Returns whether the column is currently selected for sorting, and if so,
    /// what direction.
    fn sorting_status(&self) -> SortStatus;

    /// Sets the sorting status.
    fn set_sorting_status(&mut self, sorting_status: SortStatus);

    // ----- The following are required since SortableColumn implements TableColumn -----

    /// Returns the displayed name on the column when drawing.
    fn display_name(&self) -> Cow<'static, str>;

    /// Returns the desired width of the column when drawing.
    fn get_desired_width(&self) -> &DesiredColumnWidth;

    /// Returns the x bounds of a column. The y is assumed to be 0, relative to the table..
    fn get_x_bounds(&self) -> Option<(u16, u16)>;

    /// Sets the x bounds of a column.
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
    original_name: Cow<'static, str>,
    pub shortcut: Option<(KeyEvent, String)>,
    pub default_descending: bool,

    pub internal: SimpleColumn,

    /// Whether this column is currently selected for sorting, and which direction.
    sorting_status: SortStatus,
}

impl SimpleSortableColumn {
    /// Creates a new [`SimpleSortableColumn`].
    fn new(
        original_name: Cow<'static, str>, full_name: Cow<'static, str>,
        shortcut: Option<(KeyEvent, String)>, default_descending: bool,
        desired_width: DesiredColumnWidth,
    ) -> Self {
        Self {
            original_name,
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
            (name.clone(), None)
        };
        let full_name_len = full_name.len();

        SimpleSortableColumn::new(
            name,
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
            (name.clone(), None)
        };
        let full_name_len = full_name.len();

        SimpleSortableColumn::new(
            name,
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
    fn original_name(&self) -> &Cow<'static, str> {
        &self.original_name
    }

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
    /// Creates a new [`SortableTextTable`]. Note that `columns` cannot be empty.
    pub fn new(columns: Vec<S>) -> Self {
        let mut st = Self {
            sort_index: 0,
            table: TextTable::new(columns),
        };
        st.set_sort_index(0);
        st
    }

    pub fn try_show_gap(mut self, show_gap: bool) -> Self {
        self.table = self.table.try_show_gap(show_gap);
        self
    }

    pub fn default_sort_index(mut self, index: usize) -> Self {
        self.set_sort_index(index);
        self
    }

    pub fn current_scroll_index(&self) -> usize {
        self.table.current_scroll_index()
    }

    /// Returns a reference to the current column the table is sorting by.
    pub fn current_sorting_column(&self) -> &S {
        &self.table.columns[self.sort_index]
    }

    /// Returns a mutable reference to the current column the table is sorting by.
    pub fn current_mut_sorting_column(&mut self) -> &mut S {
        &mut self.table.columns[self.sort_index]
    }

    /// Returns the current column index the table is sorting by.
    pub fn current_sorting_column_index(&self) -> usize {
        self.sort_index
    }

    pub fn columns(&self) -> &[S] {
        &self.table.columns
    }

    pub fn reverse_current_sort(&mut self) {
        if self.is_sort_descending() {
            self.table.columns[self.sort_index].set_sorting_status(SortStatus::SortAscending);
        } else {
            self.table.columns[self.sort_index].set_sorting_status(SortStatus::SortDescending);
        }
    }

    pub fn is_sort_descending(&self) -> bool {
        matches!(
            self.table.columns[self.sort_index].sorting_status(),
            SortStatus::SortDescending
        )
    }

    pub fn set_column(&mut self, mut column: S, index: usize) {
        if let Some(old_column) = self.table.columns().get(index) {
            column.set_sorting_status(old_column.sorting_status());
        }
        self.table.set_column(index, column);
    }

    pub fn add_column(&mut self, column: S, index: usize) {
        self.table.add_column(index, column);
    }

    pub fn remove_column(&mut self, index: usize, new_sort_index: Option<usize>) {
        self.table.remove_column(index);

        // Reset the sort index either a supplied one or a new one if needed.
        if index == self.sort_index {
            if let Some(new_sort_index) = new_sort_index {
                self.set_sort_index(new_sort_index);
            } else {
                self.set_sort_index(0);
            }
        }
    }

    pub fn set_sort_index(&mut self, new_index: usize) {
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

    pub fn invalidate_cached_columns(&mut self) {
        self.table.invalidate_cached_columns();
    }

    /// Draws a [`tui::widgets::Table`] on screen.
    ///
    /// Note if the number of columns don't match in the [`SortableTextTable`] and data,
    /// it will only create as many columns as it can grab data from both sources from.
    pub fn draw_tui_table<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, data: &TextTableDataRef,
        block: BlockBuilder, block_area: Rect, show_selected_entry: bool,
        show_scroll_position: bool,
    ) {
        self.table.draw_tui_table(
            painter,
            f,
            data,
            block,
            block_area,
            show_selected_entry,
            show_scroll_position,
        );
    }
}

impl<S> Component for SortableTextTable<S>
where
    S: SortableColumn,
{
    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        for (index, column) in self.table.columns.iter().enumerate() {
            if let Some((shortcut, _)) = *column.shortcut() {
                if shortcut == event {
                    self.set_sort_index(index);
                    return ComponentEventResult::Signal(ReturnSignal::Update);
                }
            }
        }

        self.table.scrollable.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
            if !self.does_bounds_intersect_mouse(&event) {
                return ComponentEventResult::NoRedraw;
            }

            // Note these are representing RELATIVE coordinates! They *need* the above intersection check for validity!
            let x = event.column - self.table.bounds.left();
            let y = event.row - self.table.bounds.top();

            if y == 0 {
                for (index, column) in self.table.columns.iter().enumerate() {
                    if let Some((start, end)) = column.get_x_bounds() {
                        if x >= start && x <= end {
                            self.set_sort_index(index);
                            return ComponentEventResult::Signal(ReturnSignal::Update);
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
        self.table.bounds()
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.table.set_bounds(new_bounds)
    }

    fn border_bounds(&self) -> Rect {
        self.table.border_bounds()
    }

    fn set_border_bounds(&mut self, new_bounds: Rect) {
        self.table.set_border_bounds(new_bounds)
    }
}
