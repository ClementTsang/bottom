//! The "widgets" of bottom.

mod battery_info;
mod cpu_graph;
mod disk_graph;
mod disk_table;
mod mem_graph;
mod network_graph;
mod process_table;
mod temperature_graph;
mod temperature_table;

pub use battery_info::*;
pub use cpu_graph::*;
pub use disk_graph::*;
pub use disk_table::*;
pub use mem_graph::*;
pub use network_graph::*;
pub use process_table::*;
pub use temperature_graph::*;
pub use temperature_table::*;
