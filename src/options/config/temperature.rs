use serde::Deserialize;

use super::IgnoreList;
use crate::widgets::TempWidgetColumn;

/// Temperature configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct TempConfig {
    /// A filter over the sensor names.
    pub(crate) sensor_filter: Option<IgnoreList>,

    /// The default sort column.
    #[serde(default)]
    pub(crate) default_sort: Option<TempWidgetColumn>,
}

#[cfg(test)]
mod tests {
    /// Test that temp enum variants that are advertised in the schema are valid.
    #[cfg(feature = "generate_schema")]
    #[test]
    fn ensure_temp_column_schema_is_accepted() {
        use strum::VariantArray;

        use crate::options::{Config, TempWidgetColumn};

        for column in TempWidgetColumn::VARIANTS {
            for &name in column.get_schema_names() {
                let config = format!("[temperature]\ndefault_sort= \"{name}\"\n");
                toml_edit::de::from_str::<Config>(&config).unwrap_or_else(|e| {
                    panic!("schema name {name:?} was rejected:\n{e}\nconfig was:\n{config}")
                });
            }
        }
    }
}
