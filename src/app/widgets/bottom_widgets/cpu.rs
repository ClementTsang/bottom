use std::borrow::Cow;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    app::{
        event::{ComponentEventResult, SelectionAction},
        text_table::SimpleColumn,
        time_graph::TimeGraphData,
        widgets::tui_stuff::BlockBuilder,
        AppConfigFields, Component, DataCollection, TextTable, TimeGraph, Widget,
    },
    canvas::Painter,
    data_conversion::{convert_cpu_data_points, ConvertedCpuData},
    options::layout_options::LayoutRule,
};

/// Which part of the [`CpuGraph`] is currently selected.
enum CpuGraphSelection {
    Graph,
    Legend,
}

/// Whether the [`CpuGraph`]'s legend is placed on the left or right.
pub enum CpuGraphLegendPosition {
    Left,
    Right,
}

/// A widget designed to show CPU usage via a graph, along with a side legend in a table.
pub struct CpuGraph {
    graph: TimeGraph,
    legend: TextTable<SimpleColumn>,
    legend_position: CpuGraphLegendPosition,
    showing_avg: bool,

    bounds: Rect,
    selected: CpuGraphSelection,

    display_data: Vec<ConvertedCpuData>,
    load_avg_data: [f32; 3],

    width: LayoutRule,
    height: LayoutRule,
}

impl CpuGraph {
    /// Creates a new [`CpuGraph`] from a config.
    pub fn from_config(app_config_fields: &AppConfigFields) -> Self {
        let graph = TimeGraph::from_config(app_config_fields);
        let legend = TextTable::new(vec![
            SimpleColumn::new_flex("CPU".into(), 0.5),
            SimpleColumn::new_hard("Use".into(), None),
        ])
        .default_ltr(false)
        .try_show_gap(app_config_fields.table_gap);
        let legend_position = if app_config_fields.left_legend {
            CpuGraphLegendPosition::Left
        } else {
            CpuGraphLegendPosition::Right
        };
        let showing_avg = app_config_fields.show_average_cpu;

        Self {
            graph,
            legend,
            legend_position,
            showing_avg,
            bounds: Rect::default(),
            selected: CpuGraphSelection::Graph,
            display_data: Default::default(),
            load_avg_data: [0.0; 3],
            width: LayoutRule::default(),
            height: LayoutRule::default(),
        }
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
}

impl Component for CpuGraph {
    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        match self.selected {
            CpuGraphSelection::Graph => self.graph.handle_key_event(event),
            CpuGraphSelection::Legend => self.legend.handle_key_event(event),
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        if self.graph.does_border_intersect_mouse(&event) {
            if let CpuGraphSelection::Graph = self.selected {
                self.graph.handle_mouse_event(event)
            } else {
                self.selected = CpuGraphSelection::Graph;
                self.graph.handle_mouse_event(event);
                ComponentEventResult::Redraw
            }
        } else if self.legend.does_border_intersect_mouse(&event) {
            if let CpuGraphSelection::Legend = self.selected {
                self.legend.handle_mouse_event(event)
            } else {
                self.selected = CpuGraphSelection::Legend;
                self.legend.handle_mouse_event(event);
                ComponentEventResult::Redraw
            }
        } else {
            ComponentEventResult::Unhandled
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for CpuGraph {
    fn get_pretty_name(&self) -> &'static str {
        "CPU"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
        expanded: bool,
    ) {
        let constraints = {
            const CPU_LEGEND_MIN_WIDTH: u16 = 10;
            let (legend_constraint, cpu_constraint) =
                if area.width * 15 / 100 < CPU_LEGEND_MIN_WIDTH {
                    (Constraint::Length(CPU_LEGEND_MIN_WIDTH), Constraint::Min(0))
                } else {
                    (Constraint::Percentage(15), Constraint::Percentage(85))
                };
            match self.legend_position {
                CpuGraphLegendPosition::Left => [legend_constraint, cpu_constraint],
                CpuGraphLegendPosition::Right => [cpu_constraint, legend_constraint],
            }
        };

        let split_area = Layout::default()
            .margin(0)
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let (graph_block_area, legend_block_area) = match self.legend_position {
            CpuGraphLegendPosition::Left => (split_area[1], split_area[0]),
            CpuGraphLegendPosition::Right => (split_area[0], split_area[1]),
        };

        let legend_block = self
            .block()
            .selected(selected && matches!(&self.selected, CpuGraphSelection::Legend))
            .show_esc(expanded)
            .hide_title(true);

        let legend_data = self
            .display_data
            .iter()
            .enumerate()
            .map(|(cpu_index, core_data)| {
                let style = Some(if cpu_index == 0 {
                    painter.colours.all_colour_style
                } else if self.showing_avg && cpu_index == 1 {
                    painter.colours.avg_colour_style
                } else {
                    let cpu_style_index = if self.showing_avg {
                        // No underflow should occur, as if cpu_index was
                        // 1 and avg is showing, it's caught by the above case!
                        cpu_index - 2
                    } else {
                        cpu_index - 1
                    };
                    painter.colours.cpu_colour_styles
                        [cpu_style_index % painter.colours.cpu_colour_styles.len()]
                });

                vec![
                    (
                        core_data.cpu_name.clone().into(),
                        Some(core_data.short_cpu_name.clone().into()),
                        style,
                    ),
                    (core_data.legend_value.clone().into(), None, style),
                ]
            })
            .collect::<Vec<_>>();

        // TODO: [Gotcha, Refactor] You MUST draw the table first, otherwise the index may mismatch after a reset. This is a bad gotcha - we should look into auto-updating the table's length!
        self.legend.draw_tui_table(
            painter,
            f,
            &legend_data,
            legend_block,
            legend_block_area,
            true,
            false,
        );

        const Y_BOUNDS: [f64; 2] = [0.0, 100.5];
        let y_bound_labels: [Cow<'static, str>; 2] = ["0%".into(), "100%".into()];

        let current_index = self.legend.current_scroll_index();
        let sliced_cpu_data = if current_index == 0 {
            &self.display_data[..]
        } else {
            &self.display_data[current_index..current_index + 1]
        };

        let cpu_data = sliced_cpu_data
            .iter()
            .enumerate()
            .map(|(cpu_index, core_data)| TimeGraphData {
                data: &core_data.cpu_data,
                label: None,
                style: {
                    let offset_cpu_index = cpu_index + current_index;
                    if offset_cpu_index == 0 {
                        painter.colours.all_colour_style
                    } else if self.showing_avg && offset_cpu_index == 1 {
                        painter.colours.avg_colour_style
                    } else {
                        let cpu_style_index = if self.showing_avg {
                            // No underflow should occur, as if offset_cpu_index was
                            // 1 and avg is showing, it's caught by the above case!
                            offset_cpu_index - 2
                        } else {
                            offset_cpu_index - 1
                        };
                        painter.colours.cpu_colour_styles
                            [cpu_style_index % painter.colours.cpu_colour_styles.len()]
                    }
                },
            })
            .collect::<Vec<_>>();

        let name = format!(
            "{} ─ {:.2} {:.2} {:.2}",
            self.get_pretty_name(),
            self.load_avg_data[0],
            self.load_avg_data[1],
            self.load_avg_data[2],
        );
        let graph_block = BlockBuilder::new(name)
            .selected(selected && matches!(&self.selected, CpuGraphSelection::Graph))
            .show_esc(expanded)
            .build(painter, graph_block_area);

        self.graph.draw_tui_chart(
            painter,
            f,
            &cpu_data,
            &y_bound_labels,
            Y_BOUNDS,
            true,
            graph_block,
            graph_block_area,
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        convert_cpu_data_points(data_collection, &mut self.display_data);
        self.load_avg_data = data_collection.load_avg_harvest;
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }

    fn handle_widget_selection_left(&mut self) -> SelectionAction {
        match self.legend_position {
            CpuGraphLegendPosition::Left => {
                if let CpuGraphSelection::Graph = self.selected {
                    self.selected = CpuGraphSelection::Legend;
                    SelectionAction::Handled
                } else {
                    SelectionAction::NotHandled
                }
            }
            CpuGraphLegendPosition::Right => {
                if let CpuGraphSelection::Legend = self.selected {
                    self.selected = CpuGraphSelection::Graph;
                    SelectionAction::Handled
                } else {
                    SelectionAction::NotHandled
                }
            }
        }
    }

    fn handle_widget_selection_right(&mut self) -> SelectionAction {
        match self.legend_position {
            CpuGraphLegendPosition::Left => {
                if let CpuGraphSelection::Legend = self.selected {
                    self.selected = CpuGraphSelection::Graph;
                    SelectionAction::Handled
                } else {
                    SelectionAction::NotHandled
                }
            }
            CpuGraphLegendPosition::Right => {
                if let CpuGraphSelection::Graph = self.selected {
                    self.selected = CpuGraphSelection::Legend;
                    SelectionAction::Handled
                } else {
                    SelectionAction::NotHandled
                }
            }
        }
    }
}
