use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{
    app::{AppConfigFields, AxisScaling, Component, DataCollection, Widget},
    canvas::Painter,
    constants::SIDE_BORDERS,
    data_conversion::convert_network_data_points,
    options::layout_options::LayoutRule,
    units::data_units::DataUnit,
};

#[derive(Debug)]
pub struct BasicNet {
    bounds: Rect,
    width: LayoutRule,

    rx_display: String,
    tx_display: String,
    total_rx_display: String,
    total_tx_display: String,

    pub unit_type: DataUnit,
    pub use_binary_prefix: bool,
}

impl BasicNet {
    /// Creates a new [`BasicNet`] given a [`AppConfigFields`].
    pub fn from_config(app_config_fields: &AppConfigFields) -> Self {
        Self {
            bounds: Default::default(),
            width: Default::default(),
            rx_display: "RX: 0b/s".to_string(),
            tx_display: "TX: 0b/s".to_string(),
            total_rx_display: "Total RX: 0B".to_string(),
            total_tx_display: "Total TX: 0B".to_string(),
            unit_type: app_config_fields.network_unit_type.clone(),
            use_binary_prefix: app_config_fields.network_use_binary_prefix,
        }
    }

    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }
}

impl Component for BasicNet {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for BasicNet {
    fn get_pretty_name(&self) -> &'static str {
        "Network"
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
            .direction(tui::layout::Direction::Horizontal)
            .constraints(CONSTRAINTS)
            .split(inner_area);
        let texts = [
            [
                Spans::from(Span::styled(&self.rx_display, painter.colours.rx_style)),
                Spans::from(Span::styled(&self.tx_display, painter.colours.tx_style)),
            ],
            [
                Spans::from(Span::styled(
                    &self.total_rx_display,
                    painter.colours.total_rx_style,
                )),
                Spans::from(Span::styled(
                    &self.total_tx_display,
                    painter.colours.total_tx_style,
                )),
            ],
        ];

        if selected {
            f.render_widget(block, area);
        }

        IntoIterator::into_iter(texts)
            .zip(split_area)
            .for_each(|(text, area)| f.render_widget(Paragraph::new(text.to_vec()), area));
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        let network_data = convert_network_data_points(
            data_collection,
            true,
            &AxisScaling::Linear,
            &self.unit_type,
            self.use_binary_prefix,
        );
        self.rx_display = format!("RX: {}", network_data.rx_display);
        self.tx_display = format!("TX: {}", network_data.tx_display);
        if let Some(total_rx_display) = network_data.total_rx_display {
            self.total_rx_display = format!("Total RX: {}", total_rx_display);
        }
        if let Some(total_tx_display) = network_data.total_tx_display {
            self.total_tx_display = format!("Total TX: {}", total_tx_display);
        }
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        LayoutRule::Length { length: 2 }
    }
}
