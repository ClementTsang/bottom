//! Widgets for bottom.

pub mod battery_widget;
pub use battery_widget::Battery;

pub mod cpu_widget;
pub use cpu_widget::Cpu;

pub mod disk_widget;
pub use disk_widget::Disk;

pub mod memory_widget;
pub use memory_widget::Memory;

pub mod network_widget;
pub use network_widget::Network;

pub mod processes_widget;
pub use processes_widget::Processes;

pub mod temperature_widget;
pub use temperature_widget::Temperature;
