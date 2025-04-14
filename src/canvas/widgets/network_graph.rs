use std::time::Duration;

use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    text::Text,
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    app::{App, AppConfigFields, AxisScaling},
    canvas::{
        Painter,
        components::time_graph::{AxisBound, ChartScaling, GraphData, TimeGraph},
        drawing_utils::should_hide_x_label,
    },
    utils::{
        data_units::*,
        general::{saturating_log2, saturating_log10},
    },
    widgets::NetWidgetHeightCache,
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
        full_screen: bool,
    ) {
        if let Some(network_widget_state) =
            app_state.states.net_state.widget_states.get_mut(&widget_id)
        {
            let shared_data = app_state.data_store.get_data();
            let network_latest_data = &(shared_data.network_harvest);
            let rx_points = &(shared_data.timeseries_data.rx);
            let tx_points = &(shared_data.timeseries_data.tx);
            let time = &(shared_data.timeseries_data.time);
            let time_start = -(network_widget_state.current_display_time as f64) / 1000.0;

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                &mut network_widget_state.autohide_timer,
                draw_loc,
            );

            let y_max = {
                if let Some(last_time) = time.last() {
                    // For now, just do it each time. Might want to cache this later though.

                    let (mut biggest, mut biggest_time, first_time) = {
                        let initial_first_time = *last_time
                            - Duration::from_millis(network_widget_state.current_display_time);

                        match &network_widget_state.height_cache {
                            Some(NetWidgetHeightCache {
                                best_point,
                                right_edge,
                                period,
                            }) => {
                                if *period != network_widget_state.current_display_time
                                    || best_point.0 < initial_first_time
                                {
                                    (0.0, initial_first_time, initial_first_time)
                                } else {
                                    (best_point.1, best_point.0, *right_edge)
                                }
                            }
                            None => (0.0, initial_first_time, initial_first_time),
                        }
                    };

                    for (&time, &v) in rx_points
                        .iter_along_base(time)
                        .rev()
                        .take_while(|&(&time, _)| time >= first_time)
                    {
                        if v > biggest {
                            biggest = v;
                            biggest_time = time;
                        }
                    }

                    for (&time, &v) in tx_points
                        .iter_along_base(time)
                        .rev()
                        .take_while(|&(&time, _)| time >= first_time)
                    {
                        if v > biggest {
                            biggest = v;
                            biggest_time = time;
                        }
                    }

                    network_widget_state.height_cache = Some(NetWidgetHeightCache {
                        best_point: (biggest_time, biggest),
                        right_edge: *last_time,
                        period: network_widget_state.current_display_time,
                    });

                    biggest
                } else {
                    0.0
                }
            };
            let (y_max, y_labels) = adjust_network_data_point(y_max, &app_state.app_config_fields);
            let y_bounds = AxisBound::Max(y_max);

            let legend_constraints = if full_screen {
                (Constraint::Ratio(0, 1), Constraint::Ratio(0, 1))
            } else {
                (Constraint::Ratio(1, 1), Constraint::Ratio(3, 4))
            };

            // TODO: Add support for clicking on legend to only show that value on chart.

            let use_binary_prefix = app_state.app_config_fields.network_use_binary_prefix;
            let unit_type = app_state.app_config_fields.network_unit_type;
            let unit = match unit_type {
                DataUnit::Byte => "B/s",
                DataUnit::Bit => "b/s",
            };

            let rx = get_unit_prefix(network_latest_data.rx, use_binary_prefix);
            let tx = get_unit_prefix(network_latest_data.tx, use_binary_prefix);
            let total_rx = convert_bits(network_latest_data.total_rx, use_binary_prefix);
            let total_tx = convert_bits(network_latest_data.total_tx, use_binary_prefix);

            // TODO: This behaviour is pretty weird, we should probably just make it so if you use old network legend
            // you don't do whatever this is...
            let graph_data = if app_state.app_config_fields.use_old_network_legend && !full_screen {
                let rx_label = format!("RX: {:.1}{}{}", rx.0, rx.1, unit);
                let tx_label = format!("TX: {:.1}{}{}", tx.0, tx.1, unit);
                let total_rx_label = format!("Total RX: {:.1}{}", total_rx.0, total_rx.1);
                let total_tx_label = format!("Total TX: {:.1}{}", total_tx.0, total_tx.1);

                vec![
                    GraphData::default()
                        .name(rx_label.into())
                        .time(time)
                        .values(rx_points)
                        .style(self.styles.rx_style),
                    GraphData::default()
                        .name(tx_label.into())
                        .time(time)
                        .values(tx_points)
                        .style(self.styles.tx_style),
                    GraphData::default()
                        .style(self.styles.total_rx_style)
                        .name(total_rx_label.into()),
                    GraphData::default()
                        .style(self.styles.total_tx_style)
                        .name(total_tx_label.into()),
                ]
            } else {
                let rx_label = format!("{:.1}{}{}", rx.0, rx.1, unit);
                let tx_label = format!("{:.1}{}{}", tx.0, tx.1, unit);
                let total_rx_label = format!("{:.1}{}", total_rx.0, total_rx.1);
                let total_tx_label = format!("{:.1}{}", total_tx.0, total_tx.1);

                vec![
                    GraphData::default()
                        .name(format!("RX: {:<10}  All: {}", rx_label, total_rx_label).into())
                        .time(time)
                        .values(rx_points)
                        .style(self.styles.rx_style),
                    GraphData::default()
                        .name(format!("TX: {:<10}  All: {}", tx_label, total_tx_label).into())
                        .time(time)
                        .values(tx_points)
                        .style(self.styles.tx_style),
                ]
            };

            let marker = if app_state.app_config_fields.use_dot {
                Marker::Dot
            } else {
                Marker::Braille
            };

            let scaling = match app_state.app_config_fields.network_scale_type {
                AxisScaling::Log => {
                    // TODO: I might change this behaviour later.
                    if app_state.app_config_fields.network_use_binary_prefix {
                        ChartScaling::Log2
                    } else {
                        ChartScaling::Log10
                    }
                }
                AxisScaling::Linear => ChartScaling::Linear,
            };

            TimeGraph {
                x_min: time_start,
                hide_x_labels,
                y_bounds,
                y_labels: &(y_labels.into_iter().map(Into::into).collect::<Vec<_>>()),
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
                scaling,
            }
            .draw_time_graph(f, draw_loc, graph_data);
        }
    }

    fn draw_network_labels(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        const NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "Total RX", "Total TX"];

        let network_latest_data = &(app_state.data_store.get_data().network_harvest);
        let use_binary_prefix = app_state.app_config_fields.network_use_binary_prefix;
        let unit_type = app_state.app_config_fields.network_unit_type;
        let unit = match unit_type {
            DataUnit::Byte => "B/s",
            DataUnit::Bit => "b/s",
        };

        let rx = get_unit_prefix(network_latest_data.rx, use_binary_prefix);
        let tx = get_unit_prefix(network_latest_data.tx, use_binary_prefix);

        let rx_label = format!("{:.1}{}{}", rx.0, rx.1, unit);
        let tx_label = format!("{:.1}{}{}", tx.0, tx.1, unit);

        let total_rx = convert_bits(network_latest_data.total_rx, use_binary_prefix);
        let total_tx = convert_bits(network_latest_data.total_tx, use_binary_prefix);
        let total_rx_label = format!("{:.1}{}", total_rx.0, total_rx.1);
        let total_tx_label = format!("{:.1}{}", total_tx.0, total_tx.1);

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

/// Returns the required labels.
///
/// TODO: This is _really_ ugly... also there might be a bug with certain heights and too many labels.
/// We may need to take draw height into account, either here, or in the time graph itself.
fn adjust_network_data_point(max_entry: f64, config: &AppConfigFields) -> (f64, Vec<String>) {
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
    // So, how do we do this in ratatui?  Well, if we are using intervals that tie
    // in perfectly to the max value we want... then it's actually not that
    // hard. Since ratatui accepts a vector as labels and will properly space
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

    let scale_type = config.network_scale_type;
    let use_binary_prefix = config.network_use_binary_prefix;
    let network_unit_type = config.network_unit_type;

    let unit_char = match network_unit_type {
        DataUnit::Byte => "B",
        DataUnit::Bit => "b",
    };

    match scale_type {
        AxisScaling::Linear => {
            let (k_limit, m_limit, g_limit, t_limit) = if use_binary_prefix {
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

            let max_entry_upper = max_entry * 1.5; // We use the bumped up version to calculate our unit type.
            let (max_value_scaled, unit_prefix, unit_type): (f64, &str, &str) =
                if max_entry_upper < k_limit {
                    (max_entry, "", unit_char)
                } else if max_entry_upper < m_limit {
                    (
                        max_entry / k_limit,
                        if use_binary_prefix { "Ki" } else { "K" },
                        unit_char,
                    )
                } else if max_entry_upper < g_limit {
                    (
                        max_entry / m_limit,
                        if use_binary_prefix { "Mi" } else { "M" },
                        unit_char,
                    )
                } else if max_entry_upper < t_limit {
                    (
                        max_entry / g_limit,
                        if use_binary_prefix { "Gi" } else { "G" },
                        unit_char,
                    )
                } else {
                    (
                        max_entry / t_limit,
                        if use_binary_prefix { "Ti" } else { "T" },
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
            .map(|s| {
                // Pull 5 as the longest legend value is generally going to be 5 digits (if they somehow hit over 5 terabits per second)
                format!("{s:>5}")
            })
            .collect();

            (max_entry_upper, labels)
        }
        AxisScaling::Log => {
            let (m_limit, g_limit, t_limit) = if use_binary_prefix {
                (LOG_MEBI_LIMIT, LOG_GIBI_LIMIT, LOG_TEBI_LIMIT)
            } else {
                (LOG_MEGA_LIMIT, LOG_GIGA_LIMIT, LOG_TERA_LIMIT)
            };

            // Remember to do saturating log checks as otherwise 0.0 becomes inf, and you get
            // gaps!
            let max_entry = if use_binary_prefix {
                saturating_log2(max_entry)
            } else {
                saturating_log10(max_entry)
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
                        get_zero(use_binary_prefix, unit_char),
                        get_k(use_binary_prefix, unit_char),
                        get_m(use_binary_prefix, unit_char),
                    ],
                )
            } else if max_entry < g_limit {
                (
                    g_limit,
                    vec![
                        get_zero(use_binary_prefix, unit_char),
                        get_k(use_binary_prefix, unit_char),
                        get_m(use_binary_prefix, unit_char),
                        get_g(use_binary_prefix, unit_char),
                    ],
                )
            } else if max_entry < t_limit {
                (
                    t_limit,
                    vec![
                        get_zero(use_binary_prefix, unit_char),
                        get_k(use_binary_prefix, unit_char),
                        get_m(use_binary_prefix, unit_char),
                        get_g(use_binary_prefix, unit_char),
                        get_t(use_binary_prefix, unit_char),
                    ],
                )
            } else {
                // I really doubt anyone's transferring beyond petabyte speeds...
                (
                    if use_binary_prefix {
                        LOG_PEBI_LIMIT
                    } else {
                        LOG_PETA_LIMIT
                    },
                    vec![
                        get_zero(use_binary_prefix, unit_char),
                        get_k(use_binary_prefix, unit_char),
                        get_m(use_binary_prefix, unit_char),
                        get_g(use_binary_prefix, unit_char),
                        get_t(use_binary_prefix, unit_char),
                        get_p(use_binary_prefix, unit_char),
                    ],
                )
            }
        }
    }
}
