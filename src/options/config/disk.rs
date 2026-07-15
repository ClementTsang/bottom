use serde::Deserialize;

use super::IgnoreList;
use crate::options::DiskWidgetColumn;

/// Disk configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskConfig {
    /// A filter over the disk names.
    pub(crate) name_filter: Option<IgnoreList>,

    /// A filter over the mount names.
    pub(crate) mount_filter: Option<IgnoreList>,

    /// Whether to include block devices that aren't currently mounted (currently Linux only). Defaults to false.
    pub(crate) include_unmounted: Option<bool>,

    /// A list of disk widget columns.
    // TODO: make this more composable(?) in the future, we might need to
    // rethink how it's done for custom widgets.
    #[serde(default)]
    pub(crate) columns: Option<Vec<DiskWidgetColumn>>,

    /// The default sort column.
    #[serde(default)]
    pub(crate) default_sort: Option<DiskWidgetColumn>,
}

#[cfg(test)]
mod test {
    use super::DiskConfig;

    #[test]
    fn none_column_setting() {
        let config = "";
        let generated: DiskConfig = toml_edit::de::from_str(config).unwrap();
        assert!(generated.columns.is_none());
    }

    #[test]
    fn empty_column_setting() {
        let config = r#"columns = []"#;
        let generated: DiskConfig = toml_edit::de::from_str(config).unwrap();
        assert!(generated.columns.unwrap().is_empty());
    }

    #[test]
    fn valid_disk_column_settings() {
        let config = r#"columns = ["disk", "mount", "used", "free", "total", "used%", "free%", "r/s", "w/s"]"#;
        toml_edit::de::from_str::<DiskConfig>(config).expect("Should succeed!");
    }

    #[test]
    fn bad_disk_column_settings() {
        let config = r#"columns = ["diskk"]"#;
        toml_edit::de::from_str::<DiskConfig>(config).expect_err("Should error out!");
    }

    /// Test that disk enum variants that are advertised in the schema are valid.
    #[cfg(feature = "generate_schema")]
    #[test]
    fn ensure_disk_column_schema_is_accepted() {
        use strum::VariantArray;

        use crate::options::{Config, DiskWidgetColumn};

        for column in DiskWidgetColumn::VARIANTS {
            for &name in column.get_schema_names() {
                let config = format!("[disk]\ncolumns = [\"{name}\"]\n");
                toml_edit::de::from_str::<Config>(&config).unwrap_or_else(|e| {
                    panic!("schema name {name:?} was rejected:\n{e}\nconfig was:\n{config}")
                });
            }
        }
    }
}
