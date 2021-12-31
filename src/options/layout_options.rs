use serde::{Deserialize, Serialize};

/// Represents a row.  This has a length of some sort (optional) and a vector
/// of children.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename = "row")]
pub struct LayoutRow {
    pub child: Option<Vec<LayoutRowChild>>,
    pub ratio: Option<u16>,
}

/// Represents a child of a Row - either a Col (column) or a FinalWidget.
///
/// A Col can also have an optional length and children.  We only allow columns
/// to have FinalWidgets as children, lest we get some amount of mutual
/// recursion between Row and Col.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(untagged)]
pub enum LayoutRowChild {
    Widget(FinalWidget),
    /// The first one in the list is the "default" selected widget.
    Carousel {
        carousel_children: Vec<String>,
        default: Option<bool>,
    },
    LayoutCol {
        ratio: Option<u16>,
        child: Vec<FinalWidget>,
    },
}

/// Represents a widget.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FinalWidget {
    #[serde(flatten)]
    pub rule: Option<WidgetLayoutRule>,
    #[serde(rename = "type")]
    pub widget_type: String,
    pub default: Option<bool>,
}

/// A "rule" denoting how a widget is to be laid out.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum WidgetLayoutRule {
    /// Expand to whatever space is left. The `ratio` field determines
    /// how much space to allocate if there are other [`WidgetLayoutRule::Expand`]
    /// items.
    Expand { ratio: u16 },

    /// Take up an exact amount of space, if possible.
    Length {
        width: Option<u16>,
        height: Option<u16>,
    },
}

impl Default for WidgetLayoutRule {
    fn default() -> Self {
        WidgetLayoutRule::Expand { ratio: 1 }
    }
}
