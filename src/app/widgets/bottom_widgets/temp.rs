use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::{
        data_farmer::DataCollection, data_harvester::temperature::TemperatureType,
        event::WidgetEventResult, sort_text_table::SimpleSortableColumn, text_table::TextTableData,
        AppScrollWidgetState, CanvasTableWidthState, Component, TextTable, Widget,
    },
    canvas::Painter,
    data_conversion::convert_temp_row,
    options::layout_options::LayoutRule,
};

pub struct TempWidgetState {
    pub scroll_state: AppScrollWidgetState,
    pub table_width_state: CanvasTableWidthState,
}

impl TempWidgetState {
    pub fn init() -> Self {
        TempWidgetState {
            scroll_state: AppScrollWidgetState::default(),
            table_width_state: CanvasTableWidthState::default(),
        }
    }
}

#[derive(Default)]
pub struct TempState {
    pub widget_states: HashMap<u64, TempWidgetState>,
}

impl TempState {
    pub fn init(widget_states: HashMap<u64, TempWidgetState>) -> Self {
        TempState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut TempWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&TempWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

/// A table displaying disk data..
pub struct TempTable {
    table: TextTable<SimpleSortableColumn>,
    bounds: Rect,
    display_data: TextTableData,
    temp_type: TemperatureType,
    width: LayoutRule,
    height: LayoutRule,
    block_border: Borders,
}

impl Default for TempTable {
    fn default() -> Self {
        let table = TextTable::new(vec![
            SimpleSortableColumn::new_flex("Sensor".into(), None, false, 0.8),
            SimpleSortableColumn::new_hard("Temp".into(), None, false, Some(5)),
        ])
        .default_ltr(false);

        Self {
            table,
            bounds: Rect::default(),
            display_data: Default::default(),
            temp_type: TemperatureType::default(),
            width: LayoutRule::default(),
            height: LayoutRule::default(),
            block_border: Borders::ALL,
        }
    }
}

impl TempTable {
    /// Sets the [`TemperatureType`] for the [`TempTable`].
    pub fn set_temp_type(mut self, temp_type: TemperatureType) -> Self {
        self.temp_type = temp_type;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: LayoutRule) -> Self {
        self.height = height;
        self
    }

    /// Sets the block border style.
    pub fn basic_mode(mut self, basic_mode: bool) -> Self {
        if basic_mode {
            self.block_border = *crate::constants::SIDE_BORDERS;
        }

        self
    }
}

impl Component for TempTable {
    fn handle_key_event(&mut self, event: KeyEvent) -> WidgetEventResult {
        self.table.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> WidgetEventResult {
        self.table.handle_mouse_event(event)
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for TempTable {
    fn get_pretty_name(&self) -> &'static str {
        "Temperature"
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
            .borders(self.block_border.clone()); // TODO: Also do the scrolling indicator!

        self.table
            .draw_tui_table(painter, f, &self.display_data, block, area, selected);
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        self.display_data = convert_temp_row(data_collection, &self.temp_type);
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }
}
