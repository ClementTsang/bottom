use serde::Deserialize;

use super::IgnoreList;
use crate::options::DiskColumn;

/// Disk configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskConfig {
    /// A filter over the disk names.
    pub(crate) name_filter: Option<IgnoreList>,

    /// A filter over the mount names.
    pub(crate) mount_filter: Option<IgnoreList>,

    /// A list of disk widget columns.
    #[serde(default)]
    pub(crate) columns: Option<Vec<DiskColumn>>, // TODO: make this more composable(?) in the future, we might need to rethink how it's done for custom widgets
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
}
