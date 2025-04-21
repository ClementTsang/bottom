use serde::{Deserialize, Serialize};

/// Workaround as per <https://github.com/serde-rs/serde/issues/1030>.
fn default_as_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct IgnoreList {
    #[serde(default = "default_as_true")]
    // TODO: Deprecate and/or rename, current name sounds awful.
    // Maybe to something like "deny_entries"?  Currently it defaults to a denylist anyways, so
    // maybe "allow_entries"?
    pub is_list_ignored: bool,
    pub list: Vec<String>,
    #[serde(default)]
    pub regex: bool,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub whole_word: bool,
}
