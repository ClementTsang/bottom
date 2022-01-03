pub mod simple_table;
pub use simple_table::*;

pub mod cpu_graph;
pub use cpu_graph::*;

pub mod disk_table;
pub use disk_table::*;

pub mod mem_graph;
pub use mem_graph::*;

pub mod net_graph;
pub use net_graph::*;

pub mod process_table;
pub use process_table::*;

pub mod temp_table;
pub use temp_table::*;

pub mod battery_table;
pub use battery_table::*;

pub mod cpu_simple;
pub use cpu_simple::*;

pub mod mem_simple;
pub use mem_simple::*;

pub mod net_simple;
pub use net_simple::*;

use crate::{app::AppConfig, canvas::Painter, data_conversion::ConvertedData, tuine::BuildContext};

pub trait AppWidget {
    fn build(
        ctx: &mut BuildContext<'_>, painter: &Painter, config: &AppConfig,
        data: &mut ConvertedData<'_>,
    ) -> Self;
}
