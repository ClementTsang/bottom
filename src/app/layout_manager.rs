use std::collections::BTreeMap;

use tui::layout::Constraint;

use crate::{constants::DEFAULT_WIDGET_ID, options::OptionError};

// Represents a start and end coordinate in some dimension.
type LineSegment = (u16, u16);

type WidgetMappings = (u16, BTreeMap<LineSegment, u64>);
type ColumnRowMappings = (u16, BTreeMap<LineSegment, WidgetMappings>);
type ColumnMappings = (u16, BTreeMap<LineSegment, ColumnRowMappings>);

/// Represents a more usable representation of the layout, derived from the
/// config.
///
/// FIXME: This is kinda gross. Ideally optimize out the hard-coded stuff.
#[derive(Clone, Debug)]
pub struct BottomLayout {
    pub rows: Vec<BottomRow>,
    pub total_row_height_ratio: u16,
}

trait Ratio {
    fn ratio(&self) -> u16;
}

impl Ratio for Constraint {
    fn ratio(&self) -> u16 {
        match self {
            Constraint::Min(min) => std::cmp::max(*min, 1),
            Constraint::Length(_) => 1,
            Constraint::Fill(scaling) => *scaling,
            _ => unreachable!("if this gets hit then you're refactoring layouts"),
        }
    }
}

impl BottomLayout {
    pub fn get_movement_mappings(&mut self) {
        #[expect(clippy::suspicious_operation_groupings)] // Have to enable this, clippy really doesn't like me doing this with tuples...
        fn is_intersecting(a: LineSegment, b: LineSegment) -> bool {
            a.0 >= b.0 && a.1 <= b.1
                || a.1 >= b.1 && a.0 <= b.0
                || a.0 <= b.0 && a.1 >= b.0
                || a.0 >= b.0 && a.0 < b.1 && a.1 >= b.1
        }

        fn get_distance(target: LineSegment, candidate: LineSegment) -> u16 {
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
                        let widget_ratio = widget
                            .ratio_override
                            .unwrap_or_else(|| widget.constraint.ratio());

                        match widget.widget_type {
                            BottomWidgetType::Empty => {}
                            _ => {
                                is_valid_col_row = true;
                                col_row_mapping.insert(
                                    (
                                        widget_width * 100 / col_row.total_widget_ratio,
                                        (widget_width + widget_ratio) * 100
                                            / col_row.total_widget_ratio,
                                    ),
                                    widget.widget_id,
                                );
                            }
                        }
                        widget_width += widget_ratio;
                    }
                    if is_valid_col_row {
                        col_mapping.insert(
                            (
                                col_row_height * 100 / col.total_col_row_ratio,
                                (col_row_height + col_row.constraint.ratio()) * 100
                                    / col.total_col_row_ratio,
                            ),
                            (col.total_col_row_ratio, col_row_mapping),
                        );
                        is_valid_col = true;
                    }

                    col_row_height += col_row.constraint.ratio();
                }
                if is_valid_col {
                    row_mapping.insert(
                        (
                            row_width * 100 / row.total_col_ratio,
                            (row_width + col.constraint.ratio()) * 100 / row.total_col_ratio,
                        ),
                        (row.total_col_ratio, col_mapping),
                    );
                    is_valid_row = true;
                }

                row_width += col.constraint.ratio();
            }
            if is_valid_row {
                layout_mapping.insert(
                    (
                        total_height * 100 / self.total_row_height_ratio,
                        (total_height + row.constraint.ratio()) * 100 / self.total_row_height_ratio,
                    ),
                    (self.total_row_height_ratio, row_mapping),
                );
            }
            total_height += row.constraint.ratio();
        }

        // Now pass through a second time; this time we want to build up
        // our neighbour profile.
        let mut height_cursor = 0;
        for row in &mut self.rows {
            let mut col_cursor = 0;
            let row_height_percentage_start = height_cursor * 100 / self.total_row_height_ratio;
            let row_height_percentage_end =
                (height_cursor + row.constraint.ratio()) * 100 / self.total_row_height_ratio;

            for col in &mut row.children {
                let mut col_row_cursor = 0;
                let col_width_percentage_start = col_cursor * 100 / row.total_col_ratio;
                let col_width_percentage_end =
                    (col_cursor + col.constraint.ratio()) * 100 / row.total_col_ratio;

                for col_row in &mut col.children {
                    let mut widget_cursor = 0;
                    let col_row_height_percentage_start =
                        col_row_cursor * 100 / col.total_col_row_ratio;
                    let col_row_height_percentage_end =
                        (col_row_cursor + col_row.constraint.ratio()) * 100
                            / col.total_col_row_ratio;
                    let col_row_children_len = col_row.children.len();

                    for widget in &mut col_row.children {
                        // Bail if empty.
                        if let BottomWidgetType::Empty = widget.widget_type {
                            continue;
                        }

                        let widget_ratio = widget
                            .ratio_override
                            .unwrap_or_else(|| widget.constraint.ratio());

                        let widget_width_percentage_start =
                            widget_cursor * 100 / col_row.total_widget_ratio;
                        let widget_width_percentage_end =
                            (widget_cursor + widget_ratio) * 100 / col_row.total_widget_ratio;

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
                        widget_cursor += widget_ratio;
                    }
                    col_row_cursor += col_row.constraint.ratio();
                }
                col_cursor += col.constraint.ratio();
            }
            height_cursor += row.constraint.ratio();
        }
    }

    pub fn init_basic_default(use_battery: bool) -> Self {
        let table_widgets = if use_battery {
            let disk_widget = BottomWidget::new(BottomWidgetType::Disk, 4)
                .canvas_handled()
                .up_neighbour(Some(100))
                .left_neighbour(Some(8))
                .right_neighbour(Some(DEFAULT_WIDGET_ID + 2));

            let proc_sort = BottomWidget::new(BottomWidgetType::ProcSort, DEFAULT_WIDGET_ID + 2)
                .canvas_handled()
                .up_neighbour(Some(100))
                .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                .left_neighbour(Some(4))
                .right_neighbour(Some(DEFAULT_WIDGET_ID))
                .ratio(1)
                .parent_reflector(Some((WidgetDirection::Right, 2)));

            let proc = BottomWidget::new(BottomWidgetType::Proc, DEFAULT_WIDGET_ID)
                .canvas_handled()
                .up_neighbour(Some(100))
                .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                .left_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                .right_neighbour(Some(7))
                .ratio(2);

            let proc_search =
                BottomWidget::new(BottomWidgetType::ProcSearch, DEFAULT_WIDGET_ID + 1)
                    .canvas_handled()
                    .up_neighbour(Some(DEFAULT_WIDGET_ID))
                    .left_neighbour(Some(4))
                    .right_neighbour(Some(7))
                    .parent_reflector(Some((WidgetDirection::Up, 1)));

            let temp = BottomWidget::new(BottomWidgetType::Temp, 7)
                .canvas_handled()
                .up_neighbour(Some(100))
                .left_neighbour(Some(DEFAULT_WIDGET_ID))
                .right_neighbour(Some(8));

            let battery = BottomWidget::new(BottomWidgetType::Battery, 8)
                .canvas_handled()
                .up_neighbour(Some(100))
                .left_neighbour(Some(7))
                .right_neighbour(Some(4));

            vec![
                BottomCol::new(vec![BottomColRow::new(vec![disk_widget]).canvas_handled()])
                    .canvas_handled(),
                BottomCol::new(vec![
                    BottomColRow::new(vec![proc_sort, proc])
                        .canvas_handled()
                        .total_widget_ratio(3),
                    BottomColRow::new(vec![proc_search]).canvas_handled(),
                ])
                .canvas_handled(),
                BottomCol::new(vec![BottomColRow::new(vec![temp]).canvas_handled()])
                    .canvas_handled(),
                BottomCol::new(vec![BottomColRow::new(vec![battery]).canvas_handled()])
                    .canvas_handled(),
            ]
        } else {
            let disk = BottomWidget::new(BottomWidgetType::Disk, 4)
                .canvas_handled()
                .up_neighbour(Some(100))
                .left_neighbour(Some(7))
                .right_neighbour(Some(DEFAULT_WIDGET_ID + 2));

            let proc_sort = BottomWidget::new(BottomWidgetType::ProcSort, DEFAULT_WIDGET_ID + 2)
                .canvas_handled()
                .up_neighbour(Some(100))
                .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                .left_neighbour(Some(4))
                .right_neighbour(Some(DEFAULT_WIDGET_ID))
                .parent_reflector(Some((WidgetDirection::Right, 2)));

            let proc = BottomWidget::new(BottomWidgetType::Proc, DEFAULT_WIDGET_ID)
                .canvas_handled()
                .up_neighbour(Some(100))
                .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                .left_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                .right_neighbour(Some(7));

            let proc_search =
                BottomWidget::new(BottomWidgetType::ProcSearch, DEFAULT_WIDGET_ID + 1)
                    .canvas_handled()
                    .up_neighbour(Some(DEFAULT_WIDGET_ID))
                    .left_neighbour(Some(4))
                    .right_neighbour(Some(7))
                    .parent_reflector(Some((WidgetDirection::Up, 1)));

            let temp = BottomWidget::new(BottomWidgetType::Temp, 7)
                .canvas_handled()
                .up_neighbour(Some(100))
                .left_neighbour(Some(DEFAULT_WIDGET_ID))
                .right_neighbour(Some(4));

            vec![
                BottomCol::new(vec![BottomColRow::new(vec![disk]).canvas_handled()])
                    .canvas_handled(),
                BottomCol::new(vec![
                    BottomColRow::new(vec![proc_sort, proc]).canvas_handled(),
                    BottomColRow::new(vec![proc_search]).canvas_handled(),
                ])
                .canvas_handled(),
                BottomCol::new(vec![BottomColRow::new(vec![temp]).canvas_handled()])
                    .canvas_handled(),
            ]
        };

        let cpu = BottomWidget::new(BottomWidgetType::BasicCpu, 1)
            .canvas_handled()
            .down_neighbour(Some(2));

        let mem = BottomWidget::new(BottomWidgetType::BasicMem, 2)
            .canvas_handled()
            .up_neighbour(Some(1))
            .down_neighbour(Some(100))
            .right_neighbour(Some(3));

        let net = BottomWidget::new(BottomWidgetType::BasicNet, 3)
            .canvas_handled()
            .up_neighbour(Some(1))
            .down_neighbour(Some(100))
            .left_neighbour(Some(2));

        let table = BottomWidget::new(BottomWidgetType::BasicTables, 100)
            .canvas_handled()
            .up_neighbour(Some(2));

        BottomLayout {
            total_row_height_ratio: 3,
            rows: vec![
                BottomRow::new(vec![
                    BottomCol::new(vec![BottomColRow::new(vec![cpu]).canvas_handled()])
                        .canvas_handled(),
                ])
                .canvas_handled(),
                BottomRow::new(vec![
                    BottomCol::new(vec![BottomColRow::new(vec![mem, net]).canvas_handled()])
                        .canvas_handled(),
                ])
                .canvas_handled(),
                BottomRow::new(vec![
                    BottomCol::new(vec![BottomColRow::new(vec![table]).canvas_handled()])
                        .canvas_handled(),
                ])
                .canvas_handled(),
                BottomRow::new(table_widgets).canvas_handled(),
            ],
        }
    }
}

/// Represents a single row in the layout.
#[derive(Clone, Debug)]
pub struct BottomRow {
    pub children: Vec<BottomCol>,
    pub total_col_ratio: u16,
    pub constraint: Constraint,
}

impl BottomRow {
    pub fn new(children: Vec<BottomCol>) -> Self {
        Self {
            children,
            total_col_ratio: 1,
            constraint: Constraint::Fill(1),
        }
    }

    pub fn total_col_ratio(mut self, total_col_ratio: u16) -> Self {
        self.total_col_ratio = total_col_ratio;
        self
    }

    pub fn ratio(mut self, value: u16) -> Self {
        self.constraint = Constraint::Fill(value);
        self
    }

    pub fn canvas_handled(mut self) -> Self {
        self.constraint = Constraint::Length(0);
        self
    }
}

/// Represents a single column in the layout.  We assume that even if the column
/// contains only ONE element, it is still a column (rather than either a col or
/// a widget, as per the config, for simplicity's sake).
#[derive(Clone, Debug)]
pub struct BottomCol {
    pub children: Vec<BottomColRow>,
    pub total_col_row_ratio: u16,
    pub constraint: Constraint,
}

impl BottomCol {
    pub fn new(children: Vec<BottomColRow>) -> Self {
        Self {
            children,
            total_col_row_ratio: 1,
            constraint: Constraint::Fill(1),
        }
    }

    pub fn total_col_row_ratio(mut self, total_col_row_ratio: u16) -> Self {
        self.total_col_row_ratio = total_col_row_ratio;
        self
    }

    pub fn ratio(mut self, value: u16) -> Self {
        self.constraint = Constraint::Fill(value);
        self
    }

    pub fn canvas_handled(mut self) -> Self {
        self.constraint = Constraint::Length(0);
        self
    }
}

#[derive(Clone, Default, Debug)]
pub struct BottomColRow {
    pub children: Vec<BottomWidget>,
    pub total_widget_ratio: u16,
    pub constraint: Constraint,
}

impl BottomColRow {
    pub(crate) fn new(children: Vec<BottomWidget>) -> Self {
        Self {
            children,
            total_widget_ratio: 1,
            constraint: Constraint::Fill(1),
        }
    }

    pub(crate) fn total_widget_ratio(mut self, total_widget_ratio: u16) -> Self {
        self.total_widget_ratio = total_widget_ratio;
        self
    }

    pub fn ratio(mut self, value: u16) -> Self {
        self.constraint = Constraint::Fill(value);
        self
    }

    pub fn canvas_handled(mut self) -> Self {
        self.constraint = Constraint::Length(0);
        self
    }

    pub fn grow(mut self, minimum: Option<u16>) -> Self {
        self.constraint = Constraint::Min(minimum.unwrap_or(0));
        self
    }
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
        let to_compare = match &self {
            WidgetDirection::Left => WidgetDirection::Right,
            WidgetDirection::Right => WidgetDirection::Left,
            WidgetDirection::Up => WidgetDirection::Down,
            WidgetDirection::Down => WidgetDirection::Up,
        };

        *other_direction == to_compare
    }
}

/// Represents a single widget.
#[derive(Debug, Default, Clone)]
pub struct BottomWidget {
    pub widget_type: BottomWidgetType,
    pub widget_id: u64,
    pub constraint: Constraint,
    pub left_neighbour: Option<u64>,
    pub right_neighbour: Option<u64>,
    pub up_neighbour: Option<u64>,
    pub down_neighbour: Option<u64>,

    /// The value is the direction to bounce, as well as the parent offset.
    pub parent_reflector: Option<(WidgetDirection, u64)>,

    /// Top left corner when drawn, for mouse click detection. (x, y)
    ///
    /// TODO: Replace this with just an Option<Rect> for top + bottom.
    pub top_left_corner: Option<(u16, u16)>,

    /// Bottom right corner when drawn, for mouse click detection. (x, y)
    pub bottom_right_corner: Option<(u16, u16)>,

    /// TODO: REMOVE THIS LATER. This is temporary code to bridge the
    /// old layout system with a newer system later.
    ratio_override: Option<u16>,
}

impl BottomWidget {
    pub(crate) fn new(widget_type: BottomWidgetType, widget_id: u64) -> Self {
        Self {
            widget_type,
            widget_id,
            constraint: Constraint::Fill(1),
            left_neighbour: None,
            right_neighbour: None,
            up_neighbour: None,
            down_neighbour: None,
            parent_reflector: None,
            top_left_corner: None,
            bottom_right_corner: None,
            ratio_override: None,
        }
    }

    pub(crate) fn left_neighbour(mut self, left_neighbour: Option<u64>) -> Self {
        self.left_neighbour = left_neighbour;
        self
    }

    pub(crate) fn right_neighbour(mut self, right_neighbour: Option<u64>) -> Self {
        self.right_neighbour = right_neighbour;
        self
    }

    pub(crate) fn up_neighbour(mut self, up_neighbour: Option<u64>) -> Self {
        self.up_neighbour = up_neighbour;
        self
    }

    pub(crate) fn down_neighbour(mut self, down_neighbour: Option<u64>) -> Self {
        self.down_neighbour = down_neighbour;
        self
    }

    pub(crate) fn ratio(mut self, value: u16) -> Self {
        self.constraint = Constraint::Fill(value);
        self
    }

    pub fn canvas_handled(mut self) -> Self {
        self.constraint = Constraint::Length(0);
        self
    }

    pub fn grow(mut self, minimum: Option<u16>) -> Self {
        self.constraint = Constraint::Min(minimum.unwrap_or(0));
        self
    }

    /// TODO: REMOVE THIS LATER. This is temporary code to bridge the
    /// old layout system with a newer system later.
    pub fn with_ratio_override(mut self, ratio_override: u16) -> Self {
        self.ratio_override = Some(ratio_override);
        self
    }

    pub(crate) fn parent_reflector(
        mut self, parent_reflector: Option<(WidgetDirection, u64)>,
    ) -> Self {
        self.parent_reflector = parent_reflector;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum BottomWidgetType {
    #[default]
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
    Gpu,
}

impl BottomWidgetType {
    pub fn is_widget_table(&self) -> bool {
        use BottomWidgetType::*;
        matches!(self, Disk | Proc | ProcSort | Temp | CpuLegend)
    }

    pub fn is_widget_graph(&self) -> bool {
        use BottomWidgetType::*;
        matches!(self, Cpu | Net | Mem | Gpu)
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
            Gpu => "GPU",
            _ => "",
        }
    }
}

impl std::str::FromStr for BottomWidgetType {
    type Err = OptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_lowercase();
        match lower_case.as_str() {
            "cpu" => Ok(BottomWidgetType::Cpu),
            "mem" | "memory" => Ok(BottomWidgetType::Mem),
            "net" | "network" => Ok(BottomWidgetType::Net),
            "proc" | "process" | "processes" => Ok(BottomWidgetType::Proc),
            "temp" | "temperature" => Ok(BottomWidgetType::Temp),
            "disk" => Ok(BottomWidgetType::Disk),
            "empty" => Ok(BottomWidgetType::Empty),
            #[cfg(feature = "battery")]
            "battery" | "batt" => Ok(BottomWidgetType::Battery),
            "gpu" => Ok(BottomWidgetType::Gpu),
            _ => {
                #[cfg(feature = "battery")]
                {
                    Err(OptionError::config(format!(
                        "'{s}' is an invalid widget name.
        
Supported widget names:
+--------------------------+
|            cpu           |
+--------------------------+
|        mem, memory       |
+--------------------------+
|       net, network       |
+--------------------------+
| proc, process, processes |
+--------------------------+
|     temp, temperature    |
+--------------------------+
|           disk           |
+--------------------------+
|       batt, battery      |
+--------------------------+
|            gpu           |
+--------------------------+
|           empty          |
+--------------------------+
                ",
                    )))
                }
                #[cfg(not(feature = "battery"))]
                {
                    Err(OptionError::config(format!(
                        "'{s}' is an invalid widget name.

Supported widget names:
+--------------------------+
|            cpu           |
+--------------------------+
|        mem, memory       |
+--------------------------+
|       net, network       |
+--------------------------+
| proc, process, processes |
+--------------------------+
|     temp, temperature    |
+--------------------------+
|           disk           |
+--------------------------+
|            gpu           |
+--------------------------+
|           empty          |
+--------------------------+
                ",
                    )))
                }
            }
        }
    }
}

#[derive(Clone, Default, Debug, Copy)]
pub struct UsedWidgets {
    pub use_cpu: bool,
    pub use_mem: bool,
    pub use_cache: bool,
    pub use_gpu: bool,
    pub use_net: bool,
    pub use_proc: bool,
    pub use_disk: bool,
    pub use_temp: bool,
    pub use_battery: bool,
}
