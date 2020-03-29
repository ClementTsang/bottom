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
        &self, iter_id: &mut u64, total_height_ratio: &mut u32, left_legend: bool,
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
                        let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;
                        children.push(match widget_type {
                            BottomWidgetType::Cpu => {
                                let iter_old_id = *iter_id;
                                *iter_id += 1;
                                BottomCol {
                                    total_col_row_ratio: 1,
                                    col_width_ratio: width_ratio,
                                    children: if left_legend {
                                        vec![BottomColRow {
                                            col_row_height_ratio: 1,
                                            total_widget_ratio: 20,
                                            children: vec![
                                                BottomWidget {
                                                    width_ratio: 3,
                                                    widget_type: BottomWidgetType::CpuLegend,
                                                    widget_id: *iter_id,
                                                    canvas_handle_height: true,
                                                    ..BottomWidget::default()
                                                },
                                                BottomWidget {
                                                    width_ratio: 17,
                                                    widget_type: BottomWidgetType::Cpu,
                                                    widget_id: iter_old_id,
                                                    flex_grow: true,
                                                    ..BottomWidget::default()
                                                },
                                            ],
                                            ..BottomColRow::default()
                                        }]
                                    } else {
                                        vec![BottomColRow {
                                            col_row_height_ratio: 1,
                                            total_widget_ratio: 20,
                                            children: vec![
                                                BottomWidget {
                                                    width_ratio: 17,
                                                    widget_type: BottomWidgetType::Cpu,
                                                    widget_id: iter_old_id,
                                                    flex_grow: true,
                                                    ..BottomWidget::default()
                                                },
                                                BottomWidget {
                                                    width_ratio: 3,
                                                    widget_type: BottomWidgetType::CpuLegend,
                                                    widget_id: *iter_id,
                                                    canvas_handle_height: true,
                                                    ..BottomWidget::default()
                                                },
                                            ],
                                            ..BottomColRow::default()
                                        }]
                                    },
                                }
                            }
                            BottomWidgetType::Proc => {
                                let iter_old_id = *iter_id;
                                *iter_id += 1;
                                BottomCol {
                                    total_col_row_ratio: 2,
                                    col_width_ratio: width_ratio,
                                    children: vec![
                                        BottomColRow {
                                            col_row_height_ratio: 1,
                                            total_widget_ratio: 1,
                                            children: vec![BottomWidget {
                                                width_ratio: 1,
                                                widget_type: BottomWidgetType::Proc,
                                                widget_id: iter_old_id,
                                                ..BottomWidget::default()
                                            }],
                                            flex_grow: true,
                                            ..BottomColRow::default()
                                        },
                                        BottomColRow {
                                            col_row_height_ratio: 1,
                                            total_widget_ratio: 1,
                                            children: vec![BottomWidget {
                                                width_ratio: 1,
                                                widget_type: BottomWidgetType::ProcSearch,
                                                widget_id: *iter_id,
                                                ..BottomWidget::default()
                                            }],
                                            canvas_handle_height: true,
                                            ..BottomColRow::default()
                                        },
                                    ],
                                }
                            }
                            _ => BottomCol {
                                total_col_row_ratio: 1,
                                col_width_ratio: width_ratio,
                                children: vec![BottomColRow {
                                    col_row_height_ratio: 1,
                                    total_widget_ratio: 1,
                                    children: vec![BottomWidget {
                                        width_ratio: 1,
                                        widget_type,
                                        widget_id: *iter_id,
                                        ..BottomWidget::default()
                                    }],
                                    ..BottomColRow::default()
                                }],
                            },
                        });
                    }
                    RowChildren::Col { ratio, child } => {
                        let col_width_ratio = ratio.unwrap_or(1);
                        total_col_ratio += col_width_ratio;
                        let mut total_col_row_ratio = 0;
                        let mut contains_proc = false;

                        let mut col_row_children = Vec::new();

                        for column_child in child {
                            let widget_type =
                                column_child.widget_type.parse::<BottomWidgetType>()?;
                            *iter_id += 1;
                            let col_row_height_ratio = column_child.ratio.unwrap_or(1);
                            total_col_row_ratio += col_row_height_ratio;

                            match widget_type {
                                BottomWidgetType::Cpu => {
                                    let iter_old_id = *iter_id;
                                    *iter_id += 1;
                                    if left_legend {
                                        col_row_children.push(BottomColRow {
                                            col_row_height_ratio,
                                            total_widget_ratio: 20,
                                            children: vec![
                                                BottomWidget {
                                                    width_ratio: 3,
                                                    widget_type: BottomWidgetType::CpuLegend,
                                                    widget_id: *iter_id,
                                                    canvas_handle_height: true,
                                                    ..BottomWidget::default()
                                                },
                                                BottomWidget {
                                                    width_ratio: 17,
                                                    widget_type: BottomWidgetType::Cpu,
                                                    widget_id: iter_old_id,
                                                    flex_grow: true,
                                                    ..BottomWidget::default()
                                                },
                                            ],
                                            ..BottomColRow::default()
                                        });
                                    } else {
                                        col_row_children.push(BottomColRow {
                                            col_row_height_ratio,
                                            total_widget_ratio: 20,
                                            children: vec![
                                                BottomWidget {
                                                    width_ratio: 17,
                                                    widget_type: BottomWidgetType::Cpu,
                                                    widget_id: iter_old_id,
                                                    flex_grow: true,
                                                    ..BottomWidget::default()
                                                },
                                                BottomWidget {
                                                    width_ratio: 3,
                                                    widget_type: BottomWidgetType::CpuLegend,
                                                    widget_id: *iter_id,
                                                    canvas_handle_height: true,
                                                    ..BottomWidget::default()
                                                },
                                            ],
                                            ..BottomColRow::default()
                                        });
                                    }
                                }
                                BottomWidgetType::Proc => {
                                    contains_proc = true;
                                    let iter_old_id = *iter_id;
                                    *iter_id += 1;
                                    col_row_children.push(BottomColRow {
                                        col_row_height_ratio,
                                        total_widget_ratio: 1,
                                        children: vec![BottomWidget {
                                            width_ratio: 1,
                                            widget_type: BottomWidgetType::Proc,
                                            widget_id: iter_old_id,
                                            ..BottomWidget::default()
                                        }],
                                        flex_grow: true,
                                        ..BottomColRow::default()
                                    });
                                    col_row_children.push(BottomColRow {
                                        col_row_height_ratio,
                                        total_widget_ratio: 1,
                                        children: vec![BottomWidget {
                                            width_ratio: 1,
                                            widget_type: BottomWidgetType::ProcSearch,
                                            widget_id: *iter_id,
                                            ..BottomWidget::default()
                                        }],
                                        canvas_handle_height: true,
                                        ..BottomColRow::default()
                                    });
                                }
                                _ => col_row_children.push(BottomColRow {
                                    col_row_height_ratio,
                                    total_widget_ratio: 1,
                                    children: vec![BottomWidget {
                                        width_ratio: 1,
                                        widget_type,
                                        widget_id: *iter_id,
                                        ..BottomWidget::default()
                                    }],
                                    ..BottomColRow::default()
                                }),
                            }
                        }

                        if contains_proc {
                            // Must adjust ratios to work with proc
                            total_col_row_ratio *= 2;
                            for child in &mut col_row_children {
                                // Multiply all non-proc or proc-search ratios by 2
                                if !child.children.is_empty() {
                                    match child.children[0].widget_type {
                                        BottomWidgetType::Proc | BottomWidgetType::ProcSearch => {}
                                        _ => child.col_row_height_ratio *= 2,
                                    }
                                }
                            }
                        }

                        children.push(BottomCol {
                            total_col_row_ratio,
                            col_width_ratio,
                            children: col_row_children,
                        });
                    }
                }
            }
        }

        Ok(BottomRow {
            total_col_ratio,
            row_height_ratio: row_ratio,
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
