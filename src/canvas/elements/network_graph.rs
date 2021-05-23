use once_cell::sync::Lazy;
use std::cmp::max;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::{AppState, AxisScaling},
    canvas::{
        drawing_utils::{get_column_widths, interpolate_points},
        Painter,
    },
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
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    );

    fn draw_network_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
        hide_legend: bool,
    );

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    );
}

impl NetworkGraphWidget for Painter {
    fn draw_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
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
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
        hide_legend: bool,
    ) {
        /// Point is of time, data
        type Point = (f64, f64);

        /// Returns the max data point and time given a time.
        fn get_max_entry(
            rx: &[Point], tx: &[Point], time_start: f64, network_scale_type: &AxisScaling,
            network_use_binary_prefix: bool,
        ) -> (f64, f64) {
            /// Determines a "fake" max value in circumstances where we couldn't find one from the data.
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
                (None, None) => (
                    time_start,
                    calculate_missing_max(network_scale_type, network_use_binary_prefix),
                ),
                (None, Some(filtered_tx)) => {
                    match filtered_tx
                        .iter()
                        .max_by(|(_, data_a), (_, data_b)| get_ordering(data_a, data_b, false))
                    {
                        Some((best_time, max_val)) => {
                            if *max_val == 0.0 {
                                (
                                    time_start,
                                    calculate_missing_max(
                                        network_scale_type,
                                        network_use_binary_prefix,
                                    ),
                                )
                            } else {
                                (*best_time, *max_val)
                            }
                        }
                        None => (
                            time_start,
                            calculate_missing_max(network_scale_type, network_use_binary_prefix),
                        ),
                    }
                }
                (Some(filtered_rx), None) => {
                    match filtered_rx
                        .iter()
                        .max_by(|(_, data_a), (_, data_b)| get_ordering(data_a, data_b, false))
                    {
                        Some((best_time, max_val)) => {
                            if *max_val == 0.0 {
                                (
                                    time_start,
                                    calculate_missing_max(
                                        network_scale_type,
                                        network_use_binary_prefix,
                                    ),
                                )
                            } else {
                                (*best_time, *max_val)
                            }
                        }
                        None => (
                            time_start,
                            calculate_missing_max(network_scale_type, network_use_binary_prefix),
                        ),
                    }
                }
                (Some(filtered_rx), Some(filtered_tx)) => {
                    match filtered_rx
                        .iter()
                        .chain(filtered_tx)
                        .max_by(|(_, data_a), (_, data_b)| get_ordering(data_a, data_b, false))
                    {
                        Some((best_time, max_val)) => {
                            if *max_val == 0.0 {
                                (
                                    *best_time,
                                    calculate_missing_max(
                                        network_scale_type,
                                        network_use_binary_prefix,
                                    ),
                                )
                            } else {
                                (*best_time, *max_val)
                            }
                        }
                        None => (
                            time_start,
                            calculate_missing_max(network_scale_type, network_use_binary_prefix),
                        ),
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
            // - Main goal is to maximize the amount of information displayed given a specific height.
            //   We don't want to drown out some data if the ranges are too far though!  Nor do we want to filter
            //   out too much data...
            // - Change the y-axis unit (kilo/kibi, mega/mebi...) dynamically based on max load.
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
            //
            // ===
            //
            // For log data, we just use the old method of log intervals (kilo/mega/giga/etc.).  Keep it nice and simple.

            // Now just check the largest unit we correspond to... then proceed to build some entries from there!

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

                    // Finally, build an acceptable range starting from there, using the given height!
                    // Note we try to put more of a weight on the bottom section vs. the top, since the top has less data.

                    let base_unit = max_value_scaled;
                    let labels: Vec<String> = vec![
                        format!("0{}{}", unit_prefix, unit_type),
                        format!("{:.1}", base_unit * 0.5),
                        format!("{:.1}", base_unit),
                        format!("{:.1}", base_unit * 1.5),
                    ]
                    .into_iter()
                    .map(|s| format!("{:>5}", s)) // Pull 5 as the longest legend value is generally going to be 5 digits (if they somehow hit over 5 terabits per second)
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

        if let Some(network_widget_state) = app_state.net_state.widget_states.get_mut(&widget_id) {
            let network_data_rx: &mut [(f64, f64)] = &mut app_state.canvas_data.network_data_rx;
            let network_data_tx: &mut [(f64, f64)] = &mut app_state.canvas_data.network_data_tx;

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
                Axis::default().bounds([time_start, 0.0])
            } else if let Some(time) = network_widget_state.autohide_timer {
                if std::time::Instant::now().duration_since(time).as_millis()
                    < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
                {
                    Axis::default()
                        .bounds([time_start, 0.0])
                        .style(self.colours.graph_style)
                        .labels(display_time_labels)
                } else {
                    network_widget_state.autohide_timer = None;
                    Axis::default().bounds([time_start, 0.0])
                }
            } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
                Axis::default().bounds([time_start, 0.0])
            } else {
                Axis::default()
                    .bounds([time_start, 0.0])
                    .style(self.colours.graph_style)
                    .labels(display_time_labels)
            };

            // Interpolate a point for rx and tx between the last value outside of the left bounds and the first value
            // inside it.
            // Because we assume it is all in order for... basically all our code, we can't just append it,
            // and insertion in the middle seems.  So instead, we swap *out* the value that is outside with our
            // interpolated point, draw and do whatever calculations, then swap back in the old value!
            //
            // Note there is some re-used work here!  For potential optimizations, we could re-use some work here in/from
            // get_max_entry...
            let interpolated_rx_point = if let Some(rx_end_pos) = network_data_rx
                .iter()
                .position(|(time, _data)| *time >= time_start)
            {
                if rx_end_pos > 1 {
                    let rx_start_pos = rx_end_pos - 1;
                    let outside_rx_point = network_data_rx.get(rx_start_pos);
                    let inside_rx_point = network_data_rx.get(rx_end_pos);

                    if let (Some(outside_rx_point), Some(inside_rx_point)) =
                        (outside_rx_point, inside_rx_point)
                    {
                        let old = *outside_rx_point;

                        let new_point = (
                            time_start,
                            interpolate_points(outside_rx_point, inside_rx_point, time_start),
                        );

                        // debug!(
                        //     "Interpolated between {:?} and {:?}, got rx for time {:?}: {:?}",
                        //     outside_rx_point, inside_rx_point, time_start, new_point
                        // );

                        if let Some(to_replace) = network_data_rx.get_mut(rx_start_pos) {
                            *to_replace = new_point;
                            Some((rx_start_pos, old))
                        } else {
                            None // Failed to get mutable reference.
                        }
                    } else {
                        None // Point somehow doesn't exist in our network_data_rx
                    }
                } else {
                    None // Point is already "leftmost", no need to interpolate.
                }
            } else {
                None // There is no point.
            };

            let interpolated_tx_point = if let Some(tx_end_pos) = network_data_tx
                .iter()
                .position(|(time, _data)| *time >= time_start)
            {
                if tx_end_pos > 1 {
                    let tx_start_pos = tx_end_pos - 1;
                    let outside_tx_point = network_data_tx.get(tx_start_pos);
                    let inside_tx_point = network_data_tx.get(tx_end_pos);

                    if let (Some(outside_tx_point), Some(inside_tx_point)) =
                        (outside_tx_point, inside_tx_point)
                    {
                        let old = *outside_tx_point;

                        let new_point = (
                            time_start,
                            interpolate_points(outside_tx_point, inside_tx_point, time_start),
                        );

                        if let Some(to_replace) = network_data_tx.get_mut(tx_start_pos) {
                            *to_replace = new_point;
                            Some((tx_start_pos, old))
                        } else {
                            None // Failed to get mutable reference.
                        }
                    } else {
                        None // Point somehow doesn't exist in our network_data_tx
                    }
                } else {
                    None // Point is already "leftmost", no need to interpolate.
                }
            } else {
                None // There is no point.
            };

            // TODO: Cache network results: Only update if:
            // - Force update (includes time interval change)
            // - Old max time is off screen
            // - A new time interval is better and does not fit (check from end of vector to last checked; we only want to update if it is TOO big!)

            // Find the maximal rx/tx so we know how to scale, and return it.

            let (_best_time, max_entry) = get_max_entry(
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

            // Cache results.
            // network_widget_state.draw_max_range_cache = max_range;
            // network_widget_state.draw_time_start_cache = best_time;
            // network_widget_state.draw_labels_cache = labels;

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

            // TODO: Add support for clicking on legend to only show that value on chart.
            let dataset = if app_state.app_config_fields.use_old_network_legend && !hide_legend {
                vec![
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
                    Dataset::default()
                        .name(format!(
                            "Total RX: {:7}",
                            app_state.canvas_data.total_rx_display
                        ))
                        .style(self.colours.total_rx_style),
                    Dataset::default()
                        .name(format!(
                            "Total TX: {:7}",
                            app_state.canvas_data.total_tx_display
                        ))
                        .style(self.colours.total_tx_style),
                ]
            } else {
                vec![
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
                ]
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

            // Now if you're done, reset any interpolated points!
            if let Some((index, old_value)) = interpolated_rx_point {
                if let Some(to_replace) = network_data_rx.get_mut(index) {
                    *to_replace = old_value;
                }
            }

            if let Some((index, old_value)) = interpolated_tx_point {
                if let Some(to_replace) = network_data_tx.get_mut(index) {
                    *to_replace = old_value;
                }
            }
        }
    }

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
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
