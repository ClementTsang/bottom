use crate::app::layout_manager::*;
use crate::error::Result;
use serde::Deserialize;

/// Represents a row.  This has a length of some sort (optional) and a vector
/// of children.
#[derive(Deserialize, Debug)]
#[serde(rename = "row")]
pub struct Row {
    pub ratio: Option<u32>,
    pub child: Option<Vec<RowChildren>>,
}

impl Row {
    pub fn convert_row_to_bottom_row(
        &self, iter_id: &mut u64, total_height_ratio: &mut u32,
    ) -> Result<BottomRow> {
        // In the future we want to also add percentages.
        // But for MVP, we aren't going to bother.
        let row_ratio = self.ratio.unwrap_or(1);
        let mut children = Vec::new();

        *total_height_ratio += row_ratio;

        let mut total_col_ratio = 0;
        if let Some(row_children) = &self.child {
            for row_child in row_children {
                match row_child {
                    RowChildren::Widget(widget) => {
                        *iter_id += 1;
                        let width_ratio = widget.ratio.unwrap_or(1);
                        total_col_ratio += width_ratio;
                        children.push(BottomCol {
                            total_widget_ratio: 1,
                            width_ratio,
                            children: vec![BottomWidget {
                                height_ratio: 1,
                                widget_type: widget.widget_type.parse::<BottomWidgetType>()?,
                                widget_id: *iter_id,
                                ..BottomWidget::default()
                            }],
                        });
                    }
                    RowChildren::Col { ratio, child } => {
                        let width_ratio = ratio.unwrap_or(1);
                        total_col_ratio += width_ratio;
                        let mut total_widget_ratio = 0;

                        let widget_children = child
                            .iter()
                            .map(|column_child| {
                                let parsed_widget_type =
                                    column_child.widget_type.parse::<BottomWidgetType>();
                                *iter_id += 1;
                                let height_ratio = column_child.ratio.unwrap_or(1);
                                total_widget_ratio += height_ratio;
                                match parsed_widget_type {
                                    Ok(widget_type) => Ok(BottomWidget {
                                        height_ratio,
                                        widget_type,
                                        widget_id: *iter_id,
                                        ..BottomWidget::default()
                                    }),
                                    Err(err) => Err(err),
                                }
                            })
                            .collect::<Result<Vec<BottomWidget>>>()?;

                        children.push(BottomCol {
                            total_widget_ratio,
                            width_ratio,
                            children: widget_children,
                        });
                    }
                }
            }
        }

        Ok(BottomRow {
            total_col_ratio,
            row_ratio,
            children,
        })
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
        ratio: Option<u32>,
        child: Vec<FinalWidget>,
    },
}

/// Represents a widget.
#[derive(Deserialize, Debug)]
pub struct FinalWidget {
    pub ratio: Option<u32>,
    #[serde(rename = "type")]
    pub widget_type: String,
}
