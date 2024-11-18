use super::common::DataCollector;

/// A fallback [`DataCollector`] for unsupported systems
/// that does nothing.
pub struct FallbackDataCollector {}

impl DataCollector for FallbackDataCollector {}
