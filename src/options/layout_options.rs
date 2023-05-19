use serde::{Deserialize, Serialize};

use crate::app::layout_manager::*;
use crate::error::Result;

/// Represents a row.  This has a length of some sort (optional) and a vector
/// of children.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename = "row")]
pub struct Row {
    pub ratio: Option<u32>,
    pub child: Option<Vec<RowChildren>>,
}

fn new_cpu(left_legend: bool, iter_id: &mut u64) -> BottomColRow {
    let cpu_id = *iter_id;
    *iter_id += 1;
    let legend_id = *iter_id;

    if left_legend {
        BottomColRow::new(vec![
            BottomWidget::new(BottomWidgetType::CpuLegend, legend_id)
                .width_ratio(3)
                .canvas_handle_width(true)
                .parent_reflector(Some((WidgetDirection::Right, 1))),
            BottomWidget::new(BottomWidgetType::Cpu, cpu_id)
                .width_ratio(17)
                .flex_grow(true),
        ])
    } else {
        BottomColRow::new(vec![
            BottomWidget::new(BottomWidgetType::Cpu, cpu_id)
                .width_ratio(17)
                .flex_grow(true),
            BottomWidget::new(BottomWidgetType::CpuLegend, legend_id)
                .width_ratio(3)
                .canvas_handle_width(true)
                .parent_reflector(Some((WidgetDirection::Left, 1))),
        ])
    }
    .total_widget_ratio(20)
}

fn new_proc_sort(sort_id: u64) -> BottomWidget {
    BottomWidget::new(BottomWidgetType::ProcSort, sort_id)
        .canvas_handle_width(true)
        .parent_reflector(Some((WidgetDirection::Right, 2)))
        .width_ratio(1)
}

fn new_proc(proc_id: u64) -> BottomWidget {
    BottomWidget::new(BottomWidgetType::Proc, proc_id).width_ratio(2)
}

fn new_proc_search(search_id: u64) -> BottomWidget {
    BottomWidget::new(BottomWidgetType::ProcSearch, search_id)
        .parent_reflector(Some((WidgetDirection::Up, 1)))
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
                                BottomCol::new(vec![new_cpu(left_legend, iter_id)])
                                    .col_width_ratio(width_ratio)
                            }
                            BottomWidgetType::Proc => {
                                let proc_id = *iter_id;
                                let proc_search_id = *iter_id + 1;
                                *iter_id += 2;
                                BottomCol::new(vec![
                                    BottomColRow::new(vec![
                                        new_proc_sort(*iter_id),
                                        new_proc(proc_id),
                                    ])
                                    .total_widget_ratio(3)
                                    .flex_grow(true),
                                    BottomColRow::new(vec![new_proc_search(proc_search_id)])
                                        .canvas_handle_height(true),
                                ])
                                .total_col_row_ratio(2)
                                .col_width_ratio(width_ratio)
                            }
                            _ => BottomCol::new(vec![BottomColRow::new(vec![BottomWidget::new(
                                widget_type,
                                *iter_id,
                            )])])
                            .col_width_ratio(width_ratio),
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
                                    col_row_children.push(
                                        new_cpu(left_legend, iter_id)
                                            .col_row_height_ratio(col_row_height_ratio),
                                    );
                                }
                                BottomWidgetType::Proc => {
                                    contains_proc = true;
                                    let proc_id = *iter_id;
                                    let proc_search_id = *iter_id + 1;
                                    *iter_id += 2;
                                    col_row_children.push(
                                        BottomColRow::new(vec![
                                            new_proc_sort(*iter_id),
                                            new_proc(proc_id),
                                        ])
                                        .col_row_height_ratio(col_row_height_ratio)
                                        .total_widget_ratio(3),
                                    );
                                    col_row_children.push(
                                        BottomColRow::new(vec![new_proc_search(proc_search_id)])
                                            .canvas_handle_height(true)
                                            .col_row_height_ratio(col_row_height_ratio),
                                    );
                                }
                                _ => col_row_children.push(
                                    BottomColRow::new(vec![BottomWidget::new(
                                        widget_type,
                                        *iter_id,
                                    )])
                                    .col_row_height_ratio(col_row_height_ratio),
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
                            BottomCol::new(col_row_children)
                                .total_col_row_ratio(total_col_row_ratio)
                                .col_width_ratio(col_width_ratio),
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
