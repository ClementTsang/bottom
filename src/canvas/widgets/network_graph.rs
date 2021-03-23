use once_cell::sync::Lazy;
use std::cmp::max;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::{App, AxisScaling},
    canvas::{drawing_utils::get_column_widths, Painter},
    constants::*,
    units::data_units::DataUnit,
    utils::gen_util::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    terminal::Frame,
    text::Span,
    text::{Spans, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, Row, Table},
};

const NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "Total RX", "Total TX"];

static NETWORK_HEADERS_LENS: Lazy<Vec<u16>> = Lazy::new(|| {
    NETWORK_HEADERS
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>()
});

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
                .constraints([
                    Constraint::Length(max(draw_loc.height as i64 - 5, 0) as u16),
                    Constraint::Length(5),
                ])
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

        /// Returns the max data point given a time.
        fn get_max_entry(
            rx: &[Point], tx: &[Point], time_start: f64, network_scale_type: &AxisScaling,
            network_use_binary_prefix: bool,
        ) -> f64 {
            /// Determines a "fake" max value in circumstances where we couldn't find one from the data.
            fn calculate_missing_max(
                network_scale_type: &AxisScaling, network_use_binary_prefix: bool,
            ) -> f64 {
                match network_scale_type {
                    AxisScaling::Log => {
                        if network_use_binary_prefix {
                            LOG_MEGA_LIMIT
                        } else {
                            LOG_MEBI_LIMIT
                        }
                    }
                    AxisScaling::Linear => {
                        if network_use_binary_prefix {
                            MEGA_LIMIT_F64
                        } else {
                            MEBI_LIMIT_F64
                        }
                    }
                }
            }

            // First, let's shorten our ranges to actually look.  We can abuse the fact that our rx and tx arrays
            // are sorted, so we can short-circuit our search to filter out only the relevant data points...
            let filtered_rx = if let (Some(rx_start), Some(rx_end)) = (
                rx.iter().position(|(time, _data)| *time >= time_start),
                rx.iter().rposition(|(time, _data)| *time <= 0.0),
            ) {
                Some(&rx[rx_start..=rx_end])
            } else {
                None
            };

            let filtered_tx = if let (Some(tx_start), Some(tx_end)) = (
                tx.iter().position(|(time, _data)| *time >= time_start),
                tx.iter().rposition(|(time, _data)| *time <= 0.0),
            ) {
                Some(&tx[tx_start..=tx_end])
            } else {
                None
            };

            // Then, find the maximal rx/tx so we know how to scale, and return it.
            match (filtered_rx, filtered_tx) {
                (None, None) => {
                    calculate_missing_max(network_scale_type, network_use_binary_prefix)
                }
                (None, Some(filtered_tx)) => {
                    match filtered_tx
                        .iter()
                        .max_by(|(_, data_a), (_, data_b)| get_ordering(data_a, data_b, false))
                    {
                        Some((_, max_val)) => *max_val,
                        None => {
                            calculate_missing_max(network_scale_type, network_use_binary_prefix)
                        }
                    }
                }
                (Some(filtered_rx), None) => {
                    match filtered_rx
                        .iter()
                        .max_by(|(_, data_a), (_, data_b)| get_ordering(data_a, data_b, false))
                    {
                        Some((_, max_val)) => *max_val,
                        None => {
                            calculate_missing_max(network_scale_type, network_use_binary_prefix)
                        }
                    }
                }
                (Some(filtered_rx), Some(filtered_tx)) => {
                    match filtered_rx
                        .iter()
                        .chain(filtered_tx)
                        .max_by(|(_, data_a), (_, data_b)| get_ordering(data_a, data_b, false))
                    {
                        Some((_, max_val)) => *max_val,
                        None => {
                            calculate_missing_max(network_scale_type, network_use_binary_prefix)
                        }
                    }
                }
            }
        }

        /// Returns the required max data point and labels.
        fn adjust_network_data_point(
            max_entry: f64, network_scale_type: &AxisScaling, network_unit_type: &DataUnit,
            network_use_binary_prefix: bool,
        ) -> (f64, Vec<String>) {
            // So, we're going with an approach like this:
            // - Main goal is to maximize the amount of information displayed given a specific height.
            //   We don't want to drown out some data if the ranges are too far though!  Nor do we want to filter
            //   out too much data...
            // - Change the y-axis unit (kilo/kibi, mega/mebi...) dynamically based on max load.
            //
            //
            // The idea is we take the top value, build our scale such that each "point" is a scaled version of that.
            // So for example, let's say I use 390 Mb/s.  If I drew 4 segments, it would be 97.5, 195, 292.5, 390, and
            // probably something like 438.75?
            //
            // So, how do we do this in tui-rs?  Well, if we  are using intervals that tie in perfectly to the max
            // value we want... then it's actually not that hard.  Since tui-rs accepts a vector as labels and will
            // properly space them all out... we just work with that and space it out properly.
            //
            // Dynamic chart idea based off of FreeNAS's chart design.

            // Now just check the largest unit we correspond to... then proceed to build some entries from there!
            let (k_limit, m_limit, g_limit, t_limit) = match network_scale_type {
                AxisScaling::Log => {
                    if network_use_binary_prefix {
                        (
                            LOG_KIBI_LIMIT,
                            LOG_MEBI_LIMIT,
                            LOG_GIBI_LIMIT,
                            LOG_TEBI_LIMIT,
                        )
                    } else {
                        (
                            LOG_KILO_LIMIT,
                            LOG_MEGA_LIMIT,
                            LOG_GIGA_LIMIT,
                            LOG_TERA_LIMIT,
                        )
                    }
                }
                AxisScaling::Linear => {
                    if network_use_binary_prefix {
                        (
                            KIBI_LIMIT_F64,
                            MEBI_LIMIT_F64,
                            GIBI_LIMIT_F64,
                            TEBI_LIMIT_F64,
                        )
                    } else {
                        (
                            KILO_LIMIT_F64,
                            MEGA_LIMIT_F64,
                            GIGA_LIMIT_F64,
                            TERA_LIMIT_F64,
                        )
                    }
                }
            };

            let bumped_max_entry = max_entry * 1.5; // We use the bumped up version to calculate our unit type.
            let (max_value_scaled, unit_prefix, unit_type): (f64, &str, &str) =
                if bumped_max_entry < k_limit {
                    (
                        max_entry,
                        "",
                        match network_unit_type {
                            DataUnit::Byte => "B",
                            DataUnit::Bit => "b",
                        },
                    )
                } else if bumped_max_entry < m_limit {
                    (
                        max_entry / k_limit,
                        if network_use_binary_prefix { "Ki" } else { "K" },
                        match network_unit_type {
                            DataUnit::Byte => "B",
                            DataUnit::Bit => "b",
                        },
                    )
                } else if bumped_max_entry < g_limit {
                    (
                        max_entry / m_limit,
                        if network_use_binary_prefix { "Mi" } else { "M" },
                        match network_unit_type {
                            DataUnit::Byte => "B",
                            DataUnit::Bit => "b",
                        },
                    )
                } else if bumped_max_entry < t_limit {
                    (
                        max_entry / g_limit,
                        if network_use_binary_prefix { "Gi" } else { "G" },
                        match network_unit_type {
                            DataUnit::Byte => "B",
                            DataUnit::Bit => "b",
                        },
                    )
                } else {
                    (
                        max_entry / t_limit,
                        if network_use_binary_prefix { "Ti" } else { "T" },
                        match network_unit_type {
                            DataUnit::Byte => "B",
                            DataUnit::Bit => "b",
                        },
                    )
                };

            // Finally, build an acceptable range starting from there, using the given height!
            // Note we try to put more of a weight on the bottom section vs. the top, since the top has less data.

            let base_unit = match network_scale_type {
                AxisScaling::Log => {
                    if network_use_binary_prefix {
                        f64::exp2(max_value_scaled)
                    } else {
                        10.0_f64.powf(max_value_scaled)
                    }
                }
                AxisScaling::Linear => max_value_scaled,
            };
            let labels: Vec<String> = vec![
                format!("0{}{}", unit_prefix, unit_type),
                format!("{:.1}", base_unit * 0.5),
                format!("{:.1}", base_unit),
                format!("{:.1}", base_unit * 1.5),
            ]
            .into_iter()
            .map(|s| format!("{:>5}", s))
            .collect();

            (bumped_max_entry, labels)
        }

        if let Some(network_widget_state) = app_state.net_state.widget_states.get_mut(&widget_id) {
            let network_data_rx: &[(f64, f64)] = &app_state.canvas_data.network_data_rx;
            let network_data_tx: &[(f64, f64)] = &app_state.canvas_data.network_data_tx;

            // FIXME: [NETWORK] Can we make this run just once, and cache the results,
            // and only update if the max value might exceed, or if the time updates?

            let time_start = -(network_widget_state.current_display_time as f64);

            let display_time_labels = vec![
                Span::styled(
                    format!("{}s", network_widget_state.current_display_time / 1000),
                    self.colours.graph_style,
                ),
                Span::styled("0s".to_string(), self.colours.graph_style),
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
                        .labels(display_time_labels)
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
                    .labels(display_time_labels)
            };

            // Find the maximal rx/tx so we know how to scale, and return it.
            let max_entry = get_max_entry(
                network_data_rx,
                network_data_tx,
                time_start,
                &app_state.app_config_fields.network_scale_type,
                app_state.app_config_fields.network_use_binary_prefix,
            );

            let (max_range, labels) = adjust_network_data_point(
                max_entry,
                &app_state.app_config_fields.network_scale_type,
                &app_state.app_config_fields.network_unit_type,
                app_state.app_config_fields.network_use_binary_prefix,
            );

            let y_axis_labels = labels
                .iter()
                .map(|label| Span::styled(label, self.colours.graph_style))
                .collect::<Vec<_>>();
            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .bounds([0.0, max_range])
                .labels(y_axis_labels);

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let border_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Network ── Esc to go back ";
                Spans::from(vec![
                    Span::styled(" Network ", self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to go back ",
                            "─".repeat(usize::from(draw_loc.width).saturating_sub(
                                UnicodeSegmentation::graphemes(TITLE_BASE, true).count() + 2
                            ))
                        ),
                        border_style,
                    ),
                ])
            } else {
                Spans::from(Span::styled(" Network ", self.colours.widget_title_style))
            };

            let legend_constraints = if hide_legend {
                (Constraint::Ratio(0, 1), Constraint::Ratio(0, 1))
            } else {
                (Constraint::Ratio(1, 1), Constraint::Ratio(3, 4))
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
                Chart::new(dataset)
                    .block(
                        Block::default()
                            .title(title)
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

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
            0
        } else {
            app_state.app_config_fields.table_gap
        };

        let rx_display = &app_state.canvas_data.rx_display;
        let tx_display = &app_state.canvas_data.tx_display;
        let total_rx_display = &app_state.canvas_data.total_rx_display;
        let total_tx_display = &app_state.canvas_data.total_tx_display;

        // Gross but I need it to work...
        let total_network = vec![vec![
            Text::raw(rx_display),
            Text::raw(tx_display),
            Text::raw(total_rx_display),
            Text::raw(total_tx_display),
        ]];
        let mapped_network = total_network
            .into_iter()
            .map(|val| Row::new(val).style(self.colours.text_style));

        // Calculate widths
        let intrinsic_widths = get_column_widths(
            draw_loc.width,
            &[None, None, None, None],
            &(NETWORK_HEADERS_LENS
                .iter()
                .map(|s| Some(*s))
                .collect::<Vec<_>>()),
            &[Some(0.25); 4],
            &(NETWORK_HEADERS_LENS
                .iter()
                .map(|s| Some(*s))
                .collect::<Vec<_>>()),
            true,
        );

        // Draw
        f.render_widget(
            Table::new(mapped_network)
                .header(
                    Row::new(NETWORK_HEADERS.to_vec())
                        .style(self.colours.table_header_style)
                        .bottom_margin(table_gap),
                )
                .block(Block::default().borders(Borders::ALL).border_style(
                    if app_state.current_widget.widget_id == widget_id {
                        self.colours.highlighted_border_style
                    } else {
                        self.colours.border_style
                    },
                ))
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
