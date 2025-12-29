//! A chart displaying data in the y-axis over time in the x-axis. A "base" version is available,
//! based on a vendored version of ratatui's charts, as are variants for common use cases.

mod base;
mod variants;
mod vendored;

pub(crate) use base::*;
pub(crate) use variants::percent::PercentTimeGraph;
pub(crate) use vendored::*;
