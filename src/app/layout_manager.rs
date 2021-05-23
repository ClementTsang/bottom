use crate::error::{BottomError, Result};
use std::collections::BTreeMap;
use typed_builder::*;

use crate::constants::DEFAULT_WIDGET_ID;

/// Represents a more usable representation of the layout, derived from the
/// config.
#[derive(Clone, Debug)]
pub struct BottomLayout {
    pub rows: Vec<BottomRow>,
    pub total_row_height_ratio: u32,
}

// Represents a start and end coordinate in some dimension.
type LineSegment = (u32, u32);

type WidgetMappings = (u32, BTreeMap<LineSegment, u64>);
type ColumnRowMappings = (u32, BTreeMap<LineSegment, WidgetMappings>);
type ColumnMappings = (u32, BTreeMap<LineSegment, ColumnRowMappings>);

impl BottomLayout {
    pub fn get_movement_mappings(&mut self) {
        #[allow(clippy::suspicious_operation_groupings)] // Have to enable this, clippy really doesn't like me doing this with tuples...
        fn is_intersecting(a: LineSegment, b: LineSegment) -> bool {
            a.0 >= b.0 && a.1 <= b.1
                || a.1 >= b.1 && a.0 <= b.0
                || a.0 <= b.0 && a.1 >= b.0
                || a.0 >= b.0 && a.0 < b.1 && a.1 >= b.1
        }

        fn get_distance(target: LineSegment, candidate: LineSegment) -> u32 {
            if candidate.0 < target.0 {
                candidate.1 - target.0
            } else if candidate.1 < target.1 {
                candidate.1 - candidate.0
            } else {
                target.1 - candidate.0
            }
        }

        // Now we need to create the correct mapping for moving from a specific
        // widget to another

        let mut layout_mapping: BTreeMap<LineSegment, ColumnMappings> = BTreeMap::new();
        let mut total_height = 0;
        for row in &self.rows {
            let mut row_width = 0;
            let mut row_mapping: BTreeMap<LineSegment, ColumnRowMappings> = BTreeMap::new();
            let mut is_valid_row = false;
            for col in &row.children {
                let mut col_row_height = 0;
                let mut col_mapping: BTreeMap<LineSegment, WidgetMappings> = BTreeMap::new();
                let mut is_valid_col = false;

                for col_row in &col.children {
                    let mut widget_width = 0;
                    let mut col_row_mapping: BTreeMap<LineSegment, u64> = BTreeMap::new();
                    let mut is_valid_col_row = false;
                    for widget in &col_row.children {
                        match widget.widget_type {
                            BottomWidgetType::Empty => {}
                            _ => {
                                is_valid_col_row = true;
                                col_row_mapping.insert(
                                    (
                                        widget_width * 100 / col_row.total_widget_ratio,
                                        (widget_width + widget.width_ratio) * 100
                                            / col_row.total_widget_ratio,
                                    ),
                                    widget.widget_id,
                                );
                            }
                        }
                        widget_width += widget.width_ratio;
                    }
                    if is_valid_col_row {
                        col_mapping.insert(
                            (
                                col_row_height * 100 / col.total_col_row_ratio,
                                (col_row_height + col_row.col_row_height_ratio) * 100
                                    / col.total_col_row_ratio,
                            ),
                            (col.total_col_row_ratio, col_row_mapping),
                        );
                        is_valid_col = true;
                    }

                    col_row_height += col_row.col_row_height_ratio;
                }
                if is_valid_col {
                    row_mapping.insert(
                        (
                            row_width * 100 / row.total_col_ratio,
                            (row_width + col.col_width_ratio) * 100 / row.total_col_ratio,
                        ),
                        (row.total_col_ratio, col_mapping),
                    );
                    is_valid_row = true;
                }

                row_width += col.col_width_ratio;
            }
            if is_valid_row {
                layout_mapping.insert(
                    (
                        total_height * 100 / self.total_row_height_ratio,
                        (total_height + row.row_height_ratio) * 100 / self.total_row_height_ratio,
                    ),
                    (self.total_row_height_ratio, row_mapping),
                );
            }
            total_height += row.row_height_ratio;
        }

        // Now pass through a second time; this time we want to build up
        // our neighbour profile.
        let mut height_cursor = 0;
        for row in &mut self.rows {
            let mut col_cursor = 0;
            let row_height_percentage_start = height_cursor * 100 / self.total_row_height_ratio;
            let row_height_percentage_end =
                (height_cursor + row.row_height_ratio) * 100 / self.total_row_height_ratio;

            for col in &mut row.children {
                let mut col_row_cursor = 0;
                let col_width_percentage_start = col_cursor * 100 / row.total_col_ratio;
                let col_width_percentage_end =
                    (col_cursor + col.col_width_ratio) * 100 / row.total_col_ratio;

                for col_row in &mut col.children {
                    let mut widget_cursor = 0;
                    let col_row_height_percentage_start =
                        col_row_cursor * 100 / col.total_col_row_ratio;
                    let col_row_height_percentage_end =
                        (col_row_cursor + col_row.col_row_height_ratio) * 100
                            / col.total_col_row_ratio;
                    let col_row_children_len = col_row.children.len();

                    for widget in &mut col_row.children {
                        // Bail if empty.
                        if let BottomWidgetType::Empty = widget.widget_type {
                            continue;
                        }

                        let widget_width_percentage_start =
                            widget_cursor * 100 / col_row.total_widget_ratio;
                        let widget_width_percentage_end =
                            (widget_cursor + widget.width_ratio) * 100 / col_row.total_widget_ratio;

                        if let Some(current_row) = layout_mapping
                            .get(&(row_height_percentage_start, row_height_percentage_end))
                        {
                            // First check for within the same col_row for left and right
                            if let Some(current_col) = current_row
                                .1
                                .get(&(col_width_percentage_start, col_width_percentage_end))
                            {
                                if let Some(current_col_row) = current_col.1.get(&(
                                    col_row_height_percentage_start,
                                    col_row_height_percentage_end,
                                )) {
                                    if let Some(to_left_widget) = current_col_row
                                        .1
                                        .range(
                                            ..(
                                                widget_width_percentage_start,
                                                widget_width_percentage_start,
                                            ),
                                        )
                                        .next_back()
                                    {
                                        widget.left_neighbour = Some(*to_left_widget.1);
                                    }

                                    // Right
                                    if let Some(to_right_neighbour) = current_col_row
                                        .1
                                        .range(
                                            (
                                                widget_width_percentage_end,
                                                widget_width_percentage_end,
                                            )..,
                                        )
                                        .next()
                                    {
                                        widget.right_neighbour = Some(*to_right_neighbour.1);
                                    }
                                }
                            }

                            if widget.left_neighbour.is_none() {
                                if let Some(to_left_col) = current_row
                                    .1
                                    .range(
                                        ..(col_width_percentage_start, col_width_percentage_start),
                                    )
                                    .next_back()
                                {
                                    // Check left in same row
                                    let mut current_best_distance = 0;
                                    let mut current_best_widget_id = widget.widget_id;

                                    for widget_position in &(to_left_col.1).1 {
                                        let candidate_start = (widget_position.0).0;
                                        let candidate_end = (widget_position.0).1;

                                        if is_intersecting(
                                            (
                                                col_row_height_percentage_start,
                                                col_row_height_percentage_end,
                                            ),
                                            (candidate_start, candidate_end),
                                        ) {
                                            let candidate_distance = get_distance(
                                                (
                                                    col_row_height_percentage_start,
                                                    col_row_height_percentage_end,
                                                ),
                                                (candidate_start, candidate_end),
                                            );

                                            if current_best_distance < candidate_distance {
                                                if let Some(new_best_widget) =
                                                    (widget_position.1).1.iter().next_back()
                                                {
                                                    current_best_distance = candidate_distance + 1;
                                                    current_best_widget_id = *(new_best_widget.1);
                                                }
                                            }
                                        }
                                    }
                                    if current_best_distance > 0 {
                                        widget.left_neighbour = Some(current_best_widget_id);
                                    }
                                }
                            }

                            if widget.right_neighbour.is_none() {
                                if let Some(to_right_col) = current_row
                                    .1
                                    .range((col_width_percentage_end, col_width_percentage_end)..)
                                    .next()
                                {
                                    // Check right in same row
                                    let mut current_best_distance = 0;
                                    let mut current_best_widget_id = widget.widget_id;

                                    for widget_position in &(to_right_col.1).1 {
                                        let candidate_start = (widget_position.0).0;
                                        let candidate_end = (widget_position.0).1;

                                        if is_intersecting(
                                            (
                                                col_row_height_percentage_start,
                                                col_row_height_percentage_end,
                                            ),
                                            (candidate_start, candidate_end),
                                        ) {
                                            let candidate_distance = get_distance(
                                                (
                                                    col_row_height_percentage_start,
                                                    col_row_height_percentage_end,
                                                ),
                                                (candidate_start, candidate_end),
                                            );

                                            if current_best_distance < candidate_distance {
                                                if let Some(new_best_widget) =
                                                    (widget_position.1).1.iter().next()
                                                {
                                                    current_best_distance = candidate_distance + 1;
                                                    current_best_widget_id = *(new_best_widget.1);
                                                }
                                            }
                                        }
                                    }
                                    if current_best_distance > 0 {
                                        widget.right_neighbour = Some(current_best_widget_id);
                                    }
                                }
                            }

                            // Check up/down within same row;
                            // else check up/down with other rows
                            if let Some(current_col) = current_row
                                .1
                                .get(&(col_width_percentage_start, col_width_percentage_end))
                            {
                                if let Some(to_up) = current_col
                                    .1
                                    .range(
                                        ..(
                                            col_row_height_percentage_start,
                                            col_row_height_percentage_start,
                                        ),
                                    )
                                    .next_back()
                                {
                                    // Now check each widget_width and pick the best
                                    for candidate_widget in &(to_up.1).1 {
                                        let mut current_best_distance = 0;
                                        let mut current_best_widget_id = widget.widget_id;
                                        if is_intersecting(
                                            (
                                                widget_width_percentage_start,
                                                widget_width_percentage_end,
                                            ),
                                            ((candidate_widget.0).0, (candidate_widget.0).1),
                                        ) {
                                            let candidate_best_distance = get_distance(
                                                (
                                                    widget_width_percentage_start,
                                                    widget_width_percentage_end,
                                                ),
                                                ((candidate_widget.0).0, (candidate_widget.0).1),
                                            );

                                            if current_best_distance < candidate_best_distance {
                                                current_best_distance = candidate_best_distance + 1;
                                                current_best_widget_id = *candidate_widget.1;
                                            }
                                        }

                                        if current_best_distance > 0 {
                                            widget.up_neighbour = Some(current_best_widget_id);
                                        }
                                    }
                                } else {
                                    for next_row_up in layout_mapping
                                        .range(
                                            ..(
                                                row_height_percentage_start,
                                                row_height_percentage_start,
                                            ),
                                        )
                                        .rev()
                                    {
                                        let mut current_best_distance = 0;
                                        let mut current_best_widget_id = widget.widget_id;
                                        let (target_start_width, target_end_width) =
                                            if col_row_children_len > 1 {
                                                (
                                                    col_width_percentage_start
                                                        + widget_width_percentage_start
                                                            * (col_width_percentage_end
                                                                - col_width_percentage_start)
                                                            / 100,
                                                    col_width_percentage_start
                                                        + widget_width_percentage_end
                                                            * (col_width_percentage_end
                                                                - col_width_percentage_start)
                                                            / 100,
                                                )
                                            } else {
                                                (
                                                    col_width_percentage_start,
                                                    col_width_percentage_end,
                                                )
                                            };

                                        for col_position in &(next_row_up.1).1 {
                                            if let Some(next_col_row) =
                                                (col_position.1).1.iter().next_back()
                                            {
                                                let (candidate_col_start, candidate_col_end) =
                                                    ((col_position.0).0, (col_position.0).1);
                                                let candidate_difference =
                                                    candidate_col_end - candidate_col_start;
                                                for candidate_widget in &(next_col_row.1).1 {
                                                    let candidate_start = candidate_col_start
                                                        + (candidate_widget.0).0
                                                            * candidate_difference
                                                            / 100;
                                                    let candidate_end = candidate_col_start
                                                        + (candidate_widget.0).1
                                                            * candidate_difference
                                                            / 100;

                                                    if is_intersecting(
                                                        (target_start_width, target_end_width),
                                                        (candidate_start, candidate_end),
                                                    ) {
                                                        let candidate_distance = get_distance(
                                                            (target_start_width, target_end_width),
                                                            (candidate_start, candidate_end),
                                                        );

                                                        if current_best_distance
                                                            < candidate_distance
                                                        {
                                                            current_best_distance =
                                                                candidate_distance + 1;
                                                            current_best_widget_id =
                                                                *(candidate_widget.1);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if current_best_distance > 0 {
                                            widget.up_neighbour = Some(current_best_widget_id);
                                            break;
                                        }
                                    }
                                }

                                if let Some(to_down) = current_col
                                    .1
                                    .range(
                                        (
                                            col_row_height_percentage_start + 1,
                                            col_row_height_percentage_start + 1,
                                        )..,
                                    )
                                    .next()
                                {
                                    for candidate_widget in &(to_down.1).1 {
                                        let mut current_best_distance = 0;
                                        let mut current_best_widget_id = widget.widget_id;
                                        if is_intersecting(
                                            (
                                                widget_width_percentage_start,
                                                widget_width_percentage_end,
                                            ),
                                            ((candidate_widget.0).0, (candidate_widget.0).1),
                                        ) {
                                            let candidate_best_distance = get_distance(
                                                (
                                                    widget_width_percentage_start,
                                                    widget_width_percentage_end,
                                                ),
                                                ((candidate_widget.0).0, (candidate_widget.0).1),
                                            );

                                            if current_best_distance < candidate_best_distance {
                                                current_best_distance = candidate_best_distance + 1;
                                                current_best_widget_id = *candidate_widget.1;
                                            }
                                        }

                                        if current_best_distance > 0 {
                                            widget.down_neighbour = Some(current_best_widget_id);
                                        }
                                    }
                                } else {
                                    for next_row_down in layout_mapping.range(
                                        (
                                            row_height_percentage_start + 1,
                                            row_height_percentage_start + 1,
                                        )..,
                                    ) {
                                        let mut current_best_distance = 0;
                                        let mut current_best_widget_id = widget.widget_id;
                                        let (target_start_width, target_end_width) =
                                            if col_row_children_len > 1 {
                                                (
                                                    col_width_percentage_start
                                                        + widget_width_percentage_start
                                                            * (col_width_percentage_end
                                                                - col_width_percentage_start)
                                                            / 100,
                                                    col_width_percentage_start
                                                        + widget_width_percentage_end
                                                            * (col_width_percentage_end
                                                                - col_width_percentage_start)
                                                            / 100,
                                                )
                                            } else {
                                                (
                                                    col_width_percentage_start,
                                                    col_width_percentage_end,
                                                )
                                            };

                                        for col_position in &(next_row_down.1).1 {
                                            if let Some(next_col_row) =
                                                (col_position.1).1.iter().next()
                                            {
                                                let (candidate_col_start, candidate_col_end) =
                                                    ((col_position.0).0, (col_position.0).1);
                                                let candidate_difference =
                                                    candidate_col_end - candidate_col_start;
                                                for candidate_widget in &(next_col_row.1).1 {
                                                    let candidate_start = candidate_col_start
                                                        + (candidate_widget.0).0
                                                            * candidate_difference
                                                            / 100;
                                                    let candidate_end = candidate_col_start
                                                        + (candidate_widget.0).1
                                                            * candidate_difference
                                                            / 100;

                                                    if is_intersecting(
                                                        (target_start_width, target_end_width),
                                                        (candidate_start, candidate_end),
                                                    ) {
                                                        let candidate_distance = get_distance(
                                                            (target_start_width, target_end_width),
                                                            (candidate_start, candidate_end),
                                                        );

                                                        if current_best_distance
                                                            < candidate_distance
                                                        {
                                                            current_best_distance =
                                                                candidate_distance + 1;
                                                            current_best_widget_id =
                                                                *(candidate_widget.1);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if current_best_distance > 0 {
                                            widget.down_neighbour = Some(current_best_widget_id);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        widget_cursor += widget.width_ratio;
                    }
                    col_row_cursor += col_row.col_row_height_ratio;
                }
                col_cursor += col.col_width_ratio;
            }
            height_cursor += row.row_height_ratio;
        }
    }

    pub fn init_basic_default(use_battery: bool) -> Self {
        let table_widgets = if use_battery {
            vec![
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![BottomColRow::builder()
                        .canvas_handle_height(true)
                        .children(vec![BottomWidget::builder()
                            .canvas_handle_width(true)
                            .widget_type(BottomWidgetType::Disk)
                            .widget_id(4)
                            .up_neighbour(Some(100))
                            .left_neighbour(Some(8))
                            .right_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                            .build()])
                        .build()])
                    .build(),
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![
                        BottomColRow::builder()
                            .canvas_handle_height(true)
                            .total_widget_ratio(3)
                            .children(vec![
                                BottomWidget::builder()
                                    .canvas_handle_width(true)
                                    .widget_type(BottomWidgetType::ProcSort)
                                    .widget_id(DEFAULT_WIDGET_ID + 2)
                                    .up_neighbour(Some(100))
                                    .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                                    .left_neighbour(Some(4))
                                    .right_neighbour(Some(DEFAULT_WIDGET_ID))
                                    .width_ratio(1)
                                    .parent_reflector(Some((WidgetDirection::Right, 2)))
                                    .build(),
                                BottomWidget::builder()
                                    .canvas_handle_width(true)
                                    .widget_type(BottomWidgetType::Proc)
                                    .widget_id(DEFAULT_WIDGET_ID)
                                    .up_neighbour(Some(100))
                                    .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                                    .left_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                                    .right_neighbour(Some(7))
                                    .width_ratio(2)
                                    .build(),
                            ])
                            .build(),
                        BottomColRow::builder()
                            .canvas_handle_height(true)
                            .children(vec![BottomWidget::builder()
                                .canvas_handle_width(true)
                                .widget_type(BottomWidgetType::ProcSearch)
                                .widget_id(DEFAULT_WIDGET_ID + 1)
                                .up_neighbour(Some(DEFAULT_WIDGET_ID))
                                .left_neighbour(Some(4))
                                .right_neighbour(Some(7))
                                .parent_reflector(Some((WidgetDirection::Up, 1)))
                                .build()])
                            .build(),
                    ])
                    .build(),
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![BottomColRow::builder()
                        .canvas_handle_height(true)
                        .children(vec![BottomWidget::builder()
                            .canvas_handle_width(true)
                            .widget_type(BottomWidgetType::Temp)
                            .widget_id(7)
                            .up_neighbour(Some(100))
                            .left_neighbour(Some(DEFAULT_WIDGET_ID))
                            .right_neighbour(Some(8))
                            .build()])
                        .build()])
                    .build(),
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![BottomColRow::builder()
                        .canvas_handle_height(true)
                        .children(vec![BottomWidget::builder()
                            .canvas_handle_width(true)
                            .widget_type(BottomWidgetType::Battery)
                            .widget_id(8)
                            .up_neighbour(Some(100))
                            .left_neighbour(Some(7))
                            .right_neighbour(Some(4))
                            .build()])
                        .build()])
                    .build(),
            ]
        } else {
            vec![
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![BottomColRow::builder()
                        .canvas_handle_height(true)
                        .children(vec![BottomWidget::builder()
                            .canvas_handle_width(true)
                            .widget_type(BottomWidgetType::Disk)
                            .widget_id(4)
                            .up_neighbour(Some(100))
                            .left_neighbour(Some(7))
                            .right_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                            .build()])
                        .build()])
                    .build(),
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![
                        BottomColRow::builder()
                            .canvas_handle_height(true)
                            .children(vec![
                                BottomWidget::builder()
                                    .canvas_handle_width(true)
                                    .widget_type(BottomWidgetType::ProcSort)
                                    .widget_id(DEFAULT_WIDGET_ID + 2)
                                    .up_neighbour(Some(100))
                                    .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                                    .left_neighbour(Some(4))
                                    .right_neighbour(Some(DEFAULT_WIDGET_ID))
                                    .parent_reflector(Some((WidgetDirection::Right, 2)))
                                    .build(),
                                BottomWidget::builder()
                                    .canvas_handle_width(true)
                                    .widget_type(BottomWidgetType::Proc)
                                    .widget_id(DEFAULT_WIDGET_ID)
                                    .up_neighbour(Some(100))
                                    .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                                    .left_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                                    .right_neighbour(Some(7))
                                    .build(),
                            ])
                            .build(),
                        BottomColRow::builder()
                            .canvas_handle_height(true)
                            .children(vec![BottomWidget::builder()
                                .canvas_handle_width(true)
                                .widget_type(BottomWidgetType::ProcSearch)
                                .widget_id(DEFAULT_WIDGET_ID + 1)
                                .up_neighbour(Some(DEFAULT_WIDGET_ID))
                                .left_neighbour(Some(4))
                                .right_neighbour(Some(7))
                                .parent_reflector(Some((WidgetDirection::Up, 1)))
                                .build()])
                            .build(),
                    ])
                    .build(),
                BottomCol::builder()
                    .canvas_handle_width(true)
                    .children(vec![BottomColRow::builder()
                        .canvas_handle_height(true)
                        .children(vec![BottomWidget::builder()
                            .canvas_handle_width(true)
                            .widget_type(BottomWidgetType::Temp)
                            .widget_id(7)
                            .up_neighbour(Some(100))
                            .left_neighbour(Some(DEFAULT_WIDGET_ID))
                            .right_neighbour(Some(4))
                            .build()])
                        .build()])
                    .build(),
            ]
        };

        BottomLayout {
            total_row_height_ratio: 3,
            rows: vec![
                BottomRow::builder()
                    .canvas_handle_height(true)
                    .children(vec![BottomCol::builder()
                        .canvas_handle_width(true)
                        .children(vec![BottomColRow::builder()
                            .canvas_handle_height(true)
                            .children(vec![BottomWidget::builder()
                                .canvas_handle_width(true)
                                .widget_type(BottomWidgetType::BasicCpu)
                                .widget_id(1)
                                .down_neighbour(Some(2))
                                .build()])
                            .build()])
                        .build()])
                    .build(),
                BottomRow::builder()
                    .canvas_handle_height(true)
                    .children(vec![BottomCol::builder()
                        .canvas_handle_width(true)
                        .children(vec![BottomColRow::builder()
                            .canvas_handle_height(true)
                            .children(vec![
                                BottomWidget::builder()
                                    .canvas_handle_width(true)
                                    .widget_type(BottomWidgetType::BasicMem)
                                    .widget_id(2)
                                    .up_neighbour(Some(1))
                                    .down_neighbour(Some(100))
                                    .right_neighbour(Some(3))
                                    .build(),
                                BottomWidget::builder()
                                    .canvas_handle_width(true)
                                    .widget_type(BottomWidgetType::BasicNet)
                                    .widget_id(3)
                                    .up_neighbour(Some(1))
                                    .down_neighbour(Some(100))
                                    .left_neighbour(Some(2))
                                    .build(),
                            ])
                            .build()])
                        .build()])
                    .build(),
                BottomRow::builder()
                    .canvas_handle_height(true)
                    .children(vec![BottomCol::builder()
                        .canvas_handle_width(true)
                        .children(vec![BottomColRow::builder()
                            .canvas_handle_height(true)
                            .children(vec![BottomWidget::builder()
                                .canvas_handle_width(true)
                                .widget_type(BottomWidgetType::BasicTables)
                                .widget_id(100)
                                .up_neighbour(Some(2))
                                .build()])
                            .build()])
                        .build()])
                    .build(),
                BottomRow::builder()
                    .canvas_handle_height(true)
                    .children(table_widgets)
                    .build(),
            ],
        }
    }
}

/// Represents a single row in the layout.
#[derive(Clone, Debug, TypedBuilder)]
pub struct BottomRow {
    pub children: Vec<BottomCol>,

    #[builder(default = 1)]
    pub total_col_ratio: u32,

    #[builder(default = 1)]
    pub row_height_ratio: u32,

    #[builder(default = false)]
    pub canvas_handle_height: bool,

    #[builder(default = false)]
    pub flex_grow: bool,
}

/// Represents a single column in the layout.  We assume that even if the column
/// contains only ONE element, it is still a column (rather than either a col or
/// a widget, as per the config, for simplicity's sake).
#[derive(Clone, Debug, TypedBuilder)]
pub struct BottomCol {
    pub children: Vec<BottomColRow>,

    #[builder(default = 1)]
    pub total_col_row_ratio: u32,

    #[builder(default = 1)]
    pub col_width_ratio: u32,

    #[builder(default = false)]
    pub canvas_handle_width: bool,

    #[builder(default = false)]
    pub flex_grow: bool,
}

#[derive(Clone, Default, Debug, TypedBuilder)]
pub struct BottomColRow {
    pub children: Vec<BottomWidget>,

    #[builder(default = 1)]
    pub total_widget_ratio: u32,

    #[builder(default = 1)]
    pub col_row_height_ratio: u32,

    #[builder(default = false)]
    pub canvas_handle_height: bool,

    #[builder(default = false)]
    pub flex_grow: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WidgetDirection {
    Left,
    Right,
    Up,
    Down,
}

impl WidgetDirection {
    pub fn is_opposite(&self, other_direction: &WidgetDirection) -> bool {
        match &self {
            WidgetDirection::Left => *other_direction == WidgetDirection::Right,
            WidgetDirection::Right => *other_direction == WidgetDirection::Left,
            WidgetDirection::Up => *other_direction == WidgetDirection::Down,
            WidgetDirection::Down => *other_direction == WidgetDirection::Up,
        }
    }
}

/// Represents a single widget.
#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct BottomWidget {
    pub widget_type: BottomWidgetType,
    pub widget_id: u64,

    #[builder(default = 1)]
    pub width_ratio: u32,

    #[builder(default = None)]
    pub left_neighbour: Option<u64>,

    #[builder(default = None)]
    pub right_neighbour: Option<u64>,

    #[builder(default = None)]
    pub up_neighbour: Option<u64>,

    #[builder(default = None)]
    pub down_neighbour: Option<u64>,

    /// If set to true, the canvas will override any ratios.
    #[builder(default = false)]
    pub canvas_handle_width: bool,

    /// Whether we want this widget to take up all available room (and ignore any ratios).
    #[builder(default = false)]
    pub flex_grow: bool,

    /// The value is the direction to bounce, as well as the parent offset.
    #[builder(default = None)]
    pub parent_reflector: Option<(WidgetDirection, u64)>,

    /// Top left corner when drawn, for mouse click detection.  (x, y)
    #[builder(default = None)]
    pub top_left_corner: Option<(u16, u16)>,

    /// Bottom right corner when drawn, for mouse click detection.  (x, y)
    #[builder(default = None)]
    pub bottom_right_corner: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BottomWidgetType {
    Empty,
    Cpu,
    CpuLegend,
    Mem,
    Net,
    Proc,
    ProcSearch,
    ProcSort,
    Temp,
    Disk,
    BasicCpu,
    BasicMem,
    BasicNet,
    BasicTables,
    Battery,
}

impl BottomWidgetType {
    pub fn is_widget_table(&self) -> bool {
        use BottomWidgetType::*;
        matches!(self, Disk | Proc | ProcSort | Temp | CpuLegend)
    }

    pub fn is_widget_graph(&self) -> bool {
        use BottomWidgetType::*;
        matches!(self, Cpu | Net | Mem)
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
            Battery => "Battery",
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
            "mem" | "memory" => Ok(BottomWidgetType::Mem),
            "net" | "network" => Ok(BottomWidgetType::Net),
            "proc" | "process" | "processes" => Ok(BottomWidgetType::Proc),
            "temp" | "temperature" => Ok(BottomWidgetType::Temp),
            "disk" => Ok(BottomWidgetType::Disk),
            "empty" => Ok(BottomWidgetType::Empty),
            "battery" | "batt" => Ok(BottomWidgetType::Battery),
            _ => Err(BottomError::ConfigError(format!(
                "\"{}\" is an invalid widget name.",
                s
            ))),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct UsedWidgets {
    pub use_cpu: bool,
    pub use_mem: bool,
    pub use_net: bool,
    pub use_proc: bool,
    pub use_disk: bool,
    pub use_temp: bool,
    pub use_battery: bool,
}
