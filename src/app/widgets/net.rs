use std::{collections::HashMap, time::Instant};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::{
        data_farmer::DataCollection, text_table::SimpleColumn, time_graph::TimeGraphData,
        AppConfigFields, AxisScaling,
    },
    canvas::Painter,
    data_conversion::convert_network_data_points,
    units::data_units::DataUnit,
    utils::gen_util::*,
};

use super::{Component, TextTable, TimeGraph, Widget};

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    // pub draw_max_range_cache: f64,
    // pub draw_labels_cache: Vec<String>,
    // pub draw_time_start_cache: f64,
    // TODO: Re-enable these when we move net details state-side!
    // pub unit_type: DataUnitTypes,
    // pub scale_type: AxisScaling,
}

impl NetWidgetState {
    pub fn init(
        current_display_time: u64,
        autohide_timer: Option<Instant>,
        // unit_type: DataUnitTypes,
        // scale_type: AxisScaling,
    ) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            // draw_max_range_cache: 0.0,
            // draw_labels_cache: vec![],
            // draw_time_start_cache: 0.0,
            // unit_type,
            // scale_type,
        }
    }
}

#[derive(Default)]
pub struct NetState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, NetWidgetState>,
}

impl NetState {
    pub fn init(widget_states: HashMap<u64, NetWidgetState>) -> Self {
        NetState {
            force_update: None,
            widget_states,
        }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut NetWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&NetWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

// --- NEW STUFF BELOW ---

/// Returns the max data point and time given a time.
fn get_max_entry(
    rx: &[(f64, f64)], tx: &[(f64, f64)], time_start: f64, network_scale_type: &AxisScaling,
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
                            calculate_missing_max(network_scale_type, network_use_binary_prefix),
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
                            calculate_missing_max(network_scale_type, network_use_binary_prefix),
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
                            calculate_missing_max(network_scale_type, network_use_binary_prefix),
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

/// A struct containing useful cached information for a [`NetGraph`].
#[derive(Clone)]
pub struct NetGraphCache {
    max_range: f64,
    labels: Vec<String>,
    time_start: f64,
}

enum NetGraphCacheState {
    Uncached,
    Cached {
        cached_area: Rect,
        data: NetGraphCache,
    },
}

/// A widget denoting network usage via a graph. This version is self-contained within a single [`TimeGraph`];
/// if you need the older one that splits into two sections, use [`OldNetGraph`], which is built on a [`NetGraph`].
///
/// As of now, this is essentially just a wrapper around a [`TimeGraph`].
pub struct NetGraph {
    /// The graph itself.  Just a [`TimeGraph`].
    graph: TimeGraph,

    // Cached details for drawing purposes; probably want to move at some point...
    draw_cache: NetGraphCacheState,

    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: String,
    pub total_tx_display: String,
    pub network_data_rx: Vec<(f64, f64)>,
    pub network_data_tx: Vec<(f64, f64)>,

    pub scale_type: AxisScaling,
    pub unit_type: DataUnit,
    pub use_binary_prefix: bool,

    hide_legend: bool,
}

impl NetGraph {
    /// Creates a new [`NetGraph`] given a [`AppConfigFields`].
    pub fn from_config(app_config_fields: &AppConfigFields) -> Self {
        let graph = TimeGraph::from_config(app_config_fields);

        Self {
            graph,
            draw_cache: NetGraphCacheState::Uncached,
            rx_display: Default::default(),
            tx_display: Default::default(),
            total_rx_display: Default::default(),
            total_tx_display: Default::default(),
            network_data_rx: Default::default(),
            network_data_tx: Default::default(),
            scale_type: app_config_fields.network_scale_type.clone(),
            unit_type: app_config_fields.network_unit_type.clone(),
            use_binary_prefix: app_config_fields.network_use_binary_prefix,
            hide_legend: false,
        }
    }

    /// Hides the legend. Only really useful for [`OldNetGraph`].
    pub fn hide_legend(mut self) -> Self {
        self.hide_legend = true;
        self
    }

    /// Updates the associated cache on a [`NetGraph`].
    pub fn set_cache(&mut self, area: Rect, max_range: f64, labels: Vec<String>, time_start: f64) {
        self.draw_cache = NetGraphCacheState::Cached {
            cached_area: area,
            data: NetGraphCache {
                max_range,
                labels,
                time_start,
            },
        }
    }

    /// Returns whether the [`NetGraph`] contains a cache from drawing.
    pub fn is_cache_valid(&self, area: Rect) -> bool {
        match &self.draw_cache {
            NetGraphCacheState::Uncached => false,
            NetGraphCacheState::Cached {
                cached_area,
                data: _,
            } => *cached_area == area,
        }
    }

    /// Returns a reference to the [`NetGraphCache`] tied to the [`NetGraph`].
    pub fn get_cache(&self) -> Option<&NetGraphCache> {
        match &self.draw_cache {
            NetGraphCacheState::Uncached => None,
            NetGraphCacheState::Cached {
                cached_area: _,
                data,
            } => Some(data),
        }
    }

    /// Returns the [`NetGraphCache`] tied to the [`NetGraph`].
    pub fn get_cache_owned(&self) -> Option<NetGraphCache> {
        match &self.draw_cache {
            NetGraphCacheState::Uncached => None,
            NetGraphCacheState::Cached {
                cached_area: _,
                data,
            } => Some(data.clone()),
        }
    }

    /// A wrapper function around checking the cache validity and setting/getting the cache.
    pub fn check_get_cache(&mut self) -> NetGraphCache {
        todo!()
    }
}

impl Component for NetGraph {
    fn bounds(&self) -> Rect {
        self.graph.bounds()
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.graph.set_bounds(new_bounds);
    }

    fn handle_key_event(
        &mut self, event: crossterm::event::KeyEvent,
    ) -> crate::app::event::EventResult {
        self.graph.handle_key_event(event)
    }

    fn handle_mouse_event(
        &mut self, event: crossterm::event::MouseEvent,
    ) -> crate::app::event::EventResult {
        self.graph.handle_mouse_event(event)
    }
}

impl Widget for NetGraph {
    fn get_pretty_name(&self) -> &'static str {
        "Network"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
    ) {
        let block = Block::default()
            .border_style(if selected {
                painter.colours.highlighted_border_style
            } else {
                painter.colours.border_style
            })
            .borders(Borders::ALL);

        let chart_data = vec![
            TimeGraphData {
                data: &self.network_data_rx,
                label: if self.hide_legend {
                    None
                } else {
                    Some(self.rx_display.clone().into())
                },
                style: painter.colours.rx_style,
            },
            TimeGraphData {
                data: &self.network_data_tx,
                label: if self.hide_legend {
                    None
                } else {
                    Some(self.tx_display.clone().into())
                },
                style: painter.colours.tx_style,
            },
        ];

        let (_best_time, max_entry) = get_max_entry(
            &self.network_data_rx,
            &self.network_data_tx,
            -(self.graph.get_current_display_time() as f64),
            &self.scale_type,
            self.use_binary_prefix,
        );

        let (max_range, labels) = adjust_network_data_point(
            max_entry,
            &self.scale_type,
            &self.unit_type,
            self.use_binary_prefix,
        );
        let y_bounds = [0.0, max_range];
        let y_bound_labels = labels.into_iter().map(Into::into).collect::<Vec<_>>();

        self.graph.draw_tui_chart(
            painter,
            f,
            &chart_data,
            &y_bound_labels,
            y_bounds,
            false,
            block,
            area,
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        let network_data = convert_network_data_points(
            data_collection,
            false, // TODO: I think the is_frozen here is also useless; see mem and cpu
            false,
            &self.scale_type,
            &self.unit_type,
            self.use_binary_prefix,
        );
        self.network_data_rx = network_data.rx;
        self.network_data_tx = network_data.tx;
        self.rx_display = network_data.rx_display;
        self.tx_display = network_data.tx_display;
        if let Some(total_rx_display) = network_data.total_rx_display {
            self.total_rx_display = total_rx_display;
        }
        if let Some(total_tx_display) = network_data.total_tx_display {
            self.total_tx_display = total_tx_display;
        }
    }
}

/// A widget denoting network usage via a graph and a separate, single row table. This is built on [`NetGraph`],
/// and the main difference is that it also contains a bounding box for the graph + text.
pub struct OldNetGraph {
    net_graph: NetGraph,
    table: TextTable,
    bounds: Rect,
}

impl OldNetGraph {
    /// Creates a new [`OldNetGraph`] from a [`AppConfigFields`].
    pub fn from_config(config: &AppConfigFields) -> Self {
        Self {
            net_graph: NetGraph::from_config(config).hide_legend(),
            table: TextTable::new(vec![
                SimpleColumn::new_flex("RX".into(), 0.25),
                SimpleColumn::new_flex("TX".into(), 0.25),
                SimpleColumn::new_flex("Total RX".into(), 0.25),
                SimpleColumn::new_flex("Total TX".into(), 0.25),
            ])
            .unselectable(),
            bounds: Rect::default(),
        }
    }
}

impl Component for OldNetGraph {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(
        &mut self, event: crossterm::event::KeyEvent,
    ) -> crate::app::event::EventResult {
        self.net_graph.handle_key_event(event)
    }

    fn handle_mouse_event(
        &mut self, event: crossterm::event::MouseEvent,
    ) -> crate::app::event::EventResult {
        self.net_graph.handle_mouse_event(event)
    }
}

impl Widget for OldNetGraph {
    fn get_pretty_name(&self) -> &'static str {
        "Network"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
    ) {
        const CONSTRAINTS: [Constraint; 2] = [Constraint::Min(0), Constraint::Length(4)];

        let split_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(CONSTRAINTS)
            .split(area);

        let graph_area = split_area[0];
        let table_area = split_area[1];

        self.net_graph.draw(painter, f, graph_area, selected);

        let table_block = Block::default()
            .border_style(if selected {
                painter.colours.highlighted_border_style
            } else {
                painter.colours.border_style
            })
            .borders(Borders::ALL);
        self.table.draw_tui_table(
            painter,
            f,
            &vec![vec![
                (
                    self.net_graph.rx_display.clone().into(),
                    None,
                    Some(painter.colours.rx_style),
                ),
                (
                    self.net_graph.tx_display.clone().into(),
                    None,
                    Some(painter.colours.tx_style),
                ),
                (self.net_graph.total_rx_display.clone().into(), None, None),
                (self.net_graph.total_tx_display.clone().into(), None, None),
            ]],
            table_block,
            table_area,
            selected,
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        let network_data = convert_network_data_points(
            data_collection,
            false, // TODO: I think the is_frozen here is also useless; see mem and cpu
            true,
            &self.net_graph.scale_type,
            &self.net_graph.unit_type,
            self.net_graph.use_binary_prefix,
        );
        self.net_graph.network_data_rx = network_data.rx;
        self.net_graph.network_data_tx = network_data.tx;
        self.net_graph.rx_display = network_data.rx_display;
        self.net_graph.tx_display = network_data.tx_display;
        if let Some(total_rx_display) = network_data.total_rx_display {
            self.net_graph.total_rx_display = total_rx_display;
        }
        if let Some(total_tx_display) = network_data.total_tx_display {
            self.net_graph.total_tx_display = total_tx_display;
        }
    }
}
