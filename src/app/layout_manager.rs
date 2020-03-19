use crate::error::{BottomError, Result};
use std::collections::{BTreeMap, HashMap};

/// Represents a more usable representation of the layout, derived from the
/// config.
#[derive(Clone, Debug)]
pub struct BottomLayout {
    pub rows: Vec<BottomRow>,
    pub total_height_ratio: u32,
}

type WidgetMappings = (u32, BTreeMap<u32, u64>);
type ColumnMappings = (u32, BTreeMap<u32, WidgetMappings>);
impl BottomLayout {
    pub fn convert_to_hashmap(&self) -> HashMap<u64, BottomWidget> {
        let mut return_map: HashMap<u64, BottomWidget> = HashMap::new();

        for row in &self.rows {
            for col in &row.children {
                for widget in &col.children {
                    return_map.insert(widget.widget_id, widget.clone());
                }
            }
        }

        return_map
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn get_movement_mappings(&mut self) {
        // Now we need to create the correct mapping for moving from a specific
        // widget to another

        let mut layout_mapping: BTreeMap<u32, ColumnMappings> = BTreeMap::new();
        let mut total_height = 0;
        for row in &self.rows {
            total_height += row.row_ratio;
            let mut row_width = 0;
            let mut row_mapping: BTreeMap<u32, WidgetMappings> = BTreeMap::new();
            let mut is_valid_row = false;
            for col in &row.children {
                let mut col_height = 0;
                let mut col_mapping: BTreeMap<u32, u64> = BTreeMap::new();
                let mut is_valid_col = false;
                row_width += col.width_ratio;

                for widget in &col.children {
                    col_height += widget.height_ratio;

                    match widget.widget_type {
                        BottomWidgetType::Empty => {}
                        _ => {
                            is_valid_col = true;
                            col_mapping.insert(
                                col_height * 100 / col.total_widget_ratio,
                                widget.widget_id,
                            );
                        }
                    }
                }
                if is_valid_col {
                    row_mapping.insert(
                        row_width * 100 / row.total_col_ratio,
                        (row.total_col_ratio, col_mapping),
                    );
                    is_valid_row = true;
                }
            }
            if is_valid_row {
                layout_mapping.insert(
                    total_height * 100 / self.total_height_ratio,
                    (self.total_height_ratio, row_mapping),
                );
            }
        }

        // debug!("Map: {:?}", layout_mapping);

        // Now pass through a second time; this time we want to build up
        // our neighbour profile.
        let mut height_cursor = 0;
        for row in &mut self.rows {
            let mut col_cursor = 0;
            height_cursor += row.row_ratio;
            let height_percentage = height_cursor * 100 / self.total_height_ratio;

            // debug!("Height percentage: {}", height_percentage);

            for col in &mut row.children {
                let mut widget_cursor = 0;
                col_cursor += col.width_ratio;
                let col_percentage = col_cursor * 100 / row.total_col_ratio;

                for widget in &mut col.children {
                    widget_cursor += widget.height_ratio;

                    // Bail if empty.
                    if let BottomWidgetType::Empty = widget.widget_type {
                        continue;
                    }

                    let widget_percentage = widget_cursor * 100 / col.total_widget_ratio;

                    if let Some(row) = layout_mapping.get(&height_percentage) {
                        // Check right in same row
                        if let Some(to_right) = row.1.range(col_percentage + 1..).next() {
                            if let Some(right_neighbour) =
                                (to_right.1).1.range(..widget_percentage).next_back()
                            {
                                widget.right_neighbour = Some(*right_neighbour.1);
                            // debug!("Set right neighbour to: {:?}", widget.right_neighbour);
                            } else if let Some(right_neighbour) =
                                (to_right.1).1.range(widget_percentage..).next_back()
                            {
                                widget.right_neighbour = Some(*right_neighbour.1);
                                // debug!("Set right neighbour to: {:?}", widget.right_neighbour);
                            }
                        }

                        // Check left in same row
                        if let Some(to_left) = row.1.range(..col_percentage).next_back() {
                            // Similar logic to checking for right neighbours.
                            if let Some(left_neighbour) =
                                (to_left.1).1.range(..widget_percentage).next_back()
                            {
                                widget.left_neighbour = Some(*left_neighbour.1);
                            // debug!("Set left neighbour to: {:?}", widget.left_neighbour);
                            } else if let Some(left_neighbour) =
                                (to_left.1).1.range(widget_percentage..).next_back()
                            {
                                widget.left_neighbour = Some(*left_neighbour.1);
                                // debug!("Set left neighbour to: {:?}", widget.left_neighbour);
                            }
                        }

                        // Check up/down within same row;
                        // else check up/down with other rows
                        if let Some(col) = row.1.get(&col_percentage) {
                            if let Some(to_up) = col.1.range(..widget_percentage).next_back() {
                                // In this case, then we can simply just set this immediately!
                                widget.up_neighbour = Some(*to_up.1);
                            // debug!("Set up neighbour to {:?}", widget.up_neighbour);
                            } else if let Some(next_row_up) =
                                layout_mapping.range(..height_percentage).next_back()
                            {
                                // Try to get the closest widget in the same
                                // column vicinity, then get the one nearest
                                // at the bottom of this column stack!

                                if let Some(column_candidate) =
                                    (next_row_up.1).1.range(..col_percentage).next_back()
                                {
                                    if let Some(result) =
                                        ((column_candidate).1).1.iter().next_back()
                                    {
                                        widget.up_neighbour = Some(*result.1);
                                        // debug!("Set up neighbour to {:?}", widget.up_neighbour);
                                    }
                                } else if let Some(column_candidate) =
                                    (next_row_up.1).1.range(col_percentage..).next_back()
                                {
                                    if let Some(result) =
                                        ((column_candidate).1).1.iter().next_back()
                                    {
                                        widget.up_neighbour = Some(*result.1);
                                        // debug!("Set up neighbour to {:?}", widget.up_neighbour);
                                    }
                                }
                            }

                            if let Some(to_down) = col.1.range(widget_percentage + 1..).next() {
                                widget.down_neighbour = Some(*to_down.1);
                            // debug!("Set down neighbour to {:?}", widget.down_neighbour);
                            } else if let Some(next_row_down) =
                                layout_mapping.range(height_percentage + 1..).next()
                            {
                                if let Some(column_candidate) =
                                    (next_row_down.1).1.range(..col_percentage).next_back()
                                {
                                    if let Some(result) = ((column_candidate).1).1.iter().next() {
                                        widget.down_neighbour = Some(*result.1);
                                        // debug!("Set down neighbour to {:?}", widget.down_neighbour);
                                    }
                                } else if let Some(column_candidate) =
                                    (next_row_down.1).1.range(col_percentage..).next_back()
                                {
                                    if let Some(result) = ((column_candidate).1).1.iter().next() {
                                        widget.down_neighbour = Some(*result.1);
                                        // debug!("Set down neighbour to {:?}", widget.down_neighbour);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for BottomLayout {
    fn default() -> Self {
        BottomLayout {
            total_height_ratio: 10,
            rows: vec![
                BottomRow {
                    total_col_ratio: 1,
                    row_ratio: 3,
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
                    row_ratio: 4,
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
                                    up_neighbour: Some(2),
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
                    row_ratio: 3,
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
