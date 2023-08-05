use std::collections::BTreeMap;

use tui::layout::Direction;

use crate::canvas::LayoutConstraint;
use crate::constants::DEFAULT_WIDGET_ID;
use crate::error::{BottomError, Result};
use crate::options::layout_options::{Row, RowChildren};
use crate::utils::error;

/// Represents a start and end coordinate in some dimension.
type LineSegment = (u32, u32);

type WidgetMappings = (u32, BTreeMap<LineSegment, u64>);
type ColumnRowMappings = (u32, BTreeMap<LineSegment, WidgetMappings>);
type ColumnMappings = (u32, BTreeMap<LineSegment, ColumnRowMappings>);

/// A "container" that contains more child elements, stored as [`NodeId`]s.
#[derive(Debug, Clone)]
pub(crate) struct Container {
    /// The children elements.
    children: Vec<NodeId>,

    /// How the container should be sized.
    constraint: LayoutConstraint,

    /// The direction.
    direction: ContainerDirection,
}

impl Container {
    pub(crate) fn row(children: Vec<NodeId>, sizing: LayoutConstraint) -> Self {
        Self {
            children,
            constraint: sizing,
            direction: ContainerDirection::Row,
        }
    }

    pub(crate) fn col(children: Vec<NodeId>, constraint: LayoutConstraint) -> Self {
        Self {
            children,
            constraint,
            direction: ContainerDirection::Col,
        }
    }

    /// Returns the constraint of the container.
    #[inline]
    pub fn constraint(&self) -> LayoutConstraint {
        self.constraint
    }

    /// Returns a reference to the children.
    #[inline]
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    /// Returns the direction of the container.
    #[inline]
    pub fn direction(&self) -> ContainerDirection {
        self.direction
    }
}

/// The direction in which children in a [`BottomContainer`] will be laid out.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ContainerDirection {
    /// Lay out all children horizontally.
    Row,

    /// Lay out all children vertically.
    Col,
}

impl From<ContainerDirection> for Direction {
    fn from(value: ContainerDirection) -> Self {
        match value {
            ContainerDirection::Row => Direction::Horizontal,
            ContainerDirection::Col => Direction::Vertical,
        }
    }
}

/// An ID for a node in a [`BottomLayout`].
#[derive(Clone, Copy, Debug)]
pub enum NodeId {
    /// The ID for a [`Container`].
    Container(usize),

    /// The ID for a [`BottomWidget`].
    Widget(usize),
}

fn new_cpu(
    layout: &mut BottomLayout, left_legend: bool, iter_id: &mut u64, width: u32, total: u32,
) -> NodeId {
    let cpu_id = *iter_id;
    *iter_id += 1;
    let legend_id = *iter_id;

    if left_legend {
        let cpu_legend = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::CpuLegend, legend_id)
                .parent_reflector(Some((WidgetDirection::Right, 1))),
        );
        let cpu = layout.add_widget(BottomWidget::new_handled(BottomWidgetType::Cpu, cpu_id));

        layout.add_container(Container::row(
            vec![cpu_legend, cpu],
            LayoutConstraint::Ratio { a: width, b: total },
        ))
    } else {
        let cpu = layout.add_widget(BottomWidget::new_handled(BottomWidgetType::Cpu, cpu_id));
        let cpu_legend = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::CpuLegend, legend_id)
                .parent_reflector(Some((WidgetDirection::Left, 1))),
        );

        layout.add_container(Container::row(
            vec![cpu, cpu_legend],
            LayoutConstraint::Ratio { a: width, b: total },
        ))
    }
}

fn new_proc(layout: &mut BottomLayout, iter_id: &mut u64, width: u32, total: u32) -> NodeId {
    let main_id = *iter_id;
    let search_id = *iter_id + 1;
    *iter_id += 2;
    let sort_id = *iter_id;

    let main = layout.add_widget(BottomWidget::new_fill(BottomWidgetType::Proc, main_id));

    let search = layout.add_widget(
        BottomWidget::new_fill(BottomWidgetType::ProcSearch, search_id)
            .parent_reflector(Some((WidgetDirection::Up, 1))),
    );

    let sort = layout.add_widget(
        BottomWidget::new_handled(BottomWidgetType::ProcSort, sort_id)
            .parent_reflector(Some((WidgetDirection::Right, 2))),
    );

    let top = layout.add_container(Container::row(
        vec![sort, main],
        LayoutConstraint::CanvasHandled,
    ));

    layout.add_container(Container::col(
        vec![top, search],
        LayoutConstraint::Ratio { a: width, b: total },
    ))
}

fn new_widget(
    layout: &mut BottomLayout, widget_type: BottomWidgetType, iter_id: &mut u64, width: u32,
    total_ratio: u32, left_legend: bool,
) -> NodeId {
    *iter_id += 1;

    match widget_type {
        BottomWidgetType::Cpu => new_cpu(layout, left_legend, iter_id, width, total_ratio),
        BottomWidgetType::Proc => new_proc(layout, iter_id, width, total_ratio),
        _ => layout.add_widget(BottomWidget::new(
            widget_type,
            *iter_id,
            LayoutConstraint::Ratio {
                a: width,
                b: total_ratio,
            },
        )),
    }
}

/// Represents a more usable representation of the layout, derived from the
/// config.
///
/// Internally represented by an arena-backed tree.
#[derive(Clone, Debug, Default)]
pub struct BottomLayout {
    containers: Vec<Container>,
    widgets: Vec<BottomWidget>,
}

impl BottomLayout {
    /// Add a container to the layout arena. The ID is returned.
    pub fn add_container(&mut self, container: Container) -> NodeId {
        let id = self.containers.len();
        self.containers.push(container);

        NodeId::Container(id)
    }

    /// Add a node to the layout arena. The ID is returned.
    pub fn add_widget(&mut self, widget: BottomWidget) -> NodeId {
        let id = self.widgets.len();
        self.widgets.push(widget);

        NodeId::Widget(id)
    }

    /// Get a reference to the [`Container`] with the corresponding ID if
    /// it exists.
    pub fn get_container(&self, id: usize) -> Option<&Container> {
        self.containers.get(id)
    }

    /// Get a reference to the [`BottomWidget`] with the corresponding ID if
    /// it exists.
    pub fn get_widget(&self, id: usize) -> Option<&BottomWidget> {
        self.widgets.get(id)
    }

    /// Get a mutable reference to the [`BottomWidget`] with the corresponding
    /// ID if it exists.
    pub fn get_widget_mut(&mut self, id: usize) -> Option<&mut BottomWidget> {
        self.widgets.get_mut(id)
    }

    /// Returns an iterator of all widgets.
    pub fn widgets_iter(&self) -> impl Iterator<Item = &BottomWidget> {
        self.widgets.iter()
    }

    /// Returns the root ID if there is one. If there are no nodes, it will return [`None`].
    pub fn root_id(&self) -> Option<NodeId> {
        if self.containers.is_empty() {
            if self.widgets.is_empty() {
                None
            } else {
                Some(NodeId::Widget(self.widgets.len() - 1))
            }
        } else {
            Some(NodeId::Container(self.containers.len() - 1))
        }
    }

    /// Returns the number of elements (widgets + containers) in the layout.
    pub fn len(&self) -> usize {
        self.widgets.len() + self.containers.len()
    }

    /// Returns the number of widgets in the layout.
    pub fn widgets_len(&self) -> usize {
        self.widgets.len()
    }

    /// Creates a new [`BottomLayout`] given a slice of [`Row`]s, as well as the default widget ID.
    pub fn from_rows(
        rows: &[Row], default_widget_type: Option<BottomWidgetType>, mut default_widget_count: u64,
        left_legend: bool,
    ) -> error::Result<(Self, u64)> {
        let mut layout = Self::default();
        let mut default_widget_id = 1;
        let mut iter_id = 0; // TODO: In the future, remove this in favour of using the layout's ID system.

        let outer_col_total_ratio = rows.iter().map(|row| row.ratio.unwrap_or(1)).sum();
        let mut outer_col_children = Vec::with_capacity(rows.len());

        for row in rows {
            // This code is all ported from the old row-to-bottom_row code, and converted
            // to work with our new system.

            // TODO: In the future we want to also add percentages.
            // But for MVP, we aren't going to bother.

            let row_ratio = row.ratio.unwrap_or(1);

            if let Some(children) = &row.child {
                let mut row_children = Vec::with_capacity(children.len());

                let rows_total_ratio = children
                    .iter()
                    .map(|c| match c {
                        RowChildren::Widget(w) => w.ratio.unwrap_or(1),
                        RowChildren::Col { ratio, .. } => ratio.unwrap_or(1),
                    })
                    .sum();

                for child in children {
                    match child {
                        RowChildren::Widget(widget) => {
                            let width = widget.ratio.unwrap_or(1);
                            let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;

                            if let Some(default_widget_type_val) = default_widget_type {
                                if default_widget_type_val == widget_type
                                    && default_widget_count > 0
                                {
                                    default_widget_count -= 1;
                                    if default_widget_count == 0 {
                                        default_widget_id = iter_id;
                                    }
                                }
                            } else {
                                // Check default flag
                                if let Some(default_widget_flag) = widget.default {
                                    if default_widget_flag {
                                        default_widget_id = iter_id;
                                    }
                                }
                            }

                            let widget = new_widget(
                                &mut layout,
                                widget_type,
                                &mut iter_id,
                                width,
                                rows_total_ratio,
                                left_legend,
                            );

                            row_children.push(widget);
                        }
                        RowChildren::Col {
                            ratio,
                            child: children,
                        } => {
                            let col_ratio = ratio.unwrap_or(1);
                            let mut col_children = vec![];

                            let inner_col_total_ratio =
                                children.iter().map(|w| w.ratio.unwrap_or(1)).sum();
                            for widget in children {
                                let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;
                                let height = widget.ratio.unwrap_or(1);

                                if let Some(default_widget_type_val) = default_widget_type {
                                    if default_widget_type_val == widget_type
                                        && default_widget_count > 0
                                    {
                                        default_widget_count -= 1;
                                        if default_widget_count == 0 {
                                            default_widget_id = iter_id;
                                        }
                                    }
                                } else {
                                    // Check default flag
                                    if let Some(default_widget_flag) = widget.default {
                                        if default_widget_flag {
                                            default_widget_id = iter_id;
                                        }
                                    }
                                }

                                let widget = new_widget(
                                    &mut layout,
                                    widget_type,
                                    &mut iter_id,
                                    height,
                                    inner_col_total_ratio,
                                    left_legend,
                                );

                                col_children.push(widget);
                            }

                            row_children.push(layout.add_container(Container::col(
                                col_children,
                                LayoutConstraint::Ratio {
                                    a: col_ratio,
                                    b: rows_total_ratio,
                                },
                            )));
                        }
                    }
                }

                outer_col_children.push(layout.add_container(Container::row(
                    row_children,
                    LayoutConstraint::Ratio {
                        a: row_ratio,
                        b: outer_col_total_ratio,
                    },
                )));
            };
        }

        layout.add_container(Container::col(
            outer_col_children,
            LayoutConstraint::FlexGrow,
        ));

        if layout.widgets_len() > 0 {
            layout.get_movement_mappings();
            Ok((layout, default_widget_id))
        } else {
            Err(error::BottomError::ConfigError(
                "please have at least one widget under the '[[row]]' section.".to_string(),
            ))
        }
    }

    /// Creates a new [`BottomLayout`] following the basic layout.
    pub fn new_basic(use_battery: bool) -> Self {
        let mut layout = BottomLayout::default();

        let table_widgets = if use_battery {
            let disk = layout.add_widget(
                BottomWidget::new_handled(BottomWidgetType::Disk, 4)
                    .up_neighbour(Some(100))
                    .left_neighbour(Some(8))
                    .right_neighbour(Some(DEFAULT_WIDGET_ID + 2)),
            );

            let proc = {
                let proc_sort = layout.add_widget(
                    BottomWidget::new_handled(BottomWidgetType::ProcSort, DEFAULT_WIDGET_ID + 2)
                        .up_neighbour(Some(100))
                        .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                        .left_neighbour(Some(4))
                        .right_neighbour(Some(DEFAULT_WIDGET_ID))
                        .parent_reflector(Some((WidgetDirection::Right, 2))),
                );

                let main_proc = layout.add_widget(
                    BottomWidget::new_handled(BottomWidgetType::Proc, DEFAULT_WIDGET_ID)
                        .up_neighbour(Some(100))
                        .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                        .left_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                        .right_neighbour(Some(7)),
                );

                let proc_search = layout.add_widget(
                    BottomWidget::new_handled(BottomWidgetType::ProcSearch, DEFAULT_WIDGET_ID + 1)
                        .up_neighbour(Some(DEFAULT_WIDGET_ID))
                        .left_neighbour(Some(4))
                        .right_neighbour(Some(7))
                        .parent_reflector(Some((WidgetDirection::Up, 1))),
                );

                let top = layout.add_container(Container::row(
                    vec![proc_sort, main_proc],
                    LayoutConstraint::CanvasHandled,
                ));
                layout.add_container(Container::col(
                    vec![top, proc_search],
                    LayoutConstraint::CanvasHandled,
                ))
            };

            let temp = layout.add_widget(
                BottomWidget::new_handled(BottomWidgetType::Temp, 7)
                    .up_neighbour(Some(100))
                    .left_neighbour(Some(DEFAULT_WIDGET_ID))
                    .right_neighbour(Some(8)),
            );

            let battery = layout.add_widget(
                BottomWidget::new_handled(BottomWidgetType::Battery, 8)
                    .up_neighbour(Some(100))
                    .left_neighbour(Some(7))
                    .right_neighbour(Some(4)),
            );

            layout.add_container(Container::row(
                vec![disk, proc, temp, battery],
                LayoutConstraint::CanvasHandled,
            ))
        } else {
            let disk = layout.add_widget(
                BottomWidget::new_handled(BottomWidgetType::Disk, 4)
                    .up_neighbour(Some(100))
                    .left_neighbour(Some(7))
                    .right_neighbour(Some(DEFAULT_WIDGET_ID + 2)),
            );

            let proc = {
                let proc_sort = layout.add_widget(
                    BottomWidget::new_handled(BottomWidgetType::ProcSort, DEFAULT_WIDGET_ID + 2)
                        .up_neighbour(Some(100))
                        .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                        .left_neighbour(Some(4))
                        .right_neighbour(Some(DEFAULT_WIDGET_ID))
                        .parent_reflector(Some((WidgetDirection::Right, 2))),
                );

                let main_proc = layout.add_widget(
                    BottomWidget::new_handled(BottomWidgetType::Proc, DEFAULT_WIDGET_ID)
                        .up_neighbour(Some(100))
                        .down_neighbour(Some(DEFAULT_WIDGET_ID + 1))
                        .left_neighbour(Some(DEFAULT_WIDGET_ID + 2))
                        .right_neighbour(Some(7)),
                );

                let proc_search = layout.add_widget(
                    BottomWidget::new_handled(BottomWidgetType::ProcSearch, DEFAULT_WIDGET_ID + 1)
                        .up_neighbour(Some(DEFAULT_WIDGET_ID))
                        .left_neighbour(Some(4))
                        .right_neighbour(Some(7))
                        .parent_reflector(Some((WidgetDirection::Up, 1))),
                );

                let top = layout.add_container(Container::row(
                    vec![proc_sort, main_proc],
                    LayoutConstraint::CanvasHandled,
                ));
                layout.add_container(Container::col(
                    vec![top, proc_search],
                    LayoutConstraint::CanvasHandled,
                ))
            };

            let temp = layout.add_widget(
                BottomWidget::new_handled(BottomWidgetType::Temp, 7)
                    .up_neighbour(Some(100))
                    .left_neighbour(Some(DEFAULT_WIDGET_ID))
                    .right_neighbour(Some(4)),
            );

            layout.add_container(Container::row(
                vec![disk, proc, temp],
                LayoutConstraint::CanvasHandled,
            ))
        };

        let cpu = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::BasicCpu, 1).down_neighbour(Some(2)),
        );

        let mem = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::BasicMem, 2)
                .up_neighbour(Some(1))
                .down_neighbour(Some(100))
                .right_neighbour(Some(3)),
        );

        let net = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::BasicNet, 3)
                .up_neighbour(Some(1))
                .down_neighbour(Some(100))
                .left_neighbour(Some(2)),
        );

        let net = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::BasicTables, 100).up_neighbour(Some(2)),
        );

        let middle_bars = layout.add_container(Container::row(
            vec![mem, net],
            LayoutConstraint::CanvasHandled,
        ));

        let table = layout.add_widget(
            BottomWidget::new_handled(BottomWidgetType::BasicTables, 100).up_neighbour(Some(2)),
        );

        layout.add_container(Container::col(
            vec![cpu, middle_bars, table, table_widgets],
            LayoutConstraint::CanvasHandled,
        ));

        layout
    }

    /// Creates mappings to move from one widget to another.
    fn get_movement_mappings(&mut self) {
        type LineSegment = (u32, u32);

        // Have to enable this, clippy really doesn't like me doing this with tuples...
        #[allow(clippy::suspicious_operation_groupings)]
        fn is_intersecting(a: LineSegment, b: LineSegment) -> bool {
            a.0 >= b.0 && a.1 <= b.1
                || a.1 >= b.1 && a.0 <= b.0
                || a.0 <= b.0 && a.1 >= b.0
                || a.0 >= b.0 && a.0 < b.1 && a.1 >= b.1
        }

        fn distance(target: LineSegment, candidate: LineSegment) -> u32 {
            if candidate.0 < target.0 {
                candidate.1 - target.0
            } else if candidate.1 < target.1 {
                candidate.1 - candidate.0
            } else {
                target.1 - candidate.0
            }
        }

        if let Some(root_id) = self.root_id() {
            let mut queue = vec![root_id];

            // Build a k-d tree to have a simple virtual mapping of where each
            // widget is relative to each other.
            while let Some(current) = queue.pop() {
                match current {
                    NodeId::Container(id) => if let Some(children) = self.get_container(id) {},
                    NodeId::Widget(id) => if let Some(widget) = self.get_widget(id) {},
                }
            }

            // Now traverse the layout tree a second time, assigning any missing
            // widget mappings where it makes sense.
            queue.push(root_id);
            while let Some(current) = queue.pop() {
                match current {
                    NodeId::Container(id) => if let Some(children) = self.get_container(id) {},
                    NodeId::Widget(id) => {
                        if let Some(widget) = self.get_widget_mut(id) {
                            if widget.left_neighbour.is_none() {}

                            if widget.right_neighbour.is_none() {}

                            if widget.up_neighbour.is_none() {}

                            if widget.down_neighbour.is_none() {}
                        }
                    }
                }
            }
        }
    }

    fn old_get_movement_mappings(&mut self) {
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
        // widget to another.
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
}

/// Represents a single row in the layout.
#[derive(Clone, Debug)]
pub struct BottomRow {
    pub children: Vec<BottomCol>,
    pub total_col_ratio: u32,
    pub row_height_ratio: u32,
    pub canvas_handle_height: bool,
    pub flex_grow: bool,
}

impl BottomRow {
    pub fn new(children: Vec<BottomCol>) -> Self {
        Self {
            children,
            total_col_ratio: 1,
            row_height_ratio: 1,
            canvas_handle_height: false,
            flex_grow: false,
        }
    }

    pub fn total_col_ratio(mut self, total_col_ratio: u32) -> Self {
        self.total_col_ratio = total_col_ratio;
        self
    }

    pub fn row_height_ratio(mut self, row_height_ratio: u32) -> Self {
        self.row_height_ratio = row_height_ratio;
        self
    }

    pub fn canvas_handle_height(mut self, canvas_handle_height: bool) -> Self {
        self.canvas_handle_height = canvas_handle_height;
        self
    }

    pub fn flex_grow(mut self, flex_grow: bool) -> Self {
        self.flex_grow = flex_grow;
        self
    }
}

/// Represents a single column in the layout.  We assume that even if the column
/// contains only ONE element, it is still a column (rather than either a col or
/// a widget, as per the config, for simplicity's sake).
#[derive(Clone, Debug)]
pub struct BottomCol {
    pub children: Vec<BottomColRow>,
    pub total_col_row_ratio: u32,
    pub col_width_ratio: u32,
    pub canvas_handle_width: bool,
    pub flex_grow: bool,
}

impl BottomCol {
    pub fn new(children: Vec<BottomColRow>) -> Self {
        Self {
            children,
            total_col_row_ratio: 1,
            col_width_ratio: 1,
            canvas_handle_width: false,
            flex_grow: false,
        }
    }

    pub fn total_col_row_ratio(mut self, total_col_row_ratio: u32) -> Self {
        self.total_col_row_ratio = total_col_row_ratio;
        self
    }

    pub fn col_width_ratio(mut self, col_width_ratio: u32) -> Self {
        self.col_width_ratio = col_width_ratio;
        self
    }

    pub fn canvas_handle_width(mut self, canvas_handle_width: bool) -> Self {
        self.canvas_handle_width = canvas_handle_width;
        self
    }

    pub fn flex_grow(mut self, flex_grow: bool) -> Self {
        self.flex_grow = flex_grow;
        self
    }
}

#[derive(Clone, Debug)]
pub struct BottomColRow {
    pub children: Vec<BottomWidget>,
    pub total_widget_ratio: u32,
    pub col_row_height_ratio: u32,
    pub canvas_handle_height: bool,
    pub flex_grow: bool,
}

impl BottomColRow {
    pub(crate) fn new(children: Vec<BottomWidget>) -> Self {
        Self {
            children,
            total_widget_ratio: 1,
            col_row_height_ratio: 1,
            canvas_handle_height: false,
            flex_grow: false,
        }
    }

    pub(crate) fn total_widget_ratio(mut self, total_widget_ratio: u32) -> Self {
        self.total_widget_ratio = total_widget_ratio;
        self
    }

    pub(crate) fn col_row_height_ratio(mut self, col_row_height_ratio: u32) -> Self {
        self.col_row_height_ratio = col_row_height_ratio;
        self
    }

    pub(crate) fn canvas_handle_height(mut self, canvas_handle_height: bool) -> Self {
        self.canvas_handle_height = canvas_handle_height;
        self
    }

    pub(crate) fn flex_grow(mut self, flex_grow: bool) -> Self {
        self.flex_grow = flex_grow;
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
#[derive(Debug, Clone)]
pub struct BottomWidget {
    /// The widget "type".
    pub widget_type: BottomWidgetType,

    /// The ID of this widget.
    pub widget_id: u64,

    /// How the widget should be sized by the canvas.
    pub constraint: LayoutConstraint,

    /// The widget ID to go to when moving to the left.
    pub left_neighbour: Option<u64>,

    /// The widget ID to go to when moving to the right.
    pub right_neighbour: Option<u64>,

    /// The widget ID to go to when moving up.
    pub up_neighbour: Option<u64>,

    /// The widget ID to go to when moving down.
    pub down_neighbour: Option<u64>,

    /// The value is the direction to bounce, as well as the parent offset.
    pub parent_reflector: Option<(WidgetDirection, u64)>,

    /// Top left corner when drawn, for mouse click detection. (x, y)
    pub top_left_corner: Option<(u16, u16)>,

    /// Bottom right corner when drawn, for mouse click detection. (x, y)
    pub bottom_right_corner: Option<(u16, u16)>,
}

impl BottomWidget {
    pub(crate) fn new(
        widget_type: BottomWidgetType, widget_id: u64, constraint: LayoutConstraint,
    ) -> Self {
        Self {
            widget_type,
            widget_id,
            constraint,
            left_neighbour: None,
            right_neighbour: None,
            up_neighbour: None,
            down_neighbour: None,
            parent_reflector: None,
            top_left_corner: None,
            bottom_right_corner: None,
        }
    }

    pub(crate) fn new_fill(widget_type: BottomWidgetType, widget_id: u64) -> Self {
        Self::new(
            widget_type,
            widget_id,
            LayoutConstraint::Ratio { a: 1, b: 1 },
        )
    }

    pub(crate) fn new_handled(widget_type: BottomWidgetType, widget_id: u64) -> Self {
        Self::new(widget_type, widget_id, LayoutConstraint::CanvasHandled)
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

    pub(crate) fn parent_reflector(
        mut self, parent_reflector: Option<(WidgetDirection, u64)>,
    ) -> Self {
        self.parent_reflector = parent_reflector;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
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
            "battery" | "batt" if cfg!(feature = "battery") => Ok(BottomWidgetType::Battery),
            _ => {
                if cfg!(feature = "battery") {
                    Err(BottomError::ConfigError(format!(
                        "\"{}\" is an invalid widget name.
        
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
                ",
                        s
                    )))
                } else {
                    Err(BottomError::ConfigError(format!(
                        "\"{}\" is an invalid widget name.

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
                ",
                        s
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
