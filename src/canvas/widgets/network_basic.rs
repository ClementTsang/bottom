use crate::{app::App, canvas::Painter, constants::*};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Paragraph, Text, Widget},
};

pub trait NetworkBasicWidget {
    fn draw_basic_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl NetworkBasicWidget for Painter {
    fn draw_basic_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let divided_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(draw_loc);

        let net_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(divided_loc[0]);

        let total_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(divided_loc[1]);

        if app_state.current_widget.widget_id == widget_id {
            Block::default()
                .borders(*SIDE_BORDERS)
                .border_style(self.colours.highlighted_border_style)
                .render(f, draw_loc);
        }

        let rx_label = format!("RX: {}\n", &app_state.canvas_data.rx_display);
        let tx_label = format!("TX: {}", &app_state.canvas_data.tx_display);
        let total_rx_label = format!("Total RX: {}\n", &app_state.canvas_data.total_rx_display);
        let total_tx_label = format!("Total TX: {}", &app_state.canvas_data.total_tx_display);

        let net_text = vec![
            Text::Styled(rx_label.into(), self.colours.rx_style),
            Text::Styled(tx_label.into(), self.colours.tx_style),
        ];

        let total_net_text = vec![
            Text::Styled(total_rx_label.into(), self.colours.total_rx_style),
            Text::Styled(total_tx_label.into(), self.colours.total_tx_style),
        ];

        Paragraph::new(net_text.iter())
            .block(Block::default())
            .render(f, net_loc[0]);

        Paragraph::new(total_net_text.iter())
            .block(Block::default())
            .render(f, total_loc[0]);
    }
}
