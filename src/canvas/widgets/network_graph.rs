use lazy_static::lazy_static;
use std::cmp::max;

use crate::{
    app::App,
    canvas::{drawing_utils::get_variable_intrinsic_widths, Painter},
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset, Row, Table},
};

const NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "Total RX", "Total TX"];

lazy_static! {
    static ref NETWORK_HEADERS_LENS: Vec<usize> = NETWORK_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
}

pub trait NetworkGraphWidget {
    fn draw_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );

    fn draw_network_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl NetworkGraphWidget for Painter {
    fn draw_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let network_chunk = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(max(draw_loc.height as i64 - 5, 0) as u16),
                    Constraint::Length(5),
                ]
                .as_ref(),
            )
            .split(draw_loc);

        self.draw_network_graph(f, app_state, network_chunk[0], widget_id);
        self.draw_network_labels(f, app_state, network_chunk[1], widget_id);
    }

    fn draw_network_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(network_widget_state) = app_state.net_state.widget_states.get_mut(&widget_id) {
            let network_data_rx: &[(f64, f64)] = &app_state.canvas_data.network_data_rx;
            let network_data_tx: &[(f64, f64)] = &app_state.canvas_data.network_data_tx;

            let display_time_labels = [
                format!("{}s", network_widget_state.current_display_time / 1000),
                "0s".to_string(),
            ];
            let x_axis = if app_state.app_config_fields.hide_time
                || (app_state.app_config_fields.autohide_time
                    && network_widget_state.autohide_timer.is_none())
            {
                Axis::default().bounds([-(network_widget_state.current_display_time as f64), 0.0])
            } else if let Some(time) = network_widget_state.autohide_timer {
                if std::time::Instant::now().duration_since(time).as_millis()
                    < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
                {
                    Axis::default()
                        .bounds([-(network_widget_state.current_display_time as f64), 0.0])
                        .style(self.colours.graph_style)
                        .labels_style(self.colours.graph_style)
                        .labels(&display_time_labels)
                } else {
                    network_widget_state.autohide_timer = None;
                    Axis::default()
                        .bounds([-(network_widget_state.current_display_time as f64), 0.0])
                }
            } else {
                Axis::default()
                    .bounds([-(network_widget_state.current_display_time as f64), 0.0])
                    .style(self.colours.graph_style)
                    .labels_style(self.colours.graph_style)
                    .labels(&display_time_labels)
            };

            // 0 is offset.
            let y_axis: Axis<'_, &str> = Axis::default()
                .style(self.colours.graph_style)
                .labels_style(self.colours.graph_style)
                .bounds([-0.5, 30_f64])
                .labels(&["0B", "1KiB", "1MiB", "1GiB"]);

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Network ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!(
                    " Network ─{}─ Esc to go back ",
                    "─".repeat(repeat_num as usize)
                );

                result_title
            } else {
                " Network ".to_string()
            };

            f.render_widget(
                Chart::default()
                    .block(
                        Block::default()
                            .title(&title)
                            .title_style(if app_state.is_expanded {
                                self.colours.highlighted_border_style
                            } else {
                                self.colours.widget_title_style
                            })
                            .borders(Borders::ALL)
                            .border_style(if app_state.current_widget.widget_id == widget_id {
                                self.colours.highlighted_border_style
                            } else {
                                self.colours.border_style
                            }),
                    )
                    .x_axis(x_axis)
                    .y_axis(y_axis)
                    .datasets(&[
                        Dataset::default()
                            .name(&format!("RX: {:7}", app_state.canvas_data.rx_display))
                            .marker(if app_state.app_config_fields.use_dot {
                                Marker::Dot
                            } else {
                                Marker::Braille
                            })
                            .style(self.colours.rx_style)
                            .data(&network_data_rx),
                        Dataset::default()
                            .name(&format!("TX: {:7}", app_state.canvas_data.tx_display))
                            .marker(if app_state.app_config_fields.use_dot {
                                Marker::Dot
                            } else {
                                Marker::Braille
                            })
                            .style(self.colours.tx_style)
                            .data(&network_data_tx),
                        Dataset::default()
                            .name(&format!(
                                "Total RX: {:7}",
                                app_state.canvas_data.total_rx_display
                            ))
                            .style(self.colours.total_rx_style),
                        Dataset::default()
                            .name(&format!(
                                "Total TX: {:7}",
                                app_state.canvas_data.total_tx_display
                            ))
                            .style(self.colours.total_tx_style),
                    ]),
                draw_loc,
            );
        }
    }

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let rx_display = &app_state.canvas_data.rx_display;
        let tx_display = &app_state.canvas_data.tx_display;
        let total_rx_display = &app_state.canvas_data.total_rx_display;
        let total_tx_display = &app_state.canvas_data.total_tx_display;

        // Gross but I need it to work...
        let total_network = vec![vec![
            rx_display,
            tx_display,
            total_rx_display,
            total_tx_display,
        ]];
        let mapped_network = total_network
            .iter()
            .map(|val| Row::StyledData(val.iter(), self.colours.text_style));

        // Calculate widths
        let width_ratios: Vec<f64> = vec![0.25, 0.25, 0.25, 0.25];
        let lens: &[usize] = &NETWORK_HEADERS_LENS;
        let width = f64::from(draw_loc.width);

        let variable_intrinsic_results =
            get_variable_intrinsic_widths(width as u16, &width_ratios, lens);
        let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

        // Draw
        f.render_widget(
            Table::new(NETWORK_HEADERS.iter(), mapped_network)
                .block(Block::default().borders(Borders::ALL).border_style(
                    if app_state.current_widget.widget_id == widget_id {
                        self.colours.highlighted_border_style
                    } else {
                        self.colours.border_style
                    },
                ))
                .header_style(self.colours.table_header_style)
                .style(self.colours.text_style)
                .widths(
                    &(intrinsic_widths
                        .iter()
                        .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                        .collect::<Vec<_>>()),
                ),
            draw_loc,
        );
    }
}
