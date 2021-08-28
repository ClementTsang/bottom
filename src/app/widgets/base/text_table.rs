use std::{
    borrow::Cow,
    cmp::{max, min},
};

use crossterm::event::{KeyEvent, MouseEvent};
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

/// A trait that must be implemented for anything using a [`TextTable`].
#[allow(unused_variables)]
pub trait TableColumn {
    fn display_name(&self) -> Cow<'static, str>;

    fn get_desired_width(&self) -> &DesiredColumnWidth;

    fn get_x_bounds(&self) -> Option<(u16, u16)>;

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>);
}

/// A [`SimpleColumn`] represents some column in a [`TextTable`].
#[derive(Debug)]
pub struct SimpleColumn {
    name: Cow<'static, str>,

    // TODO: I would remove these in the future, storing them here feels weird...
    desired_width: DesiredColumnWidth,
    x_bounds: Option<(u16, u16)>,
}

impl SimpleColumn {
    /// Creates a new [`SimpleColumn`].
    pub fn new(name: Cow<'static, str>, desired_width: DesiredColumnWidth) -> Self {
        Self {
            name,
            x_bounds: None,
            desired_width,
        }
    }

    /// Creates a new [`SimpleColumn`] with a hard desired width. If none is specified,
    /// it will instead use the name's length + 1.
    pub fn new_hard(name: Cow<'static, str>, hard_length: Option<u16>) -> Self {
        let name_len = name.len();
        SimpleColumn::new(
            name,
            DesiredColumnWidth::Hard(hard_length.unwrap_or(name_len as u16 + 1)),
        )
    }

    /// Creates a new [`SimpleColumn`] with a flexible desired width.
    pub fn new_flex(name: Cow<'static, str>, max_percentage: f64) -> Self {
        let name_len = name.len();
        SimpleColumn::new(
            name,
            DesiredColumnWidth::Flex {
                desired: name_len as u16,
                max_percentage,
            },
        )
    }
}

impl TableColumn for SimpleColumn {
    fn display_name(&self) -> Cow<'static, str> {
        self.name.clone()
    }

    fn get_desired_width(&self) -> &DesiredColumnWidth {
        &self.desired_width
    }

    fn get_x_bounds(&self) -> Option<(u16, u16)> {
        self.x_bounds
    }

    fn set_x_bounds(&mut self, x_bounds: Option<(u16, u16)>) {
        self.x_bounds = x_bounds;
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
pub struct TextTable<C = SimpleColumn>
where
    C: TableColumn,
{
    /// Controls the scrollable state.
    pub scrollable: Scrollable,

    /// The columns themselves.
    pub columns: Vec<C>,

    /// Cached column width data.
    cached_column_widths: CachedColumnWidths,

    /// Whether to show a gap between the column headers and the columns.
    pub show_gap: bool,

    /// The bounding box of the [`TextTable`].
    pub bounds: Rect, // TODO: Consider moving bounds to something else???

    /// Whether we draw columns from left-to-right.
    pub left_to_right: bool,
}

impl<C> TextTable<C>
where
    C: TableColumn,
{
    pub fn new(columns: Vec<C>) -> Self {
        Self {
            scrollable: Scrollable::new(0),
            columns,
            cached_column_widths: CachedColumnWidths::Uncached,
            show_gap: true,
            bounds: Rect::default(),
            left_to_right: true,
        }
    }

    pub fn default_ltr(mut self, ltr: bool) -> Self {
        self.left_to_right = ltr;
        self
    }

    pub fn try_show_gap(mut self, show_gap: bool) -> Self {
        self.show_gap = show_gap;
        self
    }

    pub fn displayed_column_names(&self) -> Vec<Cow<'static, str>> {
        self.columns
            .iter()
            .map(|column| column.display_name())
            .collect()
    }

    pub fn update_num_items(&mut self, num_items: usize) {
        self.scrollable.update_num_items(num_items);
    }

    pub fn update_single_column(&mut self, index: usize, column: C) {
        if let Some(c) = self.columns.get_mut(index) {
            *c = column;
        }
    }

    pub fn get_desired_column_widths(
        columns: &[C], data: &[Vec<(Cow<'static, str>, Option<Cow<'static, str>>)>],
    ) -> Vec<DesiredColumnWidth> {
        columns
            .iter()
            .enumerate()
            .map(|(column_index, c)| match c.get_desired_width() {
                DesiredColumnWidth::Hard(width) => {
                    let max_len = data
                        .iter()
                        .filter_map(|c| c.get(column_index))
                        .max_by(|(x, short_x), (y, short_y)| {
                            let x = if let Some(short_x) = short_x {
                                short_x
                            } else {
                                x
                            };

                            let y = if let Some(short_y) = short_y {
                                short_y
                            } else {
                                y
                            };

                            x.len().cmp(&y.len())
                        })
                        .map(|(s, _)| s.len())
                        .unwrap_or(0) as u16;

                    DesiredColumnWidth::Hard(max(max_len, *width))
                }
                DesiredColumnWidth::Flex {
                    desired: _,
                    max_percentage: _,
                } => c.get_desired_width().clone(),
            })
            .collect::<Vec<_>>()
    }

    fn get_cache(
        &mut self, area: Rect, data: &[Vec<(Cow<'static, str>, Option<Cow<'static, str>>)>],
    ) -> Vec<u16> {
        fn calculate_column_widths(
            left_to_right: bool, mut desired_widths: Vec<DesiredColumnWidth>, total_width: u16,
        ) -> Vec<u16> {
            let mut total_width_left = total_width;
            if !left_to_right {
                desired_widths.reverse();
            }

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

            column_widths
        }

        // If empty, do NOT save the cache!  We have to get it again when it updates.
        if data.is_empty() {
            vec![0; self.columns.len()]
        } else {
            let was_cached: bool;
            let column_widths = match &mut self.cached_column_widths {
                CachedColumnWidths::Uncached => {
                    // Always recalculate.
                    was_cached = false;
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
                        was_cached = false;
                        let desired_widths =
                            TextTable::get_desired_column_widths(&self.columns, data);
                        let calculated_widths =
                            calculate_column_widths(self.left_to_right, desired_widths, area.width);
                        *cached_area = area;
                        *cached_data = calculated_widths.clone();

                        calculated_widths
                    } else {
                        was_cached = true;
                        cached_data.clone()
                    }
                }
            };

            if !was_cached {
                let mut column_start = 0;
                for (column, width) in self.columns.iter_mut().zip(&column_widths) {
                    let column_end = column_start + *width;
                    column.set_x_bounds(Some((column_start, column_end)));
                    column_start = column_end + 1;
                }
            }

            column_widths
        }
    }

    /// Creates a [`Table`] given the [`TextTable`] and the given data, along with its
    /// widths (because for some reason a [`Table`] only borrows the constraints...?)
    /// and [`TableState`] (so we know which row is selected).
    ///
    /// Note if the number of columns don't match in the [`TextTable`] and data,
    /// it will only create as many columns as it can grab data from both sources from.
    pub fn create_draw_table(
        &mut self, painter: &Painter, data: &[Vec<(Cow<'static, str>, Option<Cow<'static, str>>)>],
        area: Rect,
    ) -> (Table<'_>, Vec<Constraint>, TableState) {
        use tui::widgets::Row;

        let table_gap = if !self.show_gap || area.height < TABLE_GAP_HEIGHT_LIMIT {
            0
        } else {
            1
        };

        self.update_num_items(data.len());
        self.set_bounds(area);
        let table_extras = 1 + table_gap;
        let scrollable_height = area.height.saturating_sub(table_extras);
        self.scrollable.set_bounds(Rect::new(
            area.x,
            area.y + table_extras,
            area.width,
            scrollable_height,
        ));

        // Calculate widths first, since we need them later.
        let calculated_widths = self.get_cache(area, data);
        let widths = calculated_widths
            .iter()
            .map(|column| Constraint::Length(*column))
            .collect::<Vec<_>>();

        // Then calculate rows. We truncate the amount of data read based on height,
        // as well as truncating some entries based on available width.
        let data_slice = {
            let start = self.scrollable.get_list_start(scrollable_height as usize);
            let end = std::cmp::min(
                self.scrollable.num_items(),
                start + scrollable_height as usize,
            );
            &data[start..end]
        };
        let rows = data_slice.iter().map(|row| {
            Row::new(
                row.iter()
                    .zip(&calculated_widths)
                    .map(|((text, shrunk_text), width)| {
                        let width = *width as usize;
                        let graphemes = UnicodeSegmentation::graphemes(text.as_ref(), true)
                            .collect::<Vec<&str>>();
                        let grapheme_width = graphemes.len();
                        if width < grapheme_width && width > 1 {
                            if let Some(shrunk_text) = shrunk_text {
                                Text::raw(shrunk_text.clone())
                            } else {
                                Text::raw(format!("{}…", graphemes[..(width - 1)].concat()))
                            }
                        } else {
                            Text::raw(text.to_owned())
                        }
                    }),
            )
        });

        // Now build up our headers...
        let header = Row::new(self.displayed_column_names())
            .style(painter.colours.table_header_style)
            .bottom_margin(table_gap);

        // And return tui-rs's [`TableState`].
        let tui_state = self.scrollable.tui_state();

        (
            Table::new(rows)
                .header(header)
                .style(painter.colours.text_style),
            widths,
            tui_state,
        )
    }

    /// Creates a [`Table`] representing the sort list.
    pub fn create_sort_list(&mut self) -> (Table<'_>, TableState) {
        todo!()
    }
}

impl<C> Component for TextTable<C>
where
    C: TableColumn,
{
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        self.scrollable.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        self.scrollable.handle_mouse_event(event)
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}
