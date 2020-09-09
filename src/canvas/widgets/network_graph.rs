use lazy_static::lazy_static;
use std::cmp::max;

use crate::{
    app::App,
    canvas::{drawing_utils::get_column_widths, Painter},
    constants::*,
    utils::gen_util::*,
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
    static ref NETWORK_HEADERS_LENS: Vec<u16> = NETWORK_HEADERS
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>();
}

pub trait NetworkGraphWidget {
    fn draw_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );

    fn draw_network_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
        hide_legend: bool,
    );

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl NetworkGraphWidget for Painter {
    fn draw_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if app_state.app_config_fields.use_old_network_legend {
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

            self.draw_network_graph(f, app_state, network_chunk[0], widget_id, true);
            self.draw_network_labels(f, app_state, network_chunk[1], widget_id);
        } else {
            self.draw_network_graph(f, app_state, draw_loc, widget_id, false);
        }

        if app_state.should_get_widget_bounds() {
            // Update draw loc in widget map
            // Note that in both cases, we always go to the same widget id so it's fine to do it like
            // this lol.
            if let Some(network_widget) = app_state.widget_map.get_mut(&widget_id) {
                network_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                network_widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }

    fn draw_network_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
        hide_legend: bool,
    ) {
        /// Point is of time, data
        type Point = (f64, f64);

        /// Returns the required max data point and labels.
        fn adjust_network_data_point(
            rx: &[Point], tx: &[Point], time_start: f64, time_end: f64,
        ) -> (f64, Vec<String>) {
            // First, filter and find the maximal rx or tx so we know how to scale
            let mut max_val_bytes = 0.0;
            let filtered_rx = rx
                .iter()
                .cloned()
                .filter(|(time, _data)| *time >= time_start && *time <= time_end);

            let filtered_tx = tx
                .iter()
                .cloned()
                .filter(|(time, _data)| *time >= time_start && *time <= time_end);

            for (_time, data) in filtered_rx.clone().chain(filtered_tx.clone()) {
                if data > max_val_bytes {
                    max_val_bytes = data;
                }
            }

            // FIXME [NETWORKING]: Do ya think it would be possible for a more granular approach?
            // Currently we do 32 -> 33... which skips some gigabit values
            let true_max_val: f64;
            let mut labels = vec![];
            if max_val_bytes < LOG_KIBI_LIMIT {
                true_max_val = LOG_KIBI_LIMIT;
                labels = vec!["0B".to_string(), "1KiB".to_string()];
            } else if max_val_bytes < LOG_MEBI_LIMIT {
                true_max_val = LOG_MEBI_LIMIT;
                labels = vec!["0B".to_string(), "1KiB".to_string(), "1MiB".to_string()];
            } else if max_val_bytes < LOG_GIBI_LIMIT {
                true_max_val = LOG_GIBI_LIMIT;
                labels = vec![
                    "0B".to_string(),
                    "1KiB".to_string(),
                    "1MiB".to_string(),
                    "1GiB".to_string(),
                ];
            } else if max_val_bytes < LOG_TEBI_LIMIT {
                true_max_val = max_val_bytes.ceil() + 1.0;
                let cap_u32 = true_max_val as u32;

                for i in 0..=cap_u32 {
                    match i {
                        0 => labels.push("0B".to_string()),
                        LOG_KIBI_LIMIT_U32 => labels.push("1KiB".to_string()),
                        LOG_MEBI_LIMIT_U32 => labels.push("1MiB".to_string()),
                        LOG_GIBI_LIMIT_U32 => labels.push("1GiB".to_string()),
                        _ if i == cap_u32 => {
                            labels.push(format!("{}GiB", 2_u64.pow(cap_u32 - LOG_GIBI_LIMIT_U32)))
                        }
                        _ if i == (LOG_GIBI_LIMIT_U32 + cap_u32) / 2 => labels.push(format!(
                            "{}GiB",
                            2_u64.pow(cap_u32 - ((LOG_GIBI_LIMIT_U32 + cap_u32) / 2))
                        )), // ~Halfway point
                        _ => labels.push(String::default()),
                    }
                }
            } else {
                true_max_val = max_val_bytes.ceil() + 1.0;
                let cap_u32 = true_max_val as u32;

                for i in 0..=cap_u32 {
                    match i {
                        0 => labels.push("0B".to_string()),
                        LOG_KIBI_LIMIT_U32 => labels.push("1KiB".to_string()),
                        LOG_MEBI_LIMIT_U32 => labels.push("1MiB".to_string()),
                        LOG_GIBI_LIMIT_U32 => labels.push("1GiB".to_string()),
                        LOG_TEBI_LIMIT_U32 => labels.push("1TiB".to_string()),
                        _ if i == cap_u32 => {
                            labels.push(format!("{}GiB", 2_u64.pow(cap_u32 - LOG_TEBI_LIMIT_U32)))
                        }
                        _ if i == (LOG_TEBI_LIMIT_U32 + cap_u32) / 2 => labels.push(format!(
                            "{}TiB",
                            2_u64.pow(cap_u32 - ((LOG_TEBI_LIMIT_U32 + cap_u32) / 2))
                        )), // ~Halfway point
                        _ => labels.push(String::default()),
                    }
                }
            }

            (true_max_val, labels)
        }

        if let Some(network_widget_state) = app_state.net_state.widget_states.get_mut(&widget_id) {
            let network_data_rx: &[(f64, f64)] = &app_state.canvas_data.network_data_rx;
            let network_data_tx: &[(f64, f64)] = &app_state.canvas_data.network_data_tx;

            let (max_range, labels) = adjust_network_data_point(
                network_data_rx,
                network_data_tx,
                -(network_widget_state.current_display_time as f64),
                0.0,
            );
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
                        .labels(&display_time_labels)
                        .labels_style(self.colours.graph_style)
                } else {
                    network_widget_state.autohide_timer = None;
                    Axis::default()
                        .bounds([-(network_widget_state.current_display_time as f64), 0.0])
                }
            } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
                Axis::default().bounds([-(network_widget_state.current_display_time as f64), 0.0])
            } else {
                Axis::default()
                    .bounds([-(network_widget_state.current_display_time as f64), 0.0])
                    .style(self.colours.graph_style)
                    .labels(&display_time_labels)
                    .labels_style(self.colours.graph_style)
            };

            let y_axis_labels = labels;
            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .bounds([0.0, max_range])
                .labels(&y_axis_labels)
                .labels_style(self.colours.graph_style);

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Network ── Esc to go back ";
                format!(
                    " Network ─{}─ Esc to go back ",
                    "─".repeat(
                        usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2)
                    )
                )
            } else {
                " Network ".to_string()
            };
            let title_style = if app_state.is_expanded {
                self.colours.highlighted_border_style
            } else {
                self.colours.widget_title_style
            };

            let legend_constraints = if hide_legend {
                (Constraint::Ratio(0, 1), Constraint::Ratio(0, 1))
            } else {
                (Constraint::Ratio(3, 4), Constraint::Ratio(3, 4))
            };

            let dataset = if app_state.app_config_fields.use_old_network_legend && !hide_legend {
                let mut ret_val = vec![];
                ret_val.push(
                    Dataset::default()
                        .name(format!("RX: {:7}", app_state.canvas_data.rx_display))
                        .marker(if app_state.app_config_fields.use_dot {
                            Marker::Dot
                        } else {
                            Marker::Braille
                        })
                        .style(self.colours.rx_style)
                        .data(&network_data_rx)
                        .graph_type(tui::widgets::GraphType::Line),
                );

                ret_val.push(
                    Dataset::default()
                        .name(format!("TX: {:7}", app_state.canvas_data.tx_display))
                        .marker(if app_state.app_config_fields.use_dot {
                            Marker::Dot
                        } else {
                            Marker::Braille
                        })
                        .style(self.colours.tx_style)
                        .data(&network_data_tx)
                        .graph_type(tui::widgets::GraphType::Line),
                );
                ret_val.push(
                    Dataset::default()
                        .name(format!(
                            "Total RX: {:7}",
                            app_state.canvas_data.total_rx_display
                        ))
                        .style(self.colours.total_rx_style),
                );

                ret_val.push(
                    Dataset::default()
                        .name(format!(
                            "Total TX: {:7}",
                            app_state.canvas_data.total_tx_display
                        ))
                        .style(self.colours.total_tx_style),
                );

                ret_val
            } else {
                let mut ret_val = vec![];

                ret_val.push(
                    Dataset::default()
                        .name(&app_state.canvas_data.rx_display)
                        .marker(if app_state.app_config_fields.use_dot {
                            Marker::Dot
                        } else {
                            Marker::Braille
                        })
                        .style(self.colours.rx_style)
                        .data(&network_data_rx)
                        .graph_type(tui::widgets::GraphType::Line),
                );

                ret_val.push(
                    Dataset::default()
                        .name(&app_state.canvas_data.tx_display)
                        .marker(if app_state.app_config_fields.use_dot {
                            Marker::Dot
                        } else {
                            Marker::Braille
                        })
                        .style(self.colours.tx_style)
                        .data(&network_data_tx)
                        .graph_type(tui::widgets::GraphType::Line),
                );

                ret_val
            };

            f.render_widget(
                // Chart::new(dataset)
                Chart::default()
                    .datasets(&dataset)
                    .block(
                        Block::default()
                            .title(&title)
                            .title_style(title_style)
                            .borders(Borders::ALL)
                            .border_style(if app_state.current_widget.widget_id == widget_id {
                                self.colours.highlighted_border_style
                            } else {
                                self.colours.border_style
                            }),
                    )
                    .x_axis(x_axis)
                    .y_axis(y_axis)
                    .hidden_legend_constraints(legend_constraints),
                draw_loc,
            );
        }
    }

    // TODO: [DEPRECATED] Get rid of this in, like, 0.6...?
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
        let intrinsic_widths = get_column_widths(
            draw_loc.width,
            &[None, None, None, None],
            &[Some(6); 4],
            &[Some(0.25); 4],
            &(NETWORK_HEADERS_LENS
                .iter()
                .map(|s| Some(*s))
                .collect::<Vec<_>>()),
            &[0, 1, 2, 3],
        );

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
