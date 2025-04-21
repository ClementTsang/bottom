//! How we manage data internally.

mod time_series;
pub use time_series::{TimeSeriesData, Values};

mod process;
pub use process::ProcessData;

mod store;
pub use store::*;

mod temperature;
pub use temperature::*;
