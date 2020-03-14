use crate::app::layout_manager::*;
use crate::error::Result;
use serde::Deserialize;

/// Represents a row.  This has a length of some sort (optional) and a vector
/// of children.
#[derive(Deserialize, Debug)]
#[serde(rename = "row")]
pub struct Row {
    pub ratio: Option<u64>,
    pub child: Option<Vec<RowChildren>>,
}

impl Row {
    pub fn convert_row_to_bottom_row(&self) -> Result<BottomRow> {
        // In the future we want to also add percentages.
        // But for MVP, we aren't going to bother.
        let ratio = self.ratio.unwrap_or(1);
        let mut children = Vec::new();

        if let Some(row_children) = &self.child {
            for row_child in row_children {
                match row_child {
                    RowChildren::Widget(widget) => {
                        children.push(BottomCol {
                            ratio: widget.ratio.unwrap_or(1),
                            children: vec![BottomWidget {
                                ratio: 1,
                                widget_type: widget.widget_type.parse::<BottomWidgetType>()?,
                            }],
                        });
                    }
                    RowChildren::Col { ratio, child } => {
                        children.push(BottomCol {
                            ratio: ratio.unwrap_or(1),
                            children: child
                                .iter()
                                .map(|column_child| {
                                    let parsed_widget_type =
                                        column_child.widget_type.parse::<BottomWidgetType>();
                                    match parsed_widget_type {
                                        Ok(widget_type) => Ok(BottomWidget {
                                            ratio: column_child.ratio.unwrap_or(1),
                                            widget_type,
                                        }),
                                        Err(err) => Err(err),
                                    }
                                })
                                .collect::<Result<Vec<BottomWidget>>>()?,
                        });
                    }
                }
            }
        }

        Ok(BottomRow { ratio, children })
    }
}

/// Represents a child of a Row - either a Col (column) or a FinalWidget.
///
/// A Col can also have an optional length and children.  We only allow columns
/// to have FinalWidgets as children, lest we get some amount of mutual
/// recursion between Row and Col.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RowChildren {
    Widget(FinalWidget),
    Col {
        ratio: Option<u64>,
        child: Vec<FinalWidget>,
    },
}

/// Represents a widget.
#[derive(Deserialize, Debug)]
pub struct FinalWidget {
    pub ratio: Option<u64>,
    #[serde(rename = "type")]
    pub widget_type: String,
}
