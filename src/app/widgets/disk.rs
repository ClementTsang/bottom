use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::{data_farmer::DataCollection, event::EventResult, sort_text_table::SimpleSortableColumn},
    canvas::Painter,
    data_conversion::convert_disk_row,
};

use super::{
    text_table::TextTableData, AppScrollWidgetState, CanvasTableWidthState, Component,
    SortableTextTable, Widget,
};

pub struct DiskWidgetState {
    pub scroll_state: AppScrollWidgetState,
    pub table_width_state: CanvasTableWidthState,
}

impl DiskWidgetState {
    pub fn init() -> Self {
        DiskWidgetState {
            scroll_state: AppScrollWidgetState::default(),
            table_width_state: CanvasTableWidthState::default(),
        }
    }
}

#[derive(Default)]
pub struct DiskState {
    pub widget_states: HashMap<u64, DiskWidgetState>,
}

impl DiskState {
    pub fn init(widget_states: HashMap<u64, DiskWidgetState>) -> Self {
        DiskState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut DiskWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&DiskWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

/// A table displaying disk data.  Essentially a wrapper around a [`TextTable`].
pub struct DiskTable {
    table: SortableTextTable,
    bounds: Rect,

    display_data: TextTableData,
}

impl Default for DiskTable {
    fn default() -> Self {
        let table = SortableTextTable::new(vec![
            SimpleSortableColumn::new_flex("Disk".into(), None, false, 0.2),
            SimpleSortableColumn::new_flex("Mount".into(), None, false, 0.2),
            SimpleSortableColumn::new_hard("Used".into(), None, false, Some(5)),
            SimpleSortableColumn::new_hard("Free".into(), None, false, Some(6)),
            SimpleSortableColumn::new_hard("Total".into(), None, false, Some(6)),
            SimpleSortableColumn::new_hard("R/s".into(), None, false, Some(7)),
            SimpleSortableColumn::new_hard("W/s".into(), None, false, Some(7)),
        ]);

        Self {
            table,
            bounds: Rect::default(),
            display_data: Default::default(),
        }
    }
}

impl Component for DiskTable {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        self.table.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        self.table.handle_mouse_event(event)
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for DiskTable {
    fn get_pretty_name(&self) -> &'static str {
        "Disk"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
    ) {
        let block = Block::default()
            .border_style(if selected {
                painter.colours.highlighted_border_style
            } else {
                painter.colours.border_style
            })
            .borders(Borders::ALL);

        self.table
            .table
            .draw_tui_table(painter, f, &self.display_data, block, area, selected);
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        self.display_data = convert_disk_row(data_collection);
    }
}
