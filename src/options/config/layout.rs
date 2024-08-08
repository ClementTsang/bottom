use serde::{Deserialize, Serialize};

use crate::{app::layout_manager::*, options::OptionResult};

/// Represents a row. This has a length of some sort (optional) and a vector
/// of children.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
#[serde(rename = "row")]
pub struct Row {
    pub ratio: Option<u32>,
    pub child: Option<Vec<RowChildren>>,
}

fn new_cpu(cpu_left_legend: bool, iter_id: &mut u64) -> BottomColRow {
    let cpu_id = *iter_id;
    *iter_id += 1;
    let legend_id = *iter_id;

    if cpu_left_legend {
        BottomColRow::new(vec![
            BottomWidget::new(BottomWidgetType::CpuLegend, legend_id)
                .canvas_with_ratio(3)
                .parent_reflector(Some((WidgetDirection::Right, 1))),
            BottomWidget::new(BottomWidgetType::Cpu, cpu_id).grow(Some(17)),
        ])
    } else {
        BottomColRow::new(vec![
            BottomWidget::new(BottomWidgetType::Cpu, cpu_id).grow(Some(17)),
            BottomWidget::new(BottomWidgetType::CpuLegend, legend_id)
                .canvas_with_ratio(3)
                .parent_reflector(Some((WidgetDirection::Left, 1))),
        ])
    }
    .total_widget_ratio(20)
}

fn new_proc_sort(sort_id: u64) -> BottomWidget {
    BottomWidget::new(BottomWidgetType::ProcSort, sort_id)
        .canvas_handled()
        .parent_reflector(Some((WidgetDirection::Right, 2)))
}

fn new_proc(proc_id: u64) -> BottomWidget {
    BottomWidget::new(BottomWidgetType::Proc, proc_id).ratio(2)
}

fn new_proc_search(search_id: u64) -> BottomWidget {
    BottomWidget::new(BottomWidgetType::ProcSearch, search_id)
        .parent_reflector(Some((WidgetDirection::Up, 1)))
}

impl Row {
    pub fn convert_row_to_bottom_row(
        &self, iter_id: &mut u64, total_height_ratio: &mut u32, default_widget_id: &mut u64,
        default_widget_type: &Option<BottomWidgetType>, default_widget_count: &mut u64,
        cpu_left_legend: bool,
    ) -> OptionResult<BottomRow> {
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
                                BottomCol::new(vec![new_cpu(cpu_left_legend, iter_id)])
                                    .ratio(width_ratio)
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
                                    .grow(None)
                                    .total_widget_ratio(3),
                                    BottomColRow::new(vec![new_proc_search(proc_search_id)])
                                        .canvas_handled(),
                                ])
                                .total_col_row_ratio(2)
                                .ratio(width_ratio)
                            }
                            _ => BottomCol::new(vec![BottomColRow::new(vec![BottomWidget::new(
                                widget_type,
                                *iter_id,
                            )])])
                            .ratio(width_ratio),
                        });
                    }
                    RowChildren::Col { ratio, child } => {
                        let col_width_ratio = ratio.unwrap_or(1);
                        total_col_ratio += col_width_ratio;
                        let mut total_col_row_ratio = 0;

                        let mut col_row_children: Vec<BottomColRow> = Vec::new();

                        for widget in child {
                            let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;
                            *iter_id += 1;

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
                                    let col_row_height_ratio = widget.ratio.unwrap_or(1);
                                    total_col_row_ratio += col_row_height_ratio;

                                    col_row_children.push(
                                        new_cpu(cpu_left_legend, iter_id)
                                            .ratio(col_row_height_ratio),
                                    );
                                }
                                BottomWidgetType::Proc => {
                                    let col_row_height_ratio = widget.ratio.unwrap_or(1) + 1;
                                    total_col_row_ratio += col_row_height_ratio;

                                    let proc_id = *iter_id;
                                    let proc_search_id = *iter_id + 1;
                                    *iter_id += 2;
                                    col_row_children.push(
                                        BottomColRow::new(vec![
                                            new_proc_sort(*iter_id),
                                            new_proc(proc_id),
                                        ])
                                        .ratio(col_row_height_ratio)
                                        .total_widget_ratio(3),
                                    );
                                    col_row_children.push(
                                        BottomColRow::new(vec![new_proc_search(proc_search_id)])
                                            .canvas_handled(),
                                    );
                                }
                                _ => {
                                    let col_row_height_ratio = widget.ratio.unwrap_or(1);
                                    total_col_row_ratio += col_row_height_ratio;

                                    col_row_children.push(
                                        BottomColRow::new(vec![BottomWidget::new(
                                            widget_type,
                                            *iter_id,
                                        )])
                                        .ratio(col_row_height_ratio),
                                    )
                                }
                            }
                        }

                        children.push(
                            BottomCol::new(col_row_children)
                                .total_col_row_ratio(total_col_row_ratio)
                                .ratio(col_width_ratio),
                        );
                    }
                }
            }
        }

        Ok(BottomRow::new(children)
            .total_col_ratio(total_col_ratio)
            .ratio(row_ratio))
    }
}

/// Represents a child of a Row - either a Col (column) or a FinalWidget.
///
/// A Col can also have an optional length and children.  We only allow columns
/// to have FinalWidgets as children, lest we get some amount of mutual
/// recursion between Row and Col.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub enum RowChildren {
    Widget(FinalWidget),
    Col {
        ratio: Option<u32>,
        child: Vec<FinalWidget>,
    },
}

/// Represents a widget.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct FinalWidget {
    pub ratio: Option<u32>,
    #[serde(rename = "type")]
    pub widget_type: String,
    pub default: Option<bool>,
}

#[cfg(test)]
mod test {
    use toml_edit::de::from_str;

    use super::*;
    use crate::{
        constants::{DEFAULT_LAYOUT, DEFAULT_WIDGET_ID},
        options::Config,
    };

    const PROC_LAYOUT: &str = r#"
    [[row]]
        [[row.child]]
            type="proc"
    [[row]]
        [[row.child]]
            type="proc"
        [[row.child]]
            type="proc"
    [[row]]
        [[row.child]]
            type="proc"
        [[row.child]]
            type="proc"
    "#;

    fn test_create_layout(
        rows: &[Row], default_widget_id: u64, default_widget_type: Option<BottomWidgetType>,
        default_widget_count: u64, left_legend: bool,
    ) -> BottomLayout {
        let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
        let mut total_height_ratio = 0;
        let mut default_widget_count = default_widget_count;
        let mut default_widget_id = default_widget_id;

        let mut ret_bottom_layout = BottomLayout {
            rows: rows
                .iter()
                .map(|row| {
                    row.convert_row_to_bottom_row(
                        &mut iter_id,
                        &mut total_height_ratio,
                        &mut default_widget_id,
                        &default_widget_type,
                        &mut default_widget_count,
                        left_legend,
                    )
                })
                .collect::<OptionResult<Vec<_>>>()
                .unwrap(),
            total_row_height_ratio: total_height_ratio,
        };
        ret_bottom_layout.get_movement_mappings();

        ret_bottom_layout
    }

    #[test]
    /// Tests the default setup.
    fn test_default_movement() {
        let rows = from_str::<Config>(DEFAULT_LAYOUT).unwrap().row.unwrap();
        let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, false);

        // Simple tests for the top CPU widget
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
            Some(3)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
            Some(2)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
            None
        );

        // Test CPU legend
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
            Some(4)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
            Some(1)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
            None
        );

        // Test memory->temp, temp->disk, disk->memory mappings
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[0].right_neighbour,
            Some(4)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[1].children[0].children[0].down_neighbour,
            Some(5)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[1].children[1].children[0].left_neighbour,
            Some(3)
        );

        // Test disk -> processes, processes -> process sort, process sort -> network
        assert_eq!(
            ret_bottom_layout.rows[1].children[1].children[1].children[0].down_neighbour,
            Some(7)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[1].left_neighbour,
            Some(9)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[0].left_neighbour,
            Some(6)
        );
    }

    #[cfg(feature = "battery")]
    #[test]
    /// Tests battery movement in the default setup.
    fn test_default_battery_movement() {
        use crate::constants::DEFAULT_BATTERY_LAYOUT;

        let rows = from_str::<Config>(DEFAULT_BATTERY_LAYOUT)
            .unwrap()
            .row
            .unwrap();
        let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, false);

        // Simple tests for the top CPU widget
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
            Some(4)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
            Some(2)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
            None
        );

        // Test CPU legend
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
            Some(5)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
            Some(3)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
            Some(1)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
            None
        );
    }

    #[test]
    /// Tests using cpu_left_legend.
    fn test_cpu_left_legend() {
        let rows = from_str::<Config>(DEFAULT_LAYOUT).unwrap().row.unwrap();
        let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, true);

        // Legend
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
            Some(3)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
            Some(1)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
            None
        );

        // Widget
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
            Some(3)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
            Some(2)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
            None
        );
    }

    #[test]
    /// Tests explicit default widget.
    fn test_default_widget_in_layout() {
        let proc_layout = r#"
    [[row]]
        [[row.child]]
            type="proc"
    [[row]]
        [[row.child]]
            type="proc"
        [[row.child]]
            type="proc"
    [[row]]
        [[row.child]]
            type="proc"
            default=true
        [[row.child]]
            type="proc"
    "#;

        let rows = from_str::<Config>(proc_layout).unwrap().row.unwrap();
        let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
        let mut total_height_ratio = 0;
        let mut default_widget_count = 1;
        let mut default_widget_id = DEFAULT_WIDGET_ID;
        let default_widget_type = None;
        let cpu_left_legend = false;

        let mut ret_bottom_layout = BottomLayout {
            rows: rows
                .iter()
                .map(|row| {
                    row.convert_row_to_bottom_row(
                        &mut iter_id,
                        &mut total_height_ratio,
                        &mut default_widget_id,
                        &default_widget_type,
                        &mut default_widget_count,
                        cpu_left_legend,
                    )
                })
                .collect::<OptionResult<Vec<_>>>()
                .unwrap(),
            total_row_height_ratio: total_height_ratio,
        };
        ret_bottom_layout.get_movement_mappings();

        assert_eq!(default_widget_id, 10);
    }

    #[test]
    /// Tests default widget by setting type and count.
    fn test_default_widget_by_option() {
        let rows = from_str::<Config>(PROC_LAYOUT).unwrap().row.unwrap();
        let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
        let mut total_height_ratio = 0;
        let mut default_widget_count = 3;
        let mut default_widget_id = DEFAULT_WIDGET_ID;
        let default_widget_type = Some(BottomWidgetType::Proc);
        let cpu_left_legend = false;

        let mut ret_bottom_layout = BottomLayout {
            rows: rows
                .iter()
                .map(|row| {
                    row.convert_row_to_bottom_row(
                        &mut iter_id,
                        &mut total_height_ratio,
                        &mut default_widget_id,
                        &default_widget_type,
                        &mut default_widget_count,
                        cpu_left_legend,
                    )
                })
                .collect::<OptionResult<Vec<_>>>()
                .unwrap(),
            total_row_height_ratio: total_height_ratio,
        };
        ret_bottom_layout.get_movement_mappings();

        assert_eq!(default_widget_id, 7);
    }

    #[test]
    fn test_proc_custom_layout() {
        let rows = from_str::<Config>(PROC_LAYOUT).unwrap().row.unwrap();
        let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, false);

        // First proc widget
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
            Some(2)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
            Some(3)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
            None
        );

        // Its search
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[1].children[0].down_neighbour,
            Some(4)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[1].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[1].children[0].right_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[1].children[0].up_neighbour,
            Some(1)
        );

        // Its sort
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
            Some(2)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
            Some(1)
        );
        assert_eq!(
            ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
            None
        );

        // Let us now test the second row's first widget...
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[1].down_neighbour,
            Some(5)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[1].left_neighbour,
            Some(6)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[1].right_neighbour,
            Some(9)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[1].up_neighbour,
            Some(2)
        );

        // Sort
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[0].down_neighbour,
            Some(5)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[0].right_neighbour,
            Some(4)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[0].children[0].up_neighbour,
            Some(2)
        );

        // Search
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[1].children[0].down_neighbour,
            Some(10)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[1].children[0].left_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[1].children[0].right_neighbour,
            Some(8)
        );
        assert_eq!(
            ret_bottom_layout.rows[1].children[0].children[1].children[0].up_neighbour,
            Some(4)
        );

        // Third row, second
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[1].down_neighbour,
            Some(14)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[1].left_neighbour,
            Some(15)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[1].right_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[1].up_neighbour,
            Some(8)
        );

        // Sort
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[0].down_neighbour,
            Some(14)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[0].left_neighbour,
            Some(10)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[0].right_neighbour,
            Some(13)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[0].children[0].up_neighbour,
            Some(8)
        );

        // Search
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[1].children[0].down_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[1].children[0].left_neighbour,
            Some(11)
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[1].children[0].right_neighbour,
            None
        );
        assert_eq!(
            ret_bottom_layout.rows[2].children[1].children[1].children[0].up_neighbour,
            Some(13)
        );
    }
}
