use crate::app::layout_manager::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Represents a row.  This has a length of some sort (optional) and a vector
/// of children.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename = "row")]
pub struct Row {
    pub ratio: Option<u32>,
    pub child: Option<Vec<RowChildren>>,
}

impl Row {
    pub fn convert_row_to_bottom_row(
        &self, iter_id: &mut u64, total_height_ratio: &mut u32, default_widget_id: &mut u64,
        default_widget_type: &Option<BottomWidgetType>, default_widget_count: &mut u64,
        left_legend: bool,
    ) -> Result<BottomRow> {
        // TODO: In the future we want to also add percentages.
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

                        if let Some(default_widget_type_val) = default_widget_type {
                            if *default_widget_type_val == widget_type && *default_widget_count > 0
                            {
                                *default_widget_count -= 1;
                                if *default_widget_count == 0 {
                                    *default_widget_id = *iter_id;
                                }
                            }
                        } else {
                            // Check default flag
                            if let Some(default_widget_flag) = widget.default {
                                if default_widget_flag {
                                    *default_widget_id = *iter_id;
                                }
                            }
                        }

                        children.push(match widget_type {
                            BottomWidgetType::Cpu => {
                                let cpu_id = *iter_id;
                                *iter_id += 1;
                                BottomCol::builder()
                                    .col_width_ratio(width_ratio)
                                    .children(if left_legend {
                                        vec![BottomColRow::builder()
                                            .total_widget_ratio(20)
                                            .children(vec![
                                                BottomWidget::builder()
                                                    .width_ratio(3)
                                                    .widget_type(BottomWidgetType::CpuLegend)
                                                    .widget_id(*iter_id)
                                                    .canvas_handle_width(true)
                                                    .parent_reflector(Some((
                                                        WidgetDirection::Right,
                                                        1,
                                                    )))
                                                    .build(),
                                                BottomWidget::builder()
                                                    .width_ratio(17)
                                                    .widget_type(BottomWidgetType::Cpu)
                                                    .widget_id(cpu_id)
                                                    .flex_grow(true)
                                                    .build(),
                                            ])
                                            .build()]
                                    } else {
                                        vec![BottomColRow::builder()
                                            .total_widget_ratio(20)
                                            .children(vec![
                                                BottomWidget::builder()
                                                    .width_ratio(17)
                                                    .widget_type(BottomWidgetType::Cpu)
                                                    .widget_id(cpu_id)
                                                    .flex_grow(true)
                                                    .build(),
                                                BottomWidget::builder()
                                                    .width_ratio(3)
                                                    .widget_type(BottomWidgetType::CpuLegend)
                                                    .widget_id(*iter_id)
                                                    .canvas_handle_width(true)
                                                    .parent_reflector(Some((
                                                        WidgetDirection::Left,
                                                        1,
                                                    )))
                                                    .build(),
                                            ])
                                            .build()]
                                    })
                                    .build()
                            }
                            BottomWidgetType::Proc => {
                                let proc_id = *iter_id;
                                let proc_search_id = *iter_id + 1;
                                *iter_id += 2;
                                BottomCol::builder()
                                    .total_col_row_ratio(2)
                                    .col_width_ratio(width_ratio)
                                    .children(vec![
                                        BottomColRow::builder()
                                            .children(vec![
                                                BottomWidget::builder()
                                                    .widget_type(BottomWidgetType::ProcSort)
                                                    .widget_id(*iter_id)
                                                    .canvas_handle_width(true)
                                                    .parent_reflector(Some((
                                                        WidgetDirection::Right,
                                                        2,
                                                    )))
                                                    .width_ratio(1)
                                                    .build(),
                                                BottomWidget::builder()
                                                    .widget_type(BottomWidgetType::Proc)
                                                    .widget_id(proc_id)
                                                    .width_ratio(2)
                                                    .build(),
                                            ])
                                            .total_widget_ratio(3)
                                            .flex_grow(true)
                                            .build(),
                                        BottomColRow::builder()
                                            .children(vec![BottomWidget::builder()
                                                .widget_type(BottomWidgetType::ProcSearch)
                                                .widget_id(proc_search_id)
                                                .parent_reflector(Some((WidgetDirection::Up, 1)))
                                                .build()])
                                            .canvas_handle_height(true)
                                            .build(),
                                    ])
                                    .build()
                            }
                            _ => BottomCol::builder()
                                .col_width_ratio(width_ratio)
                                .children(vec![BottomColRow::builder()
                                    .children(vec![BottomWidget::builder()
                                        .widget_type(widget_type)
                                        .widget_id(*iter_id)
                                        .build()])
                                    .build()])
                                .build(),
                        });
                    }
                    RowChildren::Col { ratio, child } => {
                        let col_width_ratio = ratio.unwrap_or(1);
                        total_col_ratio += col_width_ratio;
                        let mut total_col_row_ratio = 0;
                        let mut contains_proc = false;

                        let mut col_row_children: Vec<BottomColRow> = Vec::new();

                        for widget in child {
                            let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;
                            *iter_id += 1;
                            let col_row_height_ratio = widget.ratio.unwrap_or(1);
                            total_col_row_ratio += col_row_height_ratio;

                            if let Some(default_widget_type_val) = default_widget_type {
                                if *default_widget_type_val == widget_type
                                    && *default_widget_count > 0
                                {
                                    *default_widget_count -= 1;
                                    if *default_widget_count == 0 {
                                        *default_widget_id = *iter_id;
                                    }
                                }
                            } else {
                                // Check default flag
                                if let Some(default_widget_flag) = widget.default {
                                    if default_widget_flag {
                                        *default_widget_id = *iter_id;
                                    }
                                }
                            }

                            match widget_type {
                                BottomWidgetType::Cpu => {
                                    let cpu_id = *iter_id;
                                    *iter_id += 1;
                                    if left_legend {
                                        col_row_children.push(
                                            BottomColRow::builder()
                                                .col_row_height_ratio(col_row_height_ratio)
                                                .total_widget_ratio(20)
                                                .children(vec![
                                                    BottomWidget::builder()
                                                        .width_ratio(3)
                                                        .widget_type(BottomWidgetType::CpuLegend)
                                                        .widget_id(*iter_id)
                                                        .canvas_handle_width(true)
                                                        .parent_reflector(Some((
                                                            WidgetDirection::Right,
                                                            1,
                                                        )))
                                                        .build(),
                                                    BottomWidget::builder()
                                                        .width_ratio(17)
                                                        .widget_type(BottomWidgetType::Cpu)
                                                        .widget_id(cpu_id)
                                                        .flex_grow(true)
                                                        .build(),
                                                ])
                                                .build(),
                                        );
                                    } else {
                                        col_row_children.push(
                                            BottomColRow::builder()
                                                .col_row_height_ratio(col_row_height_ratio)
                                                .total_widget_ratio(20)
                                                .children(vec![
                                                    BottomWidget::builder()
                                                        .width_ratio(17)
                                                        .widget_type(BottomWidgetType::Cpu)
                                                        .widget_id(cpu_id)
                                                        .flex_grow(true)
                                                        .build(),
                                                    BottomWidget::builder()
                                                        .width_ratio(3)
                                                        .widget_type(BottomWidgetType::CpuLegend)
                                                        .widget_id(*iter_id)
                                                        .canvas_handle_width(true)
                                                        .parent_reflector(Some((
                                                            WidgetDirection::Left,
                                                            1,
                                                        )))
                                                        .build(),
                                                ])
                                                .build(),
                                        );
                                    }
                                }
                                BottomWidgetType::Proc => {
                                    contains_proc = true;
                                    let proc_id = *iter_id;
                                    let proc_search_id = *iter_id + 1;
                                    *iter_id += 2;
                                    col_row_children.push(
                                        BottomColRow::builder()
                                            .children(vec![
                                                BottomWidget::builder()
                                                    .widget_type(BottomWidgetType::ProcSort)
                                                    .widget_id(*iter_id)
                                                    .canvas_handle_width(true)
                                                    .parent_reflector(Some((
                                                        WidgetDirection::Right,
                                                        2,
                                                    )))
                                                    .width_ratio(1)
                                                    .build(),
                                                BottomWidget::builder()
                                                    .widget_type(BottomWidgetType::Proc)
                                                    .widget_id(proc_id)
                                                    .width_ratio(2)
                                                    .build(),
                                            ])
                                            .col_row_height_ratio(col_row_height_ratio)
                                            .total_widget_ratio(3)
                                            .build(),
                                    );
                                    col_row_children.push(
                                        BottomColRow::builder()
                                            .col_row_height_ratio(col_row_height_ratio)
                                            .children(vec![BottomWidget::builder()
                                                .widget_type(BottomWidgetType::ProcSearch)
                                                .widget_id(proc_search_id)
                                                .parent_reflector(Some((WidgetDirection::Up, 1)))
                                                .build()])
                                            .canvas_handle_height(true)
                                            .build(),
                                    );
                                }
                                _ => col_row_children.push(
                                    BottomColRow::builder()
                                        .col_row_height_ratio(col_row_height_ratio)
                                        .children(vec![BottomWidget::builder()
                                            .widget_type(widget_type)
                                            .widget_id(*iter_id)
                                            .build()])
                                        .build(),
                                ),
                            }
                        }

                        if contains_proc {
                            // Must adjust ratios to work with proc
                            total_col_row_ratio *= 2;
                            for child in &mut col_row_children {
                                // Multiply all non-proc or proc-search ratios by 2
                                if !child.children.is_empty() {
                                    match child.children[0].widget_type {
                                        BottomWidgetType::ProcSearch => {}
                                        _ => child.col_row_height_ratio *= 2,
                                    }
                                }
                            }
                        }

                        children.push(
                            BottomCol::builder()
                                .total_col_row_ratio(total_col_row_ratio)
                                .col_width_ratio(col_width_ratio)
                                .children(col_row_children)
                                .build(),
                        );
                    }
                }
            }
        }

        Ok(BottomRow::builder()
            .total_col_ratio(total_col_ratio)
            .row_height_ratio(row_ratio)
            .children(children)
            .build())
    }
}

/// Represents a child of a Row - either a Col (column) or a FinalWidget.
///
/// A Col can also have an optional length and children.  We only allow columns
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
