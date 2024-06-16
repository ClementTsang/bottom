pub mod battery_info;
pub mod cpu_graph;
pub mod disk_table;
pub mod mem_graph;
pub mod net_graph;
pub mod process_table;
pub mod temperature_table;

pub use battery_info::*;
pub use cpu_graph::*;
pub use disk_table::*;
pub use mem_graph::*;
pub use net_graph::*;
pub use process_table::*;
pub use temperature_table::*;
use tui::{layout::Rect, Frame};

/// A [`Widget`] converts raw data into something that a user can see and
/// interact with.
pub trait Widget<Data> {
    /// How to actually draw the widget to the terminal.
    fn draw(&self, f: &mut Frame<'_>, draw_location: Rect, widget_id: u64);
}
