use std::{borrow::Cow, panic::Location};

use crate::tuine::{StatefulTemplate, ViewContext};

use super::{DataRow, SortType, StyleSheet, TextColumn, TextTable, TextTableState};

pub struct TextTableBuilder<Message> {
    column_widths: Vec<u16>,
    columns: Vec<TextColumn>,
    show_gap: bool,
    show_selected_entry: bool,
    rows: Vec<DataRow>,
    style_sheet: StyleSheet,
    sort: SortType,
    table_gap: u16,
    on_select: Option<Box<dyn Fn(usize) -> Message>>,
    on_selected_click: Option<Box<dyn Fn(usize) -> Message>>,
}

impl<Message> TextTableBuilder<Message> {
    pub fn new<S: Into<Cow<'static, str>>>(columns: Vec<S>) -> Self {
        Self {
            column_widths: vec![0; columns.len()],
            columns: columns
                .into_iter()
                .map(|name| TextColumn::new(name))
                .collect(),
            show_gap: true,
            show_selected_entry: true,
            rows: Vec::default(),
            style_sheet: StyleSheet::default(),
            sort: SortType::Unsortable,
            table_gap: 0,
            on_select: None,
            on_selected_click: None,
        }
    }

    /// Sets the row to display in the table.
    ///
    /// Defaults to displaying no data if not set.
    pub fn rows(mut self, rows: Vec<DataRow>) -> Self {
        self.rows = rows;
        self
    }

    /// Adds a new row.
    pub fn row(mut self, row: DataRow) -> Self {
        self.rows.push(row);
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

    /// How the table should sort data on first initialization, if at all.
    ///
    /// Defaults to [`SortType::Unsortable`] if not set.
    pub fn default_sort(mut self, sort: SortType) -> Self {
        self.sort = sort;
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

    fn try_sort_data(&mut self, sort_type: SortType) {
        use std::cmp::Ordering;

        // TODO: We can avoid some annoying checks by using const generics - this is waiting on
        // the const_generics_defaults feature, landing in 1.59, however!

        fn sort_cmp(column: usize, a: &DataRow, b: &DataRow) -> Ordering {
            match (a.get(column), b.get(column)) {
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_a), None) => Ordering::Greater,
                (None, Some(_b)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            }
        }

        match sort_type {
            SortType::Ascending(column) => {
                self.rows.sort_by(|a, b| sort_cmp(column, a, b));
            }
            SortType::Descending(column) => {
                self.rows.sort_by(|a, b| sort_cmp(column, a, b));
                self.rows.reverse();
            }
            SortType::Unsortable => {}
        }
    }
}

impl<Message> StatefulTemplate<Message> for TextTableBuilder<Message> {
    type Component = TextTable<Message>;
    type ComponentState = TextTableState;

    #[track_caller]
    fn build(mut self, ctx: &mut ViewContext<'_>) -> Self::Component {
        let sort = self.sort;
        let (key, state) = ctx.register_and_mut_state_with_default::<_, Self::ComponentState, _>(
            Location::caller(),
            || TextTableState {
                scroll: Default::default(),
                sort,
            },
        );

        state.scroll.set_num_items(self.rows.len());
        self.try_sort_data(state.sort);

        TextTable {
            key,
            column_widths: self.column_widths,
            columns: self.columns,
            show_gap: self.show_gap,
            show_selected_entry: self.show_selected_entry,
            rows: self.rows,
            style_sheet: self.style_sheet,
            table_gap: self.table_gap,
            on_select: self.on_select,
            on_selected_click: self.on_selected_click,
        }
    }
}
