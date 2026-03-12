use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    app::App,
    canvas::{
        Painter,
        drawing_utils::widget_block,
        widgets::{PacketInfo, calculate_packet_info},
    },
    utils::data_units::{convert_bits, get_unit_prefix},
};

impl Painter {
    pub fn draw_basic_network(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let show_packets = app_state.app_config_fields.network_show_packets;

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

        let rx_label = format!("RX: {:.1}{}/s", rx.0, rx.1);
        let tx_label = format!("TX: {:.1}{}/s", tx.0, tx.1);
        let total_rx_label = format!("Total RX: {:.1}{}", total_rx.0, total_rx.1);
        let total_tx_label = format!("Total TX: {:.1}{}", total_tx.0, total_tx.1);

        // Determine if we need grid layout based on available width
        // Assume we need at least ~15 chars per column for horizontal layout
        // With 4 columns, that's ~60 chars minimum
        // If width is less than 60, use grid layout (4 rows x 2 columns)
        if show_packets {
            let PacketInfo {
                rx_packet_rate,
                tx_packet_rate,
                avg_rx_packet_size,
                avg_tx_packet_size,
            } = calculate_packet_info(network_data, use_binary_prefix);

            // TODO: Stylize packet stuff later with something else? Or maybe make it so total is now (by default) just bolded RX/TX? I doubt anyone cares...
            let rx_packet_rate_label = format!("RX Pkt: {}pkt/s", rx_packet_rate);
            let tx_packet_rate_label = format!("TX Pkt: {}pkt/s", tx_packet_rate);
            let avg_rx_packet_size_label = format!(
                "Avg RX Pkt: {:.1}{}",
                avg_rx_packet_size.0, avg_rx_packet_size.1
            );
            let avg_tx_packet_size_label = format!(
                "Avg TX Pkt: {:.1}{}",
                avg_tx_packet_size.0, avg_tx_packet_size.1
            );

            if draw_loc.width < 60 {
                // 4 rows x 2 columns layout
                // Column 1: RX, TX, Total RX, Total TX (top to bottom)
                // Column 2: RX Packets, TX Packets, AVG RX, AVG TX (top to bottom)
                let grid_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(draw_loc);

                // Column 1: RX, TX, Total RX, Total TX
                let col1_loc = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ])
                    .split(grid_loc[0]);
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(rx_label, self.styles.rx_style)))
                        .block(Block::default()),
                    col1_loc[0],
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(tx_label, self.styles.tx_style)))
                        .block(Block::default()),
                    col1_loc[1],
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        total_rx_label,
                        self.styles.total_rx_style,
                    )))
                    .block(Block::default()),
                    col1_loc[2],
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        total_tx_label,
                        self.styles.total_tx_style,
                    )))
                    .block(Block::default()),
                    col1_loc[3],
                );

                // Column 2: RX Packets, TX Packets, AVG RX, AVG TX
                let col2_loc = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ])
                    .split(grid_loc[1]);
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        rx_packet_rate_label,
                        self.styles.rx_style,
                    )))
                    .block(Block::default()),
                    col2_loc[0],
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        tx_packet_rate_label,
                        self.styles.tx_style,
                    )))
                    .block(Block::default()),
                    col2_loc[1],
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        avg_rx_packet_size_label,
                        self.styles.total_rx_style,
                    )))
                    .block(Block::default()),
                    col2_loc[2],
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        avg_tx_packet_size_label,
                        self.styles.total_tx_style,
                    )))
                    .block(Block::default()),
                    col2_loc[3],
                );
            } else {
                // Horizontal 4-column layout
                let constraints = [
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ];

                let divided_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(draw_loc);

                // Column 1: RX/TX
                let col1_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(1)
                    .split(divided_loc[0]);
                let col1_text = vec![
                    Line::from(Span::styled(rx_label, self.styles.rx_style)),
                    Line::from(Span::styled(tx_label, self.styles.tx_style)),
                ];
                f.render_widget(
                    Paragraph::new(col1_text).block(Block::default()),
                    col1_loc[0],
                );

                // Column 2: Total RX/TX
                let col2_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(1)
                    .split(divided_loc[1]);
                let col2_text = vec![
                    Line::from(Span::styled(total_rx_label, self.styles.total_rx_style)),
                    Line::from(Span::styled(total_tx_label, self.styles.total_tx_style)),
                ];
                f.render_widget(
                    Paragraph::new(col2_text).block(Block::default()),
                    col2_loc[0],
                );

                // Column 3: RX/TX packets
                let col3_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(1)
                    .split(divided_loc[2]);
                let col3_text = vec![
                    Line::from(Span::styled(rx_packet_rate_label, self.styles.rx_style)),
                    Line::from(Span::styled(tx_packet_rate_label, self.styles.tx_style)),
                ];
                f.render_widget(
                    Paragraph::new(col3_text).block(Block::default()),
                    col3_loc[0],
                );

                // Column 4: AVG RX/TX packets
                let col4_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(1)
                    .split(divided_loc[3]);
                let col4_text = vec![
                    Line::from(Span::styled(
                        avg_rx_packet_size_label,
                        self.styles.total_rx_style,
                    )),
                    Line::from(Span::styled(
                        avg_tx_packet_size_label,
                        self.styles.total_tx_style,
                    )),
                ];
                f.render_widget(
                    Paragraph::new(col4_text).block(Block::default()),
                    col4_loc[0],
                );
            }
        } else {
            // No packets, 2-column layout
            let constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];

            let divided_loc = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(draw_loc);

            // Column 1: RX/TX
            let col1_loc = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(1)
                .split(divided_loc[0]);
            let col1_text = vec![
                Line::from(Span::styled(rx_label, self.styles.rx_style)),
                Line::from(Span::styled(tx_label, self.styles.tx_style)),
            ];
            f.render_widget(
                Paragraph::new(col1_text).block(Block::default()),
                col1_loc[0],
            );

            // Column 2: Total RX/TX
            let col2_loc = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(1)
                .split(divided_loc[1]);
            let col2_text = vec![
                Line::from(Span::styled(total_rx_label, self.styles.total_rx_style)),
                Line::from(Span::styled(total_tx_label, self.styles.total_tx_style)),
            ];
            f.render_widget(
                Paragraph::new(col2_text).block(Block::default()),
                col2_loc[0],
            );
        }

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
