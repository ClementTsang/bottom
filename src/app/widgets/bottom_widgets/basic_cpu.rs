use std::cmp::max;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Block,
    Frame,
};

use crate::{
    app::{widgets::tui_stuff::PipeGauge, AppConfig, Component, DataCollection, Widget},
    canvas::Painter,
    constants::SIDE_BORDERS,
    options::layout_options::LayoutRule,
};

const REQUIRED_COLUMNS: usize = 4;

#[derive(Debug)]
pub struct BasicCpu {
    bounds: Rect,
    display_data: Vec<(f64, String, String)>,
    width: LayoutRule,
    showing_avg: bool,
}

impl BasicCpu {
    /// Creates a new [`BasicCpu`] given a [`AppConfigFields`].
    pub fn from_config(app_config_fields: &AppConfig) -> Self {
        Self {
            bounds: Default::default(),
            display_data: Default::default(),
            width: Default::default(),
            showing_avg: app_config_fields.show_average_cpu,
        }
    }

    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }
}

impl Component for BasicCpu {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for BasicCpu {
    fn get_pretty_name(&self) -> &'static str {
        "CPU"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
        _expanded: bool,
    ) {
        const CONSTRAINTS: [Constraint; 2 * REQUIRED_COLUMNS - 1] = [
            Constraint::Ratio(1, REQUIRED_COLUMNS as u32),
            Constraint::Length(2),
            Constraint::Ratio(1, REQUIRED_COLUMNS as u32),
            Constraint::Length(2),
            Constraint::Ratio(1, REQUIRED_COLUMNS as u32),
            Constraint::Length(2),
            Constraint::Ratio(1, REQUIRED_COLUMNS as u32),
        ];
        let block = Block::default()
            .borders(*SIDE_BORDERS)
            .border_style(painter.colours.highlighted_border_style);
        let inner_area = block.inner(area);
        let split_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(CONSTRAINTS)
            .split(inner_area)
            .into_iter()
            .enumerate()
            .filter_map(
                |(index, rect)| {
                    if index % 2 == 0 {
                        Some(rect)
                    } else {
                        None
                    }
                },
            );

        let display_data_len = self.display_data.len();
        let length = display_data_len / REQUIRED_COLUMNS;
        let largest_height = max(
            1,
            length
                + (if display_data_len % REQUIRED_COLUMNS == 0 {
                    0
                } else {
                    1
                }),
        );
        let mut leftover = display_data_len % REQUIRED_COLUMNS;
        let column_heights = (0..REQUIRED_COLUMNS).map(|_| {
            if leftover > 0 {
                leftover -= 1;
                length + 1
            } else {
                length
            }
        });

        if selected {
            f.render_widget(block, area);
        }

        let mut index_offset = 0;
        split_area
            .into_iter()
            .zip(column_heights)
            .for_each(|(area, height)| {
                let column_areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Length(1); largest_height])
                    .split(area);

                let num_entries = if index_offset + height < display_data_len {
                    height
                } else {
                    display_data_len - index_offset
                };
                let end = index_offset + num_entries;

                self.display_data[index_offset..end]
                    .iter()
                    .zip(column_areas)
                    .enumerate()
                    .for_each(|(column_index, ((percent, label, usage_label), area))| {
                        let cpu_index = index_offset + column_index;
                        let style = if cpu_index == 0 {
                            painter.colours.avg_colour_style
                        } else {
                            let cpu_style_index = if self.showing_avg {
                                cpu_index - 1
                            } else {
                                cpu_index
                            };
                            painter.colours.cpu_colour_styles
                                [cpu_style_index % painter.colours.cpu_colour_styles.len()]
                        };

                        f.render_widget(
                            PipeGauge::default()
                                .ratio(*percent)
                                .style(style)
                                .gauge_style(style)
                                .start_label(label.clone())
                                .end_label(usage_label.clone()),
                            area,
                        );
                    });

                index_offset = end;
            });
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        self.display_data = data_collection
            .cpu_harvest
            .iter()
            .map(|data| {
                (
                    data.cpu_usage / 100.0,
                    format!(
                        "{:3}",
                        data.cpu_count
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| data.cpu_prefix.clone())
                    ),
                    format!("{:3.0}%", data.cpu_usage.round()),
                )
            })
            .collect::<Vec<_>>();
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        let display_data_len = self.display_data.len();
        let length = max(
            1,
            (display_data_len / REQUIRED_COLUMNS) as u16
                + (if display_data_len % REQUIRED_COLUMNS == 0 {
                    0
                } else {
                    1
                }),
        );

        todo!()
    }
}
