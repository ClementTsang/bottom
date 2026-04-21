use crate::{collection::network::NetworkHarvest, utils::data_units::convert_bytes};

pub mod cpu_basic;
pub mod cpu_graph;
pub mod disk_table;
pub mod mem_basic;
pub mod mem_graph;
pub mod network_basic;
pub mod network_graph;
pub mod process_table;
pub mod temperature_table;

#[cfg(feature = "battery")]
pub mod battery_display;

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
