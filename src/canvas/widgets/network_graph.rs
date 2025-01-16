use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    text::Text,
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{
    app::{App, AxisScaling},
    canvas::{
        components::{
            time_chart::Point,
            time_graph::{GraphData, TimeGraph},
        },
        drawing_utils::should_hide_x_label,
        Painter,
    },
    utils::{data_prefixes::*, data_units::DataUnit, general::partial_ordering},
};

impl Painter {
    pub fn draw_network(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if app_state.app_config_fields.use_old_network_legend {
            const LEGEND_HEIGHT: u16 = 4;
            let network_chunk = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Length(draw_loc.height.saturating_sub(LEGEND_HEIGHT)),
                    Constraint::Length(LEGEND_HEIGHT),
                ])
                .split(draw_loc);

            self.draw_network_graph(f, app_state, network_chunk[0], widget_id, true);
            self.draw_network_labels(f, app_state, network_chunk[1], widget_id);
        } else {
            self.draw_network_graph(f, app_state, draw_loc, widget_id, false);
        }

        if app_state.should_get_widget_bounds() {
            // Update draw loc in widget map
            // Note that in both cases, we always go to the same widget id so it's fine to
            // do it like this lol.
            if let Some(network_widget) = app_state.widget_map.get_mut(&widget_id) {
                network_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                network_widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }

    pub fn draw_network_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
        not_full_screen: bool,
    ) {
        if let Some(network_widget_state) =
            app_state.states.net_state.widget_states.get_mut(&widget_id)
        {
            let shared_data = app_state.shared_data.data();
            let network_latest_data = &(shared_data.network_harvest);
            let rx_points = &(shared_data.timeseries_data.rx);
            let tx_points = &(shared_data.timeseries_data.tx);
            let time = &(shared_data.timeseries_data.time);
            let last_time = shared_data.current_instant;

            // FIXME: THIS IS TEMPORARY.
            let network_data_rx = rx_points
                .iter_along_base(time)
                .map(|i| {
                    i.map(|(t, v)| {
                        (
                            last_time.duration_since(*t).as_millis() as f64,
                            get_network_point(
                                *v,
                                &app_state.app_config_fields.network_scale_type,
                                &app_state.app_config_fields.network_unit_type,
                                app_state.app_config_fields.network_use_binary_prefix,
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let network_data_tx = tx_points
                .iter_along_base(time)
                .map(|i| {
                    i.map(|(t, v)| {
                        (
                            last_time.duration_since(*t).as_millis() as f64,
                            get_network_point(
                                *v,
                                &app_state.app_config_fields.network_scale_type,
                                &app_state.app_config_fields.network_unit_type,
                                app_state.app_config_fields.network_use_binary_prefix,
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let time_start = -(network_widget_state.current_display_time as f64);
            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let x_bounds = [0, network_widget_state.current_display_time];
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut network_widget_state.autohide_timer,
                draw_loc,
            );

            // TODO: Cache network results: Only update if:
            // - Force update (includes time interval change)
            // - Old max time is off screen
            // - A new time interval is better and does not fit (check from end of vector to
            //   last checked; we only want to update if it is TOO big!)

            // Find the maximal rx/tx so we know how to scale, and return it.
            let max_entry = get_max_entry(
                &network_data_rx,
                &network_data_tx,
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

            let y_labels = labels.iter().map(|label| label.into()).collect::<Vec<_>>();
            let y_bounds = [0.0, max_range];

            let legend_constraints = if not_full_screen {
                (Constraint::Ratio(0, 1), Constraint::Ratio(0, 1))
            } else {
                (Constraint::Ratio(1, 1), Constraint::Ratio(3, 4))
            };

            // TODO: Add support for clicking on legend to only show that value on chart.

            let use_binary_prefix = app_state.app_config_fields.network_use_binary_prefix;

            let rx = get_unit_prefix(network_latest_data.rx, use_binary_prefix);
            let tx = get_unit_prefix(network_latest_data.tx, use_binary_prefix);
            let rx_label = format!("RX: {:.1}{}", rx.0, rx.1);
            let tx_label = format!("TX: {:.1}{}", tx.0, tx.1);

            // TODO: This behaviour is pretty weird, we should probably just make it so if you use old network legend
            // and go full screen you don't get this weird state.
            let points = if app_state.app_config_fields.use_old_network_legend && !not_full_screen {
                let total_rx = convert_bytes(network_latest_data.total_rx, use_binary_prefix);
                let total_tx = convert_bytes(network_latest_data.total_tx, use_binary_prefix);
                let total_rx_label = format!("Total RX{:.1}{}", total_rx.0, total_rx.1);
                let total_tx_label = format!("Total TX{:.1}{}", total_tx.0, total_tx.1);

                vec![
                    GraphData {
                        points: &network_data_rx,
                        style: self.styles.rx_style,
                        name: Some(rx_label.into()),
                    },
                    GraphData {
                        points: &network_data_tx,
                        style: self.styles.tx_style,
                        name: Some(tx_label.into()),
                    },
                    GraphData {
                        points: &[],
                        style: self.styles.total_rx_style,
                        name: Some(total_rx_label.into()),
                    },
                    GraphData {
                        points: &[],
                        style: self.styles.total_tx_style,
                        name: Some(total_tx_label.into()),
                    },
                ]
            } else {
                vec![
                    GraphData {
                        points: &network_data_rx,
                        style: self.styles.rx_style,
                        name: Some(rx_label.into()),
                    },
                    GraphData {
                        points: &network_data_tx,
                        style: self.styles.tx_style,
                        name: Some(tx_label.into()),
                    },
                ]
            };

            let marker = if app_state.app_config_fields.use_dot {
                Marker::Dot
            } else {
                Marker::Braille
            };

            TimeGraph {
                x_bounds,
                hide_x_labels,
                y_bounds,
                y_labels: &y_labels,
                graph_style: self.styles.graph_style,
                border_style,
                border_type: self.styles.border_type,
                title: " Network ".into(),
                is_selected: app_state.current_widget.widget_id == widget_id,
                is_expanded: app_state.is_expanded,
                title_style: self.styles.widget_title_style,
                legend_position: app_state.app_config_fields.network_legend_position,
                legend_constraints: Some(legend_constraints),
                marker,
            }
            .draw_time_graph(f, draw_loc, &points);
        }
    }

    fn draw_network_labels(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        const NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "Total RX", "Total TX"];

        let network_latest_data = &(app_state.shared_data.data().network_harvest);
        let use_binary_prefix = app_state.app_config_fields.network_use_binary_prefix;

        let rx = get_unit_prefix(network_latest_data.rx, use_binary_prefix);
        let tx = get_unit_prefix(network_latest_data.tx, use_binary_prefix);
        let rx_label = format!("RX: {:.1}{}", rx.0, rx.1);
        let tx_label = format!("TX: {:.1}{}", tx.0, tx.1);

        let total_rx = convert_bytes(network_latest_data.total_rx, use_binary_prefix);
        let total_tx = convert_bytes(network_latest_data.total_tx, use_binary_prefix);
        let total_rx_label = format!("Total RX{:.1}{}", total_rx.0, total_rx.1);
        let total_tx_label = format!("Total TX{:.1}{}", total_tx.0, total_tx.1);

        // Gross but I need it to work...
        let total_network = vec![Row::new([
            Text::styled(rx_label, self.styles.rx_style),
            Text::styled(tx_label, self.styles.tx_style),
            Text::styled(total_rx_label, self.styles.total_rx_style),
            Text::styled(total_tx_label, self.styles.total_tx_style),
        ])];

        // Draw
        f.render_widget(
            Table::new(
                total_network,
                &((std::iter::repeat(draw_loc.width.saturating_sub(2) / 4))
                    .take(4)
                    .map(Constraint::Length)
                    .collect::<Vec<_>>()),
            )
            .header(Row::new(NETWORK_HEADERS).style(self.styles.table_header_style))
            .block(Block::default().borders(Borders::ALL).border_style(
                if app_state.current_widget.widget_id == widget_id {
                    self.styles.highlighted_border_style
                } else {
                    self.styles.border_style
                },
            ))
            .style(self.styles.text_style),
            draw_loc,
        );
    }
}

/// Returns the max data point and time given a time.
fn get_max_entry(
    rx: &[Point], tx: &[Point], time_start: f64, network_scale_type: &AxisScaling,
    network_use_binary_prefix: bool,
) -> f64 {
    /// Determines a "fake" max value in circumstances where we couldn't find
    /// one from the data.
    fn calculate_missing_max(
        network_scale_type: &AxisScaling, network_use_binary_prefix: bool,
    ) -> f64 {
        match network_scale_type {
            AxisScaling::Log => {
                if network_use_binary_prefix {
                    LOG_KIBI_LIMIT
                } else {
                    LOG_KILO_LIMIT
                }
            }
            AxisScaling::Linear => {
                if network_use_binary_prefix {
                    KIBI_LIMIT_F64
                } else {
                    KILO_LIMIT_F64
                }
            }
        }
    }

    // First, let's shorten our ranges to actually look.  We can abuse the fact that
    // our rx and tx arrays are sorted, so we can short-circuit our search to
    // filter out only the relevant data points...
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
        (None, None) => calculate_missing_max(network_scale_type, network_use_binary_prefix),
        (None, Some(filtered_tx)) => {
            match filtered_tx
                .iter()
                .max_by(|(_, data_a), (_, data_b)| partial_ordering(data_a, data_b))
            {
                Some((_best_time, max_val)) => {
                    if *max_val == 0.0 {
                        calculate_missing_max(network_scale_type, network_use_binary_prefix)
                    } else {
                        *max_val
                    }
                }
                None => calculate_missing_max(network_scale_type, network_use_binary_prefix),
            }
        }
        (Some(filtered_rx), None) => {
            match filtered_rx
                .iter()
                .max_by(|(_, data_a), (_, data_b)| partial_ordering(data_a, data_b))
            {
                Some((_best_time, max_val)) => {
                    if *max_val == 0.0 {
                        calculate_missing_max(network_scale_type, network_use_binary_prefix)
                    } else {
                        *max_val
                    }
                }
                None => calculate_missing_max(network_scale_type, network_use_binary_prefix),
            }
        }
        (Some(filtered_rx), Some(filtered_tx)) => {
            match filtered_rx
                .iter()
                .chain(filtered_tx)
                .max_by(|(_, data_a), (_, data_b)| partial_ordering(data_a, data_b))
            {
                Some((_best_time, max_val)) => {
                    if *max_val == 0.0 {
                        calculate_missing_max(network_scale_type, network_use_binary_prefix)
                    } else {
                        *max_val
                    }
                }
                None => calculate_missing_max(network_scale_type, network_use_binary_prefix),
            }
        }
    }
}

/// Returns the required max data point and labels.
fn adjust_network_data_point(
    max_entry: f64, network_scale_type: &AxisScaling, network_unit_type: &DataUnit,
    network_use_binary_prefix: bool,
) -> (f64, Vec<String>) {
    // So, we're going with an approach like this for linear data:
    // - Main goal is to maximize the amount of information displayed given a
    //   specific height. We don't want to drown out some data if the ranges are too
    //   far though!  Nor do we want to filter out too much data...
    // - Change the y-axis unit (kilo/kibi, mega/mebi...) dynamically based on max
    //   load.
    //
    // The idea is we take the top value, build our scale such that each "point" is
    // a scaled version of that. So for example, let's say I use 390 Mb/s.  If I
    // drew 4 segments, it would be 97.5, 195, 292.5, 390, and
    // probably something like 438.75?
    //
    // So, how do we do this in ratatui?  Well, if we  are using intervals that tie
    // in perfectly to the max value we want... then it's actually not that
    // hard.  Since ratatui accepts a vector as labels and will properly space
    // them all out... we just work with that and space it out properly.
    //
    // Dynamic chart idea based off of FreeNAS's chart design.
    //
    // ===
    //
    // For log data, we just use the old method of log intervals
    // (kilo/mega/giga/etc.).  Keep it nice and simple.

    // Now just check the largest unit we correspond to... then proceed to build
    // some entries from there!

    let unit_char = match network_unit_type {
        DataUnit::Byte => "B",
        DataUnit::Bit => "b",
    };

    match network_scale_type {
        AxisScaling::Linear => {
            let (k_limit, m_limit, g_limit, t_limit) = if network_use_binary_prefix {
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
            };

            let bumped_max_entry = max_entry * 1.5; // We use the bumped up version to calculate our unit type.
            let (max_value_scaled, unit_prefix, unit_type): (f64, &str, &str) =
                if bumped_max_entry < k_limit {
                    (max_entry, "", unit_char)
                } else if bumped_max_entry < m_limit {
                    (
                        max_entry / k_limit,
                        if network_use_binary_prefix { "Ki" } else { "K" },
                        unit_char,
                    )
                } else if bumped_max_entry < g_limit {
                    (
                        max_entry / m_limit,
                        if network_use_binary_prefix { "Mi" } else { "M" },
                        unit_char,
                    )
                } else if bumped_max_entry < t_limit {
                    (
                        max_entry / g_limit,
                        if network_use_binary_prefix { "Gi" } else { "G" },
                        unit_char,
                    )
                } else {
                    (
                        max_entry / t_limit,
                        if network_use_binary_prefix { "Ti" } else { "T" },
                        unit_char,
                    )
                };

            // Finally, build an acceptable range starting from there, using the given
            // height! Note we try to put more of a weight on the bottom section
            // vs. the top, since the top has less data.

            let base_unit = max_value_scaled;
            let labels: Vec<String> = vec![
                format!("0{unit_prefix}{unit_type}"),
                format!("{:.1}", base_unit * 0.5),
                format!("{:.1}", base_unit),
                format!("{:.1}", base_unit * 1.5),
            ]
            .into_iter()
            .map(|s| format!("{s:>5}")) // Pull 5 as the longest legend value is generally going to be 5 digits (if they somehow
            // hit over 5 terabits per second)
            .collect();

            (bumped_max_entry, labels)
        }
        AxisScaling::Log => {
            let (m_limit, g_limit, t_limit) = if network_use_binary_prefix {
                (LOG_MEBI_LIMIT, LOG_GIBI_LIMIT, LOG_TEBI_LIMIT)
            } else {
                (LOG_MEGA_LIMIT, LOG_GIGA_LIMIT, LOG_TERA_LIMIT)
            };

            fn get_zero(network_use_binary_prefix: bool, unit_char: &str) -> String {
                format!(
                    "{}0{}",
                    if network_use_binary_prefix { "  " } else { " " },
                    unit_char
                )
            }

            fn get_k(network_use_binary_prefix: bool, unit_char: &str) -> String {
                format!(
                    "1{}{}",
                    if network_use_binary_prefix { "Ki" } else { "K" },
                    unit_char
                )
            }

            fn get_m(network_use_binary_prefix: bool, unit_char: &str) -> String {
                format!(
                    "1{}{}",
                    if network_use_binary_prefix { "Mi" } else { "M" },
                    unit_char
                )
            }

            fn get_g(network_use_binary_prefix: bool, unit_char: &str) -> String {
                format!(
                    "1{}{}",
                    if network_use_binary_prefix { "Gi" } else { "G" },
                    unit_char
                )
            }

            fn get_t(network_use_binary_prefix: bool, unit_char: &str) -> String {
                format!(
                    "1{}{}",
                    if network_use_binary_prefix { "Ti" } else { "T" },
                    unit_char
                )
            }

            fn get_p(network_use_binary_prefix: bool, unit_char: &str) -> String {
                format!(
                    "1{}{}",
                    if network_use_binary_prefix { "Pi" } else { "P" },
                    unit_char
                )
            }

            if max_entry < m_limit {
                (
                    m_limit,
                    vec![
                        get_zero(network_use_binary_prefix, unit_char),
                        get_k(network_use_binary_prefix, unit_char),
                        get_m(network_use_binary_prefix, unit_char),
                    ],
                )
            } else if max_entry < g_limit {
                (
                    g_limit,
                    vec![
                        get_zero(network_use_binary_prefix, unit_char),
                        get_k(network_use_binary_prefix, unit_char),
                        get_m(network_use_binary_prefix, unit_char),
                        get_g(network_use_binary_prefix, unit_char),
                    ],
                )
            } else if max_entry < t_limit {
                (
                    t_limit,
                    vec![
                        get_zero(network_use_binary_prefix, unit_char),
                        get_k(network_use_binary_prefix, unit_char),
                        get_m(network_use_binary_prefix, unit_char),
                        get_g(network_use_binary_prefix, unit_char),
                        get_t(network_use_binary_prefix, unit_char),
                    ],
                )
            } else {
                // I really doubt anyone's transferring beyond petabyte speeds...
                (
                    if network_use_binary_prefix {
                        LOG_PEBI_LIMIT
                    } else {
                        LOG_PETA_LIMIT
                    },
                    vec![
                        get_zero(network_use_binary_prefix, unit_char),
                        get_k(network_use_binary_prefix, unit_char),
                        get_m(network_use_binary_prefix, unit_char),
                        get_g(network_use_binary_prefix, unit_char),
                        get_t(network_use_binary_prefix, unit_char),
                        get_p(network_use_binary_prefix, unit_char),
                    ],
                )
            }
        }
    }
}

fn get_network_point(
    value: f64, scale_type: &AxisScaling, unit_type: &DataUnit, use_binary_unit: bool,
) -> f64 {
    match scale_type {
        AxisScaling::Log => {
            if use_binary_unit {
                match unit_type {
                    DataUnit::Byte => value.log2() - 4.0, // As dividing by 8 is equal to subtracting 4 in base 2!
                    DataUnit::Bit => value.log2(),
                }
            } else {
                match unit_type {
                    DataUnit::Byte => (value / 8.0).log10(),
                    DataUnit::Bit => value.log10(),
                }
            }
        }
        AxisScaling::Linear => match unit_type {
            DataUnit::Byte => value / 8.0,
            DataUnit::Bit => value,
        },
    }
}
