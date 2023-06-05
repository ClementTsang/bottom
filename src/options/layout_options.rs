use serde::{Deserialize, Serialize};

/// Represents a row. This has a length of some sort (optional) and a vector
/// of children.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename = "row")]
pub struct Row {
    pub ratio: Option<u32>,
    pub child: Option<Vec<RowChildren>>,
}

/// Represents a child of a Row - either a Col (column) or a FinalWidget.
///
/// A Col can also have an optional length and children. We only allow columns
/// to have FinalWidgets as children, lest we get some amount of mutual
/// recursion between Row and Col.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(untagged)]
pub enum RowChildren {
    Widget(FinalWidget),
    Col {
        ratio: Option<u32>,
        child: Vec<FinalWidget>,
    },
}

/// Represents a widget.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FinalWidget {
    pub ratio: Option<u32>,
    #[serde(rename = "type")]
    pub widget_type: String,
    pub default: Option<bool>,
}
