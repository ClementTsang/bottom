use std::{
    borrow::Cow,
    cmp::{max, min},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use tui::{
    layout::{Constraint, Rect},
    text::Text,
    widgets::{Table, TableState},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::{event::EventResult, Component, Scrollable},
    canvas::Painter,
    constants::TABLE_GAP_HEIGHT_LIMIT,
};

/// Represents the desired widths a column tries to have.
#[derive(Clone, Debug)]
pub enum DesiredColumnWidth {
    Hard(u16),
    Flex { desired: u16, max_percentage: f64 },
}

/// A [`Column`] represents some column in a [`TextTable`].
#[derive(Debug)]
pub struct Column {
    pub name: &'static str,
    pub shortcut: Option<(KeyEvent, String)>,
    pub default_descending: bool,

    // TODO: I would remove these in the future, storing them here feels weird...
    pub desired_width: DesiredColumnWidth,
    pub x_bounds: (u16, u16),
}

impl Column {
    /// Creates a new [`Column`].
    pub fn new(
        name: &'static str, shortcut: Option<KeyEvent>, default_descending: bool,
        desired_width: DesiredColumnWidth,
    ) -> Self {
        Self {
            name,
            x_bounds: (0, 0),
            shortcut: shortcut.map(|e| {
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

                let shortcut_name = format!("({}{})", modifier, key);

                (e, shortcut_name)
            }),
            default_descending,
            desired_width,
        }
    }

    /// Creates a new [`Column`] with a hard desired width. If none is specified,
    /// it will instead use the name's length.
    pub fn new_hard(
        name: &'static str, shortcut: Option<KeyEvent>, default_descending: bool,
        hard_length: Option<u16>,
    ) -> Self {
        Column::new(
            name,
            shortcut,
            default_descending,
            DesiredColumnWidth::Hard(hard_length.unwrap_or(name.len() as u16)),
        )
    }

    /// Creates a new [`Column`] with a flexible desired width.
    pub fn new_flex(
        name: &'static str, shortcut: Option<KeyEvent>, default_descending: bool,
        max_percentage: f64,
    ) -> Self {
        Column::new(
            name,
            shortcut,
            default_descending,
            DesiredColumnWidth::Flex {
                desired: name.len() as u16,
                max_percentage,
            },
        )
    }
}

#[derive(Clone)]
enum CachedColumnWidths {
    Uncached,
    Cached {
        cached_area: Rect,
        cached_data: Vec<u16>,
    },
}

/// A sortable, scrollable table with columns.
pub struct TextTable {
    /// Controls the scrollable state.
    scrollable: Scrollable,

    /// The columns themselves.
    columns: Vec<Column>,

    /// Cached column width data.
    cached_column_widths: CachedColumnWidths,

    /// Whether to show a gap between the column headers and the columns.
    show_gap: bool,

    /// The bounding box of the [`TextTable`].
    bounds: Rect, // TODO: Consider moving bounds to something else???

    /// Which index we're sorting by.
    sort_index: usize,

    /// Whether we're sorting by ascending order.
    sort_ascending: bool,

    /// Whether we draw columns from left-to-right.
    left_to_right: bool,
}

impl TextTable {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            scrollable: Scrollable::new(0),
            columns,
            cached_column_widths: CachedColumnWidths::Uncached,
            show_gap: true,
            bounds: Rect::default(),
            sort_index: 0,
            sort_ascending: true,
            left_to_right: true,
        }
    }

    pub fn left_to_right(mut self, ltr: bool) -> Self {
        self.left_to_right = ltr;
        self
    }

    pub fn try_show_gap(mut self, show_gap: bool) -> Self {
        self.show_gap = show_gap;
        self
    }

    pub fn sort_index(mut self, sort_index: usize) -> Self {
        self.sort_index = sort_index;
        self
    }

    pub fn column_names(&self) -> Vec<&'static str> {
        self.columns.iter().map(|column| column.name).collect()
    }

    pub fn sorted_column_names(&self) -> Vec<String> {
        const UP_ARROW: char = '▲';
        const DOWN_ARROW: char = '▼';

        self.columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                if index == self.sort_index {
                    format!(
                        "{}{}{}",
                        column.name,
                        if let Some(shortcut) = &column.shortcut {
                            shortcut.1.as_str()
                        } else {
                            ""
                        },
                        if self.sort_ascending {
                            UP_ARROW
                        } else {
                            DOWN_ARROW
                        }
                    )
                } else {
                    format!(
                        "{}{}",
                        column.name,
                        if let Some(shortcut) = &column.shortcut {
                            shortcut.1.as_str()
                        } else {
                            ""
                        }
                    )
                }
            })
            .collect()
    }

    pub fn update_num_items(&mut self, num_items: usize) {
        self.scrollable.update_num_items(num_items);
    }

    pub fn update_a_column(&mut self, index: usize, column: Column) {
        if let Some(c) = self.columns.get_mut(index) {
            *c = column;
        }
    }

    pub fn get_desired_column_widths(
        columns: &[Column], data: &[Vec<String>],
    ) -> Vec<DesiredColumnWidth> {
        columns
            .iter()
            .enumerate()
            .map(|(column_index, c)| match c.desired_width {
                DesiredColumnWidth::Hard(width) => {
                    let max_len = data
                        .iter()
                        .filter_map(|c| c.get(column_index))
                        .max_by(|x, y| x.len().cmp(&y.len()))
                        .map(|s| s.len())
                        .unwrap_or(0) as u16;

                    DesiredColumnWidth::Hard(max(max_len, width))
                }
                DesiredColumnWidth::Flex {
                    desired: _,
                    max_percentage: _,
                } => c.desired_width.clone(),
            })
            .collect::<Vec<_>>()
    }

    fn get_cache(&mut self, area: Rect, data: &[Vec<String>]) -> Vec<u16> {
        fn calculate_column_widths(
            left_to_right: bool, mut desired_widths: Vec<DesiredColumnWidth>, total_width: u16,
        ) -> Vec<u16> {
            debug!("OG desired widths: {:?}", desired_widths);
            let mut total_width_left = total_width;
            if !left_to_right {
                desired_widths.reverse();
            }
            debug!("Desired widths: {:?}", desired_widths);

            let mut column_widths: Vec<u16> = Vec::with_capacity(desired_widths.len());
            for width in desired_widths {
                match width {
                    DesiredColumnWidth::Hard(width) => {
                        if width > total_width_left {
                            break;
                        } else {
                            column_widths.push(width);
                            total_width_left = total_width_left.saturating_sub(width + 1);
                        }
                    }
                    DesiredColumnWidth::Flex {
                        desired,
                        max_percentage,
                    } => {
                        if desired > total_width_left {
                            break;
                        } else {
                            let calculated_width = min(
                                max(desired, (max_percentage * total_width as f64).ceil() as u16),
                                total_width_left,
                            );

                            column_widths.push(calculated_width);
                            total_width_left =
                                total_width_left.saturating_sub(calculated_width + 1);
                        }
                    }
                }
            }
            debug!("Initial column widths: {:?}", column_widths);

            if !column_widths.is_empty() {
                let amount_per_slot = total_width_left / column_widths.len() as u16;
                total_width_left %= column_widths.len() as u16;
                for (itx, width) in column_widths.iter_mut().enumerate() {
                    if (itx as u16) < total_width_left {
                        *width += amount_per_slot + 1;
                    } else {
                        *width += amount_per_slot;
                    }
                }

                if !left_to_right {
                    column_widths.reverse();
                }
            }

            debug!("Column widths: {:?}", column_widths);

            column_widths
        }

        // If empty, do NOT save the cache!  We have to get it again when it updates.
        if data.is_empty() {
            vec![0; self.columns.len()]
        } else {
            match &mut self.cached_column_widths {
                CachedColumnWidths::Uncached => {
                    // Always recalculate.
                    let desired_widths = TextTable::get_desired_column_widths(&self.columns, data);
                    let calculated_widths =
                        calculate_column_widths(self.left_to_right, desired_widths, area.width);
                    self.cached_column_widths = CachedColumnWidths::Cached {
                        cached_area: area,
                        cached_data: calculated_widths.clone(),
                    };

                    calculated_widths
                }
                CachedColumnWidths::Cached {
                    cached_area,
                    cached_data,
                } => {
                    if *cached_area != area {
                        // Recalculate!
                        let desired_widths =
                            TextTable::get_desired_column_widths(&self.columns, data);
                        let calculated_widths =
                            calculate_column_widths(self.left_to_right, desired_widths, area.width);
                        *cached_area = area;
                        *cached_data = calculated_widths.clone();

                        calculated_widths
                    } else {
                        cached_data.clone()
                    }
                }
            }
        }
    }

    /// Creates a [`Table`] given the [`TextTable`] and the given data, along with its
    /// widths (because for some reason a [`Table`] only borrows the constraints...?)
    /// and [`TableState`] (so we know which row is selected).
    ///
    /// Note if the number of columns don't match in the [`TextTable`] and data,
    /// it will only create as many columns as it can grab data from both sources from.
    pub fn create_draw_table(
        &mut self, painter: &Painter, data: &[Vec<String>], area: Rect,
    ) -> (Table<'_>, Vec<Constraint>, TableState) {
        // TODO: Change data: &[Vec<String>] to &[Vec<Cow<'static, str>>]
        use tui::widgets::Row;

        let table_gap = if !self.show_gap || area.height < TABLE_GAP_HEIGHT_LIMIT {
            0
        } else {
            1
        };

        self.set_bounds(area);
        let scrollable_height = area.height.saturating_sub(1 + table_gap);
        self.scrollable.set_bounds(Rect::new(
            area.x,
            area.y + 1 + table_gap,
            area.width,
            scrollable_height,
        ));
        self.update_num_items(data.len());

        // Calculate widths first, since we need them later.
        let calculated_widths = self.get_cache(area, data);
        let widths = calculated_widths
            .iter()
            .map(|column| Constraint::Length(*column))
            .collect::<Vec<_>>();

        // Then calculate rows. We truncate the amount of data read based on height,
        // as well as truncating some entries based on available width.
        let data_slice = {
            let start = self.scrollable.index();
            let end = std::cmp::min(
                self.scrollable.num_items(),
                start + scrollable_height as usize,
            );
            &data[start..end]
        };
        let rows = data_slice.iter().map(|row| {
            Row::new(row.iter().zip(&calculated_widths).map(|(cell, width)| {
                let width = *width as usize;
                let graphemes =
                    UnicodeSegmentation::graphemes(cell.as_str(), true).collect::<Vec<&str>>();
                let grapheme_width = graphemes.len();
                if width < grapheme_width && width > 1 {
                    Text::raw(format!("{}…", graphemes[..(width - 1)].concat()))
                } else {
                    Text::raw(cell.to_owned())
                }
            }))
        });

        // Now build up our headers...
        let header = Row::new(self.sorted_column_names())
            .style(painter.colours.table_header_style)
            .bottom_margin(table_gap);

        // And return tui-rs's [`TableState`].
        let mut tui_state = TableState::default();
        tui_state.select(Some(self.scrollable.index()));

        (
            Table::new(rows)
                .header(header)
                .style(painter.colours.text_style),
            widths,
            tui_state,
        )
    }
}

impl Component for TextTable {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        for (index, column) in self.columns.iter().enumerate() {
            if let Some((shortcut, _)) = column.shortcut {
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
