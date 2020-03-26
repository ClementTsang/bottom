use crate::error::{BottomError, Result};
use std::collections::BTreeMap;

/// Represents a more usable representation of the layout, derived from the
/// config.
#[derive(Clone, Debug)]
pub struct BottomLayout {
    pub rows: Vec<BottomRow>,
    pub total_height_ratio: u32,
}

type WidgetMappings = (u32, BTreeMap<(u32, u32), u64>);
type ColumnMappings = (u32, BTreeMap<(u32, u32), WidgetMappings>);

impl BottomLayout {
    #[allow(clippy::cognitive_complexity)]
    pub fn get_movement_mappings(&mut self) {
        fn is_intersecting(a: (u32, u32), b: (u32, u32)) -> bool {
            a.0 >= b.0 && a.1 <= b.1
                || a.1 >= b.1 && a.0 <= b.0
                || a.0 <= b.0 && a.1 >= b.0
                || a.0 >= b.0 && a.0 < b.1 && a.1 >= b.1
        }

        // Now we need to create the correct mapping for moving from a specific
        // widget to another

        let mut layout_mapping: BTreeMap<(u32, u32), ColumnMappings> = BTreeMap::new();
        let mut total_height = 0;
        for row in &self.rows {
            let mut row_width = 0;
            let mut row_mapping: BTreeMap<(u32, u32), WidgetMappings> = BTreeMap::new();
            let mut is_valid_row = false;
            for col in &row.children {
                let mut col_height = 0;
                let mut col_mapping: BTreeMap<(u32, u32), u64> = BTreeMap::new();
                let mut is_valid_col = false;

                for widget in &col.children {
                    match widget.widget_type {
                        BottomWidgetType::Empty => {}
                        _ => {
                            is_valid_col = true;
                            col_mapping.insert(
                                (
                                    col_height * 100 / col.total_widget_ratio,
                                    (col_height + widget.height_ratio) * 100
                                        / col.total_widget_ratio,
                                ),
                                widget.widget_id,
                            );
                        }
                    }

                    col_height += widget.height_ratio;
                }
                if is_valid_col {
                    row_mapping.insert(
                        (
                            row_width * 100 / row.total_col_ratio,
                            (row_width + col.width_ratio) * 100 / row.total_col_ratio,
                        ),
                        (row.total_col_ratio, col_mapping),
                    );
                    is_valid_row = true;
                }

                row_width += col.width_ratio;
            }
            if is_valid_row {
                layout_mapping.insert(
                    (
                        total_height * 100 / self.total_height_ratio,
                        (total_height + row.row_ratio) * 100 / self.total_height_ratio,
                    ),
                    (self.total_height_ratio, row_mapping),
                );
            }
            total_height += row.row_ratio;
        }

        // Now pass through a second time; this time we want to build up
        // our neighbour profile.
        let mut height_cursor = 0;
        for row in &mut self.rows {
            // Avoid dbz
            if self.total_height_ratio == 0 {
                continue;
            }

            let mut col_cursor = 0;
            let height_percentage_start = height_cursor * 100 / self.total_height_ratio;
            let height_percentage_end =
                (height_cursor + row.row_ratio) * 100 / self.total_height_ratio;

            for col in &mut row.children {
                // Avoid dbz
                if row.total_col_ratio == 0 {
                    continue;
                }

                let mut widget_cursor = 0;
                let col_percentage_start = col_cursor * 100 / row.total_col_ratio;
                let col_percentage_end = (col_cursor + col.width_ratio) * 100 / row.total_col_ratio;

                for widget in &mut col.children {
                    // Bail if empty.
                    if let BottomWidgetType::Empty = widget.widget_type {
                        continue;
                    }

                    // Avoid dbz
                    if col.total_widget_ratio == 0 {
                        continue;
                    }

                    let widget_percentage_start = widget_cursor * 100 / col.total_widget_ratio;
                    let widget_percentage_end =
                        (widget_cursor + widget.height_ratio) * 100 / col.total_widget_ratio;

                    if let Some(current_row) =
                        layout_mapping.get(&(height_percentage_start, height_percentage_end))
                    {
                        // Check right in same row
                        if let Some(to_right_col) = current_row
                            .1
                            .range((col_percentage_end, col_percentage_end)..)
                            .next()
                        {
                            let mut current_best_distance = 0;
                            let mut current_best_widget_id = widget.widget_id;

                            for widget_position in &(to_right_col.1).1 {
                                let candidate_start = (widget_position.0).0;
                                let candidate_end = (widget_position.0).1;

                                if is_intersecting(
                                    (widget_percentage_start, widget_percentage_end),
                                    (candidate_start, candidate_end),
                                ) {
                                    let candidate_distance =
                                        if candidate_start < widget_percentage_start {
                                            candidate_end - widget_percentage_start
                                        } else if candidate_end < widget_percentage_end {
                                            candidate_end - candidate_start
                                        } else {
                                            widget_percentage_end - candidate_start
                                        };

                                    if current_best_distance < candidate_distance {
                                        current_best_distance = candidate_distance + 1;
                                        current_best_widget_id = *(widget_position.1);
                                    }
                                }
                            }
                            if current_best_distance > 0 {
                                widget.right_neighbour = Some(current_best_widget_id);
                            }
                        }

                        // Check left in same row
                        if let Some(to_left_col) = current_row
                            .1
                            .range(..(col_percentage_start, col_percentage_end))
                            .next_back()
                        {
                            let mut current_best_distance = 0;
                            let mut current_best_widget_id = widget.widget_id;

                            for widget_position in &(to_left_col.1).1 {
                                let candidate_start = (widget_position.0).0;
                                let candidate_end = (widget_position.0).1;

                                if is_intersecting(
                                    (widget_percentage_start, widget_percentage_end),
                                    (candidate_start, candidate_end),
                                ) {
                                    let candidate_distance =
                                        if candidate_start < widget_percentage_start {
                                            candidate_end - widget_percentage_start
                                        } else if candidate_end < widget_percentage_end {
                                            candidate_end - candidate_start
                                        } else {
                                            widget_percentage_end - candidate_start
                                        };

                                    if current_best_distance < candidate_distance {
                                        current_best_distance = candidate_distance + 1;
                                        current_best_widget_id = *(widget_position.1);
                                    }
                                }
                            }
                            if current_best_distance > 0 {
                                widget.left_neighbour = Some(current_best_widget_id);
                            }
                        }

                        // Check up/down within same row;
                        // else check up/down with other rows
                        if let Some(current_col) = current_row
                            .1
                            .get(&(col_percentage_start, col_percentage_end))
                        {
                            if let Some(to_up) = current_col
                                .1
                                .range(..(widget_percentage_start, widget_percentage_start))
                                .next_back()
                            {
                                // In this case, then we can simply just set this immediately!
                                widget.up_neighbour = Some(*to_up.1);
                            } else if let Some(next_row_up) = layout_mapping
                                .range(..(height_percentage_start, height_percentage_start))
                                .next_back()
                            {
                                let col_percentage_end =
                                    (col_cursor + col.width_ratio) * 100 / row.total_col_ratio;

                                // We want to get the widget with the highest percentage WITHIN our two ranges
                                let mut current_best_distance = 0;
                                let mut current_best_widget_id = widget.widget_id;
                                for col_position in &(next_row_up.1).1 {
                                    let candidate_start = (col_position.0).0;
                                    let candidate_end = (col_position.0).1;

                                    if is_intersecting(
                                        (col_percentage_start, col_percentage_end),
                                        (candidate_start, candidate_end),
                                    ) {
                                        let candidate_distance =
                                            if candidate_start < col_percentage_start {
                                                candidate_end - col_percentage_start
                                            } else if candidate_end < col_percentage_end {
                                                candidate_end - candidate_start
                                            } else {
                                                col_percentage_end - candidate_start
                                            };

                                        if current_best_distance < candidate_distance {
                                            if let Some(current_best_widget) =
                                                (col_position.1).1.iter().next_back()
                                            {
                                                current_best_distance = candidate_distance + 1;
                                                current_best_widget_id = *(current_best_widget.1);
                                            }
                                        }
                                    }
                                }

                                if current_best_distance > 0 {
                                    widget.up_neighbour = Some(current_best_widget_id);
                                }
                            }

                            if let Some(to_down) = current_col
                                .1
                                .range((widget_percentage_start + 1, widget_percentage_start + 1)..)
                                .next()
                            {
                                widget.down_neighbour = Some(*to_down.1);
                            } else if let Some(next_row_down) = layout_mapping
                                .range((height_percentage_start + 1, height_percentage_start + 1)..)
                                .next()
                            {
                                let col_percentage_end =
                                    (col_cursor + col.width_ratio) * 100 / row.total_col_ratio;

                                // We want to get the widget with the highest percentage WITHIN our two ranges
                                let mut current_best_distance = 0;
                                let mut current_best_widget_id = widget.widget_id;

                                for col_position in &(next_row_down.1).1 {
                                    let candidate_start = (col_position.0).0;
                                    let candidate_end = (col_position.0).1;

                                    if is_intersecting(
                                        (col_percentage_start, col_percentage_end),
                                        (candidate_start, candidate_end),
                                    ) {
                                        let candidate_distance =
                                            if candidate_start < col_percentage_start {
                                                candidate_end - col_percentage_start
                                            } else if candidate_end < col_percentage_end {
                                                candidate_end - candidate_start
                                            } else {
                                                col_percentage_end - candidate_start
                                            };

                                        if current_best_distance < candidate_distance {
                                            if let Some(current_best_widget) =
                                                (col_position.1).1.iter().next()
                                            {
                                                current_best_distance = candidate_distance + 1;
                                                current_best_widget_id = *(current_best_widget.1);
                                            }
                                        }
                                    }
                                }

                                if current_best_distance > 0 {
                                    widget.down_neighbour = Some(current_best_widget_id);
                                }
                            }
                        }
                    }
                    widget_cursor += widget.height_ratio;
                }
                col_cursor += col.width_ratio;
            }
            height_cursor += row.row_ratio;
        }
    }

    pub fn init_default() -> Self {
        BottomLayout {
            total_height_ratio: 100,
            rows: vec![
                BottomRow {
                    total_col_ratio: 1,
                    row_ratio: 30,
                    children: vec![BottomCol {
                        total_widget_ratio: 1,
                        width_ratio: 1,
                        children: vec![BottomWidget {
                            height_ratio: 1,
                            widget_type: BottomWidgetType::Cpu,
                            widget_id: 1,
                            left_neighbour: None,
                            right_neighbour: None,
                            up_neighbour: None,
                            down_neighbour: Some(11),
                        }],
                    }],
                },
                BottomRow {
                    total_col_ratio: 7,
                    row_ratio: 40,
                    children: vec![
                        BottomCol {
                            total_widget_ratio: 1,
                            width_ratio: 4,
                            children: vec![BottomWidget {
                                height_ratio: 1,
                                widget_type: BottomWidgetType::Mem,
                                widget_id: 11,
                                left_neighbour: None,
                                right_neighbour: Some(12),
                                up_neighbour: Some(1),
                                down_neighbour: Some(21),
                            }],
                        },
                        BottomCol {
                            total_widget_ratio: 2,
                            width_ratio: 3,
                            children: vec![
                                BottomWidget {
                                    height_ratio: 1,
                                    widget_type: BottomWidgetType::Temp,
                                    widget_id: 12,
                                    left_neighbour: Some(11),
                                    right_neighbour: None,
                                    up_neighbour: Some(1),
                                    down_neighbour: Some(13),
                                },
                                BottomWidget {
                                    height_ratio: 1,
                                    widget_type: BottomWidgetType::Disk,
                                    widget_id: 13,
                                    left_neighbour: Some(11),
                                    right_neighbour: None,
                                    up_neighbour: Some(12),
                                    down_neighbour: Some(22),
                                },
                            ],
                        },
                    ],
                },
                BottomRow {
                    total_col_ratio: 2,
                    row_ratio: 30,
                    children: vec![
                        BottomCol {
                            total_widget_ratio: 1,
                            width_ratio: 1,
                            children: vec![BottomWidget {
                                height_ratio: 1,
                                widget_type: BottomWidgetType::Net,
                                widget_id: 21,
                                left_neighbour: None,
                                right_neighbour: Some(22),
                                up_neighbour: Some(11),
                                down_neighbour: None,
                            }],
                        },
                        BottomCol {
                            total_widget_ratio: 1,
                            width_ratio: 1,
                            children: vec![BottomWidget {
                                height_ratio: 1,
                                widget_type: BottomWidgetType::Proc,
                                widget_id: 22,
                                left_neighbour: Some(21),
                                right_neighbour: None,
                                up_neighbour: Some(13),
                                down_neighbour: None,
                            }],
                        },
                    ],
                },
            ],
        }
    }
}

/// Represents a single row in the layout.
#[derive(Clone, Debug)]
pub struct BottomRow {
    pub row_ratio: u32,
    pub children: Vec<BottomCol>,
    pub total_col_ratio: u32,
}

/// Represents a single column in the layout.  We assume that even if the column
/// contains only ONE element, it is still a column (rather than either a col or
/// a widget, as per the config, for simplicity's sake).
#[derive(Clone, Debug)]
pub struct BottomCol {
    pub width_ratio: u32,
    pub children: Vec<BottomWidget>,
    pub total_widget_ratio: u32,
}

/// Represents a single widget.
#[derive(Debug, Default, Clone)]
pub struct BottomWidget {
    pub height_ratio: u32,
    pub widget_type: BottomWidgetType,
    pub widget_id: u64,
    pub left_neighbour: Option<u64>,
    pub right_neighbour: Option<u64>,
    pub up_neighbour: Option<u64>,
    pub down_neighbour: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum BottomWidgetType {
    Empty,
    Cpu,
    Mem,
    Net,
    Proc,
    Temp,
    Disk,
}

impl BottomWidgetType {
    pub fn is_widget_table(&self) -> bool {
        use BottomWidgetType::*;
        match self {
            Disk | Proc | Temp => true,
            _ => false,
        }
    }

    pub fn is_widget_graph(&self) -> bool {
        use BottomWidgetType::*;
        match self {
            Cpu | Net | Mem => true,
            _ => false,
        }
    }

    pub fn get_pretty_name(&self) -> &str {
        use BottomWidgetType::*;
        match self {
            Cpu => "CPU",
            Mem => "Memory",
            Net => "Network",
            Proc => "Processes",
            Temp => "Temperature",
            Disk => "Disks",
            _ => "",
        }
    }
}

impl Default for BottomWidgetType {
    fn default() -> Self {
        BottomWidgetType::Empty
    }
}

impl std::str::FromStr for BottomWidgetType {
    type Err = BottomError;

    fn from_str(s: &str) -> Result<Self> {
        let lower_case = s.to_lowercase();
        match lower_case.as_str() {
            "cpu" => Ok(BottomWidgetType::Cpu),
            "mem" => Ok(BottomWidgetType::Mem),
            "net" => Ok(BottomWidgetType::Net),
            "proc" => Ok(BottomWidgetType::Proc),
            "temp" => Ok(BottomWidgetType::Temp),
            "disk" => Ok(BottomWidgetType::Disk),
            "empty" => Ok(BottomWidgetType::Empty),
            _ => Err(BottomError::ConfigError(format!(
                "Invalid widget type: {}",
                s
            ))),
        }
    }
}
