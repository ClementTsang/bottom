use std::{
    cmp::{max, min},
    collections::HashMap,
};

use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Borders, Paragraph, Tabs},
    Frame,
};

use crate::{
    app::{
        data_farmer::DataCollection, does_bound_intersect_coordinate, event::WidgetEventResult,
        widgets::tui_widgets::PipeGauge, Component, Widget,
    },
    canvas::Painter,
    constants::TABLE_GAP_HEIGHT_LIMIT,
    data_conversion::{convert_battery_harvest, ConvertedBatteryData},
    options::layout_options::LayoutRule,
};

#[derive(Default)]
pub struct BatteryWidgetState {
    pub currently_selected_battery_index: usize,
    pub tab_click_locs: Option<Vec<((u16, u16), (u16, u16))>>,
}

#[derive(Default)]
pub struct BatteryState {
    pub widget_states: HashMap<u64, BatteryWidgetState>,
}

impl BatteryState {
    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut BatteryWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }
}

/// A table displaying battery information on a per-battery basis.
pub struct BatteryTable {
    bounds: Rect,
    selected_index: usize,
    battery_data: Vec<ConvertedBatteryData>,
    width: LayoutRule,
    height: LayoutRule,
    block_border: Borders,
    tab_bounds: Vec<Rect>,
}

impl Default for BatteryTable {
    fn default() -> Self {
        Self {
            bounds: Default::default(),
            selected_index: 0,
            battery_data: Default::default(),
            width: LayoutRule::default(),
            height: LayoutRule::default(),
            block_border: Borders::ALL,
            tab_bounds: Default::default(),
        }
    }
}

impl BatteryTable {
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

    /// Returns the index of the currently selected battery.
    pub fn index(&self) -> usize {
        self.selected_index
    }

    fn increment_index(&mut self) {
        if self.selected_index + 1 < self.battery_data.len() {
            self.selected_index += 1;
        }
    }

    fn decrement_index(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Sets the block border style.
    pub fn basic_mode(mut self, basic_mode: bool) -> Self {
        if basic_mode {
            self.block_border = *crate::constants::SIDE_BORDERS;
        }

        self
    }
}

impl Component for BatteryTable {
    fn bounds(&self) -> tui::layout::Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: tui::layout::Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> WidgetEventResult {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Left => {
                    let current_index = self.selected_index;
                    self.decrement_index();
                    if current_index == self.selected_index {
                        WidgetEventResult::NoRedraw
                    } else {
                        WidgetEventResult::Redraw
                    }
                }
                KeyCode::Right => {
                    let current_index = self.selected_index;
                    self.increment_index();
                    if current_index == self.selected_index {
                        WidgetEventResult::NoRedraw
                    } else {
                        WidgetEventResult::Redraw
                    }
                }
                _ => WidgetEventResult::NoRedraw,
            }
        } else {
            WidgetEventResult::NoRedraw
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> WidgetEventResult {
        for (itx, bound) in self.tab_bounds.iter().enumerate() {
            if does_bound_intersect_coordinate(event.column, event.row, *bound)
                && itx < self.battery_data.len()
            {
                self.selected_index = itx;
                return WidgetEventResult::Redraw;
            }
        }
        WidgetEventResult::NoRedraw
    }
}

impl Widget for BatteryTable {
    fn get_pretty_name(&self) -> &'static str {
        "Battery"
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        self.battery_data = convert_battery_harvest(data_collection);
        if self.battery_data.len() <= self.selected_index {
            self.selected_index = self.battery_data.len().saturating_sub(1);
        }
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
    ) {
        let block = self.block(painter, selected, self.block_border);

        let inner_area = block.inner(area);
        const CONSTRAINTS: [Constraint; 2] = [Constraint::Length(1), Constraint::Min(0)];
        let split_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(CONSTRAINTS)
            .split(inner_area);

        if self.battery_data.is_empty() {
            f.render_widget(
                Paragraph::new("No batteries found").style(painter.colours.text_style),
                split_area[0],
            );
        } else {
            let tab_area = Rect::new(
                split_area[0].x.saturating_sub(1),
                split_area[0].y,
                split_area[0].width,
                split_area[0].height,
            );
            let data_area =
                if inner_area.height >= TABLE_GAP_HEIGHT_LIMIT && split_area[1].height > 0 {
                    Rect::new(
                        split_area[1].x,
                        split_area[1].y + 1,
                        split_area[1].width,
                        split_area[1].height - 1,
                    )
                } else {
                    split_area[1]
                };

            let battery_tab_names = self
                .battery_data
                .iter()
                .map(|d| Spans::from(d.battery_name.as_str()))
                .collect::<Vec<_>>();
            let mut start_x_offset = tab_area.x + 1;
            self.tab_bounds = battery_tab_names
                .iter()
                .map(|name| {
                    let length = name.width() as u16;
                    let start = start_x_offset;
                    start_x_offset += length;
                    start_x_offset += 3;

                    Rect::new(start, tab_area.y, length, 1)
                })
                .collect();
            f.render_widget(
                Tabs::new(battery_tab_names)
                    .divider(tui::symbols::line::VERTICAL)
                    .style(painter.colours.text_style)
                    .highlight_style(painter.colours.currently_selected_text_style)
                    .select(self.selected_index),
                tab_area,
            );

            if let Some(battery_details) = self.battery_data.get(self.selected_index) {
                let labels = vec![
                    Spans::from(Span::styled("Charge %", painter.colours.text_style)),
                    Spans::from(Span::styled("Consumption", painter.colours.text_style)),
                    match &battery_details.charge_times {
                        crate::data_conversion::BatteryDuration::Charging { .. } => {
                            Spans::from(Span::styled("Time to full", painter.colours.text_style))
                        }
                        crate::data_conversion::BatteryDuration::Discharging { .. } => {
                            Spans::from(Span::styled("Time to empty", painter.colours.text_style))
                        }
                        crate::data_conversion::BatteryDuration::Neither => Spans::from(
                            Span::styled("Time to full/empty", painter.colours.text_style),
                        ),
                    },
                    Spans::from(Span::styled("Health %", painter.colours.text_style)),
                ];

                let data_constraints = if let Some(len) = labels.iter().map(|s| s.width()).max() {
                    [
                        Constraint::Length(min(
                            max(len as u16 + 2, data_area.width / 2),
                            data_area.width,
                        )),
                        Constraint::Min(0),
                    ]
                } else {
                    [Constraint::Ratio(1, 2); 2]
                };
                const VALUE_CONSTRAINTS: [Constraint; 2] =
                    [Constraint::Length(1), Constraint::Min(0)];
                let details_split_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(data_constraints)
                    .split(data_area);
                let per_detail_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(VALUE_CONSTRAINTS)
                    .split(details_split_area[1]);

                f.render_widget(Paragraph::new(labels), details_split_area[0]);
                f.render_widget(
                    PipeGauge::default()
                        .end_label(format!(
                            "{:3.0}%",
                            battery_details.charge_percentage.round()
                        ))
                        .ratio(battery_details.charge_percentage / 100.0)
                        .style(if battery_details.charge_percentage < 10.0 {
                            painter.colours.low_battery_colour
                        } else if battery_details.charge_percentage < 50.0 {
                            painter.colours.medium_battery_colour
                        } else {
                            painter.colours.high_battery_colour
                        }),
                    per_detail_area[0],
                );
                f.render_widget(
                    Paragraph::new(vec![
                        Spans::from(Span::styled(
                            battery_details.watt_consumption.clone(),
                            painter.colours.text_style,
                        )),
                        match &battery_details.charge_times {
                            crate::data_conversion::BatteryDuration::Charging { short, long }
                            | crate::data_conversion::BatteryDuration::Discharging {
                                short,
                                long,
                            } => Spans::from(Span::styled(
                                if (per_detail_area[1].width as usize) >= long.len() {
                                    long
                                } else {
                                    short
                                },
                                painter.colours.text_style,
                            )),
                            crate::data_conversion::BatteryDuration::Neither => {
                                Spans::from(Span::styled("N/A", painter.colours.text_style))
                            }
                        },
                        Spans::from(Span::styled(
                            battery_details.health.clone(),
                            painter.colours.text_style,
                        )),
                    ]),
                    per_detail_area[1],
                );
            }
        }
        // Note the block must be rendered last, to cover up the tabs!
        f.render_widget(block, area);
    }
}
