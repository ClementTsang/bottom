use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    widgets::Block,
    Frame,
};

use crate::{
    app::{
        event::ComponentEventResult, widgets::tui_stuff::PipeGauge, Component, DataCollection,
        Widget,
    },
    canvas::Painter,
    constants::SIDE_BORDERS,
    data_conversion::{convert_mem_data_points, convert_mem_labels, convert_swap_data_points},
    options::layout_options::WidgetLayoutRule,
};

#[derive(Debug)]
pub struct BasicMem {
    bounds: Rect,
    width: WidgetLayoutRule,
    mem_data: (f64, String, String),
    swap_data: Option<(f64, String, String)>,
    use_percent: bool,
}

impl Default for BasicMem {
    fn default() -> Self {
        Self {
            bounds: Default::default(),
            width: Default::default(),
            mem_data: (0.0, "0.0B/0.0B".to_string(), "0%".to_string()),
            swap_data: None,
            use_percent: false,
        }
    }
}

impl BasicMem {
    /// Sets the width.
    pub fn width(mut self, width: WidgetLayoutRule) -> Self {
        self.width = width;
        self
    }
}

impl Component for BasicMem {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        match event.code {
            KeyCode::Char('%') => {
                self.use_percent = !self.use_percent;
                ComponentEventResult::Redraw
            }
            _ => ComponentEventResult::Unhandled,
        }
    }
}

impl Widget for BasicMem {
    fn get_pretty_name(&self) -> &'static str {
        "Memory"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
        _expanded: bool,
    ) {
        let block = Block::default()
            .borders(*SIDE_BORDERS)
            .border_style(painter.colours.highlighted_border_style);
        let inner_area = block.inner(area);
        const CONSTRAINTS: [Constraint; 2] = [Constraint::Ratio(1, 2); 2];
        let split_area = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(CONSTRAINTS)
            .split(inner_area);

        if selected {
            f.render_widget(block, area);
        }

        let mut use_percentage =
            self.use_percent || (split_area[0].width as usize) < self.mem_data.1.len() + 7;

        if let Some(swap_data) = &self.swap_data {
            use_percentage =
                use_percentage || (split_area[1].width as usize) < swap_data.1.len() + 7;

            f.render_widget(
                PipeGauge::default()
                    .ratio(swap_data.0)
                    .style(painter.colours.swap_style)
                    .gauge_style(painter.colours.swap_style)
                    .start_label("SWP")
                    .end_label(if use_percentage {
                        swap_data.2.clone()
                    } else {
                        swap_data.1.clone()
                    }),
                split_area[1],
            );
        }
        f.render_widget(
            PipeGauge::default()
                .ratio(self.mem_data.0)
                .style(painter.colours.ram_style)
                .gauge_style(painter.colours.ram_style)
                .start_label("RAM")
                .end_label(if use_percentage {
                    self.mem_data.2.clone()
                } else {
                    self.mem_data.1.clone()
                }),
            split_area[0],
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        let (memory_labels, swap_labels) = convert_mem_labels(data_collection);

        // TODO: [Optimization] Probably should just make another function altogether for just basic mem mode.
        self.mem_data = if let (Some(data), Some((_, fraction))) = (
            convert_mem_data_points(data_collection).last(),
            memory_labels,
        ) {
            (
                data.1 / 100.0,
                fraction.trim().to_string(),
                format!("{:3.0}%", data.1.round()),
            )
        } else {
            (0.0, "0.0B/0.0B".to_string(), "0%".to_string())
        };
        self.swap_data = if let (Some(data), Some((_, fraction))) = (
            convert_swap_data_points(data_collection).last(),
            swap_labels,
        ) {
            Some((
                data.1 / 100.0,
                fraction.trim().to_string(),
                format!("{:3.0}%", data.1.round()),
            ))
        } else {
            None
        };
    }

    fn width(&self) -> WidgetLayoutRule {
        self.width
    }

    fn height(&self) -> WidgetLayoutRule {
        todo!()
    }
}
