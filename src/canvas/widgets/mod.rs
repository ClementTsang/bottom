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
