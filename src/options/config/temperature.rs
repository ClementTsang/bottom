use serde::Deserialize;

use super::IgnoreList;

/// Temperature configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct TempConfig {
    /// A filter over the sensor names.
    pub sensor_filter: Option<IgnoreList>,
}
