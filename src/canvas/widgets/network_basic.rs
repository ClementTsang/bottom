use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    app::App,
    canvas::{Painter, drawing_utils::widget_block},
    utils::data_units::{convert_bits, get_unit_prefix},
};

impl Painter {
    pub fn draw_basic_network(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let divided_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(draw_loc);

        let net_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(1)
            .split(divided_loc[0]);

        let total_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(1)
            .split(divided_loc[1]);

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                widget_block(true, true, self.styles.border_type)
                    .border_style(self.styles.highlighted_border_style),
                draw_loc,
            );
        }

        let use_binary_prefix = app_state.app_config_fields.network_use_binary_prefix;
        let network_data = &(app_state.data_store.get_data().network_harvest);
        let rx = get_unit_prefix(network_data.rx, use_binary_prefix);
        let tx = get_unit_prefix(network_data.tx, use_binary_prefix);
        let total_rx = convert_bits(network_data.total_rx, use_binary_prefix);
        let total_tx = convert_bits(network_data.total_tx, use_binary_prefix);

        let rx_label = format!("RX: {:.1}{}", rx.0, rx.1);
        let tx_label = format!("TX: {:.1}{}", tx.0, tx.1);
        let total_rx_label = format!("Total RX: {:.1}{}", total_rx.0, total_rx.1);
        let total_tx_label = format!("Total TX: {:.1}{}", total_tx.0, total_tx.1);

        let net_text = vec![
            Line::from(Span::styled(rx_label, self.styles.rx_style)),
            Line::from(Span::styled(tx_label, self.styles.tx_style)),
        ];

        let total_net_text = vec![
            Line::from(Span::styled(total_rx_label, self.styles.total_rx_style)),
            Line::from(Span::styled(total_tx_label, self.styles.total_tx_style)),
        ];

        f.render_widget(Paragraph::new(net_text).block(Block::default()), net_loc[0]);

        f.render_widget(
            Paragraph::new(total_net_text).block(Block::default()),
            total_loc[0],
        );

        // Update draw loc in widget map
        if app_state.should_get_widget_bounds() {
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}
