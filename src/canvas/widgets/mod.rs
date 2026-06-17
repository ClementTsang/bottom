use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;
use timeless::data::ChunkedData;
use tui::style::Style;

use crate::{
    collection::network::NetworkHarvest, utils::data_units::convert_bytes, widgets::DiskWidgetData,
};

pub mod cpu_basic;
pub mod cpu_graph;
pub mod disk_io_graph;
pub mod disk_space_graph;
pub mod disk_table;
pub mod mem_basic;
pub mod mem_graph;
pub mod network_basic;
pub mod network_graph;
pub mod process_table;
pub mod temperature_graph;
pub mod temperature_table;

#[cfg(feature = "battery")]
pub mod battery_display;

/// Map device names to mount points for the currently-mounted disks.
pub(super) fn disk_mount_map(disks: &[DiskWidgetData]) -> FxHashMap<&str, &str> {
    disks
        .iter()
        .map(|d| (d.name.as_str(), d.mount_point.as_str()))
        .collect()
}

/// Pick the style at `idx`, cycling through `styles`, or the default style if
/// `styles` is empty.
pub(super) fn cycle_style(styles: &[Style], idx: usize) -> Style {
    if styles.is_empty() {
        Style::default()
    } else {
        styles[idx % styles.len()]
    }
}

/// Returns true if `data` has at least one real (non-gap) data point within the
/// visible time window defined by `current_display_time` milliseconds from the end
/// of `times`.
pub(super) fn has_data_in_window<F: Copy + Default + Into<f64>>(
    data: &ChunkedData<F>, times: &[Instant], current_display_time: u64,
) -> bool {
    let Some(&last_time) = times.last() else {
        return false;
    };
    let display_duration = Duration::from_millis(current_display_time);
    let oldest = last_time.checked_sub(display_duration).unwrap_or(last_time);
    data.iter_along_base(times)
        .next_back()
        .is_some_and(|(t, _)| *t >= oldest)
}

/// Helper struct to hold packet-related data
pub(super) struct PacketInfo {
    /// Current received packet rate.
    pub(super) rx_packet_rate: u64,

    /// Current transmitted packet rate.
    pub(super) tx_packet_rate: u64,

    /// Average received packet size in bytes, converted to the nearest unit.
    pub(super) avg_rx_packet_size: (f64, &'static str),

    /// Average transmitted packet size in bytes, converted to the nearest unit.
    pub(super) avg_tx_packet_size: (f64, &'static str),
}

/// Calculate packet information from network data.
pub(super) fn calculate_packet_info(
    network_latest_data: &NetworkHarvest, use_binary_prefix: bool,
) -> PacketInfo {
    let rx_packet_rate = network_latest_data.rx_packets;
    let tx_packet_rate = network_latest_data.tx_packets;

    // Calculate average packet size (bytes per packet)
    let avg_rx_packet_size = if network_latest_data.rx_packets > 0 {
        (network_latest_data.rx as f64 / 8.0) / network_latest_data.rx_packets as f64 // Convert bits to bytes
    } else {
        0.0
    };

    let avg_tx_packet_size = if network_latest_data.tx_packets > 0 {
        (network_latest_data.tx as f64 / 8.0) / network_latest_data.tx_packets as f64 // Convert bits to bytes
    } else {
        0.0
    };

    let avg_rx_packet_size = convert_bytes(avg_rx_packet_size.round() as u64, use_binary_prefix);
    let avg_tx_packet_size = convert_bytes(avg_tx_packet_size.round() as u64, use_binary_prefix);

    PacketInfo {
        rx_packet_rate,
        tx_packet_rate,
        avg_rx_packet_size,
        avg_tx_packet_size,
    }
}
