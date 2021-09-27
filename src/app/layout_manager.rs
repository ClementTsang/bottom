use crate::{
    app::{
        BasicCpu, BasicMem, BasicNet, BatteryTable, Carousel, DiskTable, Empty, MemGraph, NetGraph,
        OldNetGraph, ProcessManager, SelectableType, TempTable,
    },
    error::{BottomError, Result},
    options::{
        layout_options::{LayoutRule, Row, RowChildren},
        ProcessDefaults,
    },
};
use fxhash::FxHashMap;
use indextree::{Arena, NodeId};
use std::cmp::min;
use tui::layout::Rect;

use crate::app::widgets::Widget;

use super::{
    event::SelectionAction, AppConfigFields, CpuGraph, TimeGraph, TmpBottomWidget, UsedWidgets,
};

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
    Carousel,
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
            "battery" | "batt" if cfg!(feature = "battery") => Ok(BottomWidgetType::Battery),
            "bcpu" => Ok(BottomWidgetType::BasicCpu),
            "bmem" => Ok(BottomWidgetType::BasicMem),
            "bnet" => Ok(BottomWidgetType::BasicNet),
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

/// Represents a row in the layout tree.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RowLayout {
    last_selected: Option<NodeId>,
    pub parent_rule: LayoutRule,
    pub bound: Rect,
}

impl RowLayout {
    fn new(parent_rule: LayoutRule) -> Self {
        Self {
            last_selected: None,
            parent_rule,
            bound: Rect::default(),
        }
    }
}

/// Represents a column in the layout tree.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColLayout {
    last_selected: Option<NodeId>,
    pub parent_rule: LayoutRule,
    pub bound: Rect,
}

impl ColLayout {
    fn new(parent_rule: LayoutRule) -> Self {
        Self {
            last_selected: None,
            parent_rule,
            bound: Rect::default(),
        }
    }
}

/// Represents a widget in the layout tree.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WidgetLayout {
    pub bound: Rect,
}

/// A [`LayoutNode`] represents a single node in the overall widget hierarchy. Each node is one of:
/// - [`LayoutNode::Row`] (a non-leaf that distributes its children horizontally)
/// - [`LayoutNode::Col`] (a non-leaf node that distributes its children vertically)
/// - [`LayoutNode::Widget`] (a leaf node that contains the ID of the widget it is associated with)
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LayoutNode {
    /// A non-leaf that distributes its children horizontally
    Row(RowLayout),
    /// A non-leaf node that distributes its children vertically
    Col(ColLayout),
    /// A leaf node that contains the ID of the widget it is associated with
    Widget(WidgetLayout),
}

impl LayoutNode {
    fn set_bound(&mut self, bound: Rect) {
        match self {
            LayoutNode::Row(row) => {
                row.bound = bound;
            }
            LayoutNode::Col(col) => {
                col.bound = bound;
            }
            LayoutNode::Widget(widget) => {
                widget.bound = bound;
            }
        }
    }
}

/// Relative movement direction from the currently selected widget.
pub enum MovementDirection {
    Left,
    Right,
    Up,
    Down,
}

/// A wrapper struct to simplify the output of [`create_layout_tree`].
pub struct LayoutCreationOutput {
    pub layout_tree: Arena<LayoutNode>,
    pub root: NodeId,
    pub widget_lookup_map: FxHashMap<NodeId, TmpBottomWidget>,
    pub selected: NodeId,
    pub used_widgets: UsedWidgets,
}

/// Creates a new [`Arena<LayoutNode>`] from the given config and returns it, along with the [`NodeId`] representing
/// the root of the newly created [`Arena`], a mapping from [`NodeId`]s to [`BottomWidget`]s, and optionally, a default
/// selected [`NodeId`].
// FIXME: [AFTER REFACTOR] This is currently jury-rigged "glue" just to work with the existing config system! We are NOT keeping it like this, it's too awful to keep like this!
pub fn create_layout_tree(
    rows: &[Row], process_defaults: ProcessDefaults, app_config_fields: &AppConfigFields,
) -> Result<LayoutCreationOutput> {
    fn add_widget_to_map(
        widget_lookup_map: &mut FxHashMap<NodeId, TmpBottomWidget>, widget_type: BottomWidgetType,
        widget_id: NodeId, process_defaults: &ProcessDefaults, app_config_fields: &AppConfigFields,
        width: LayoutRule, height: LayoutRule,
    ) -> Result<()> {
        match widget_type {
            BottomWidgetType::Cpu => {
                widget_lookup_map.insert(
                    widget_id,
                    CpuGraph::from_config(app_config_fields)
                        .width(width)
                        .height(height)
                        .into(),
                );
            }
            BottomWidgetType::Mem => {
                let graph = TimeGraph::from_config(app_config_fields);
                widget_lookup_map.insert(
                    widget_id,
                    MemGraph::new(graph).width(width).height(height).into(),
                );
            }
            BottomWidgetType::Net => {
                if app_config_fields.use_old_network_legend {
                    widget_lookup_map.insert(
                        widget_id,
                        OldNetGraph::from_config(app_config_fields)
                            .width(width)
                            .height(height)
                            .into(),
                    );
                } else {
                    widget_lookup_map.insert(
                        widget_id,
                        NetGraph::from_config(app_config_fields)
                            .width(width)
                            .height(height)
                            .into(),
                    );
                }
            }
            BottomWidgetType::Proc => {
                widget_lookup_map.insert(
                    widget_id,
                    ProcessManager::new(process_defaults, app_config_fields)
                        .width(width)
                        .height(height)
                        .basic_mode(app_config_fields.use_basic_mode)
                        .show_scroll_index(app_config_fields.show_table_scroll_position)
                        .into(),
                );
            }
            BottomWidgetType::Temp => {
                widget_lookup_map.insert(
                    widget_id,
                    TempTable::from_config(app_config_fields)
                        .set_temp_type(app_config_fields.temperature_type.clone())
                        .width(width)
                        .height(height)
                        .basic_mode(app_config_fields.use_basic_mode)
                        .show_scroll_index(app_config_fields.show_table_scroll_position)
                        .into(),
                );
            }
            BottomWidgetType::Disk => {
                widget_lookup_map.insert(
                    widget_id,
                    DiskTable::from_config(app_config_fields)
                        .width(width)
                        .height(height)
                        .basic_mode(app_config_fields.use_basic_mode)
                        .show_scroll_index(app_config_fields.show_table_scroll_position)
                        .into(),
                );
            }
            BottomWidgetType::Battery => {
                widget_lookup_map.insert(
                    widget_id,
                    BatteryTable::default()
                        .width(width)
                        .height(height)
                        .basic_mode(app_config_fields.use_basic_mode)
                        .into(),
                );
            }
            BottomWidgetType::BasicCpu => {
                widget_lookup_map.insert(
                    widget_id,
                    BasicCpu::from_config(app_config_fields).width(width).into(),
                );
            }
            BottomWidgetType::BasicMem => {
                widget_lookup_map.insert(widget_id, BasicMem::default().width(width).into());
            }
            BottomWidgetType::BasicNet => {
                widget_lookup_map.insert(
                    widget_id,
                    BasicNet::from_config(app_config_fields).width(width).into(),
                );
            }
            BottomWidgetType::Empty => {
                widget_lookup_map.insert(
                    widget_id,
                    Empty::default().width(width).height(height).into(),
                );
            }
            _ => {}
        }

        Ok(())
    }

    let mut arena = Arena::new();
    let root_id = arena.new_node(LayoutNode::Col(ColLayout::new(LayoutRule::Expand {
        ratio: 1,
    })));
    let mut widget_lookup_map = FxHashMap::default();
    let mut first_selected = None;
    let mut first_widget_seen = None; // Backup selected widget
    let mut used_widgets = UsedWidgets::default();

    for row in rows {
        let row_id = arena.new_node(LayoutNode::Row(RowLayout::new(
            row.ratio
                .map(|ratio| LayoutRule::Expand { ratio })
                .unwrap_or(LayoutRule::Child),
        )));
        root_id.append(row_id, &mut arena);

        if let Some(children) = &row.child {
            for child in children {
                match child {
                    RowChildren::Widget(widget) => {
                        let widget_id = arena.new_node(LayoutNode::Widget(WidgetLayout::default()));
                        row_id.append(widget_id, &mut arena);

                        if let Some(true) = widget.default {
                            first_selected = Some(widget_id);
                        }

                        if first_widget_seen.is_none() {
                            first_widget_seen = Some(widget_id);
                        }

                        let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;
                        used_widgets.add(&widget_type);

                        add_widget_to_map(
                            &mut widget_lookup_map,
                            widget_type,
                            widget_id,
                            &process_defaults,
                            app_config_fields,
                            widget.rule.unwrap_or_default(),
                            LayoutRule::default(),
                        )?;
                    }
                    RowChildren::Carousel {
                        carousel_children,
                        default,
                    } => {
                        if !carousel_children.is_empty() {
                            let mut child_ids = Vec::with_capacity(carousel_children.len());
                            let carousel_widget_id =
                                arena.new_node(LayoutNode::Widget(WidgetLayout::default()));
                            row_id.append(carousel_widget_id, &mut arena);

                            if let Some(true) = default {
                                first_selected = Some(carousel_widget_id);
                            }

                            if first_widget_seen.is_none() {
                                first_widget_seen = Some(carousel_widget_id);
                            }

                            // Handle the rest of the children.
                            for child in carousel_children {
                                let widget_id =
                                    arena.new_node(LayoutNode::Widget(WidgetLayout::default()));
                                carousel_widget_id.append(widget_id, &mut arena);

                                let widget_type = child.parse::<BottomWidgetType>()?;
                                used_widgets.add(&widget_type);

                                add_widget_to_map(
                                    &mut widget_lookup_map,
                                    widget_type,
                                    widget_id,
                                    &process_defaults,
                                    app_config_fields,
                                    LayoutRule::default(),
                                    LayoutRule::default(),
                                )?;

                                child_ids.push(widget_id);
                            }

                            widget_lookup_map.insert(
                                carousel_widget_id,
                                Carousel::new(
                                    child_ids
                                        .into_iter()
                                        .filter_map(|child_id| {
                                            widget_lookup_map
                                                .get(&child_id)
                                                .map(|w| (child_id, w.get_pretty_name().into()))
                                        })
                                        .collect(),
                                )
                                .into(),
                            );
                        }
                    }
                    RowChildren::Col {
                        ratio,
                        child: col_child,
                    } => {
                        let col_id = arena.new_node(LayoutNode::Col(ColLayout::new(
                            ratio
                                .map(|ratio| LayoutRule::Expand { ratio })
                                .unwrap_or(LayoutRule::Child),
                        )));
                        row_id.append(col_id, &mut arena);

                        for widget in col_child {
                            let widget_id =
                                arena.new_node(LayoutNode::Widget(WidgetLayout::default()));
                            col_id.append(widget_id, &mut arena);

                            if let Some(true) = widget.default {
                                first_selected = Some(widget_id);
                            }

                            if first_widget_seen.is_none() {
                                first_widget_seen = Some(widget_id);
                            }

                            let widget_type = widget.widget_type.parse::<BottomWidgetType>()?;
                            used_widgets.add(&widget_type);

                            add_widget_to_map(
                                &mut widget_lookup_map,
                                widget_type,
                                widget_id,
                                &process_defaults,
                                app_config_fields,
                                LayoutRule::default(),
                                widget.rule.unwrap_or_default(),
                            )?;
                        }
                    }
                }
            }
        }
    }

    let selected: NodeId;
    if let Some(first_selected) = first_selected {
        selected = first_selected;
    } else if let Some(first_widget_seen) = first_widget_seen {
        selected = first_widget_seen;
    } else {
        return Err(BottomError::ConfigError(
            "A layout cannot contain zero widgets!".to_string(),
        ));
    }

    correct_layout_last_selections(&mut arena, selected);

    Ok(LayoutCreationOutput {
        layout_tree: arena,
        root: root_id,
        widget_lookup_map,
        selected,
        used_widgets,
    })
}

/// We may have situations where we also have to make sure the correct layout indices are selected.
/// For example, when we select a widget by clicking, we want to update the layout so that it's as if a user
/// manually moved to it via keybinds.
///
/// We can do this by just going through the ancestors, starting from the widget itself.
pub fn correct_layout_last_selections(arena: &mut Arena<LayoutNode>, selected: NodeId) {
    let mut selected_ancestors = selected.ancestors(arena).collect::<Vec<_>>();
    let prev_node = selected_ancestors.pop();
    if let Some(mut prev_node) = prev_node {
        for node in selected_ancestors {
            if let Some(layout_node) = arena.get_mut(node).map(|n| n.get_mut()) {
                match layout_node {
                    LayoutNode::Row(RowLayout { last_selected, .. })
                    | LayoutNode::Col(ColLayout { last_selected, .. }) => {
                        *last_selected = Some(prev_node);
                    }
                    LayoutNode::Widget(_) => {}
                }
            }
            prev_node = node;
        }
    }
}

pub enum MoveWidgetResult {
    ForceRedraw(NodeId),
    NodeId(NodeId),
}

/// A more restricted movement, only within a single widget.
pub fn move_expanded_widget_selection(
    widget_lookup_map: &mut FxHashMap<NodeId, TmpBottomWidget>, current_widget_id: NodeId,
    direction: MovementDirection,
) -> MoveWidgetResult {
    if let Some(current_widget) = widget_lookup_map.get_mut(&current_widget_id) {
        match match direction {
            MovementDirection::Left => current_widget.handle_widget_selection_left(),
            MovementDirection::Right => current_widget.handle_widget_selection_right(),
            MovementDirection::Up => current_widget.handle_widget_selection_up(),
            MovementDirection::Down => current_widget.handle_widget_selection_down(),
        } {
            SelectionAction::Handled => MoveWidgetResult::ForceRedraw(current_widget_id),
            SelectionAction::NotHandled => MoveWidgetResult::NodeId(current_widget_id),
        }
    } else {
        MoveWidgetResult::NodeId(current_widget_id)
    }
}

/// Attempts to find and return the selected [`BottomWidgetId`] after moving in a direction.
///
/// Note this function assumes a properly built tree - if not, bad things may happen! We generally assume that:
/// - Only [`LayoutNode::Widget`]s are leaves.
/// - Only [`LayoutNode::Row`]s or [`LayoutNode::Col`]s are non-leaves.
pub fn move_widget_selection(
    layout_tree: &mut Arena<LayoutNode>,
    widget_lookup_map: &mut FxHashMap<NodeId, TmpBottomWidget>, current_widget_id: NodeId,
    direction: MovementDirection,
) -> MoveWidgetResult {
    // We first give our currently-selected widget a chance to react to the movement - it may handle it internally!
    let handled = {
        if let Some(current_widget) = widget_lookup_map.get_mut(&current_widget_id) {
            match direction {
                MovementDirection::Left => current_widget.handle_widget_selection_left(),
                MovementDirection::Right => current_widget.handle_widget_selection_right(),
                MovementDirection::Up => current_widget.handle_widget_selection_up(),
                MovementDirection::Down => current_widget.handle_widget_selection_down(),
            }
        } else {
            // Short circuit.
            return MoveWidgetResult::NodeId(current_widget_id);
        }
    };

    match handled {
        SelectionAction::Handled => {
            // If it was handled by the widget, then we don't have to do anything - return the current one.
            MoveWidgetResult::ForceRedraw(current_widget_id)
        }
        SelectionAction::NotHandled => {
            /// Keeps traversing up the `layout_tree` until it hits a parent where `current_id` is a child and parent
            /// is a [`LayoutNode::Row`], returning its parent's [`NodeId`] and the child's [`NodeId`] (in that order).
            /// If this crawl fails (i.e. hits a root, it is an invalid tree for some reason), it returns [`None`].
            fn find_parent_row(
                layout_tree: &Arena<LayoutNode>, current_id: NodeId,
            ) -> Option<(NodeId, NodeId)> {
                layout_tree
                    .get(current_id)
                    .and_then(|current_node| current_node.parent())
                    .and_then(|parent_id| {
                        layout_tree
                            .get(parent_id)
                            .map(|parent_node| (parent_id, parent_node))
                    })
                    .and_then(|(parent_id, parent_node)| match parent_node.get() {
                        LayoutNode::Row(_) => Some((parent_id, current_id)),
                        LayoutNode::Col(_) => find_parent_row(layout_tree, parent_id),
                        LayoutNode::Widget(_) => None,
                    })
            }

            /// Keeps traversing up the `layout_tree` until it hits a parent where `current_id` is a child and parent
            /// is a [`LayoutNode::Col`], returning its parent's [`NodeId`] and the child's [`NodeId`] (in that order).
            /// If this crawl fails (i.e. hits a root, it is an invalid tree for some reason), it returns [`None`].
            fn find_parent_col(
                layout_tree: &Arena<LayoutNode>, current_id: NodeId,
            ) -> Option<(NodeId, NodeId)> {
                layout_tree
                    .get(current_id)
                    .and_then(|current_node| current_node.parent())
                    .and_then(|parent_id| {
                        layout_tree
                            .get(parent_id)
                            .map(|parent_node| (parent_id, parent_node))
                    })
                    .and_then(|(parent_id, parent_node)| match parent_node.get() {
                        LayoutNode::Row(_) => find_parent_col(layout_tree, parent_id),
                        LayoutNode::Col(_) => Some((parent_id, current_id)),
                        LayoutNode::Widget(_) => None,
                    })
            }

            /// Descends to a leaf node.
            fn descend_to_leaf(layout_tree: &Arena<LayoutNode>, current_id: NodeId) -> NodeId {
                if let Some(current_node) = layout_tree.get(current_id) {
                    match current_node.get() {
                        LayoutNode::Row(RowLayout {
                            last_selected,
                            parent_rule: _,
                            bound: _,
                        })
                        | LayoutNode::Col(ColLayout {
                            last_selected,
                            parent_rule: _,
                            bound: _,
                        }) => {
                            if let Some(next_child) = *last_selected {
                                descend_to_leaf(layout_tree, next_child)
                            } else {
                                descend_to_leaf(
                                    layout_tree,
                                    current_node.first_child().unwrap_or(current_id),
                                )
                            }
                        }
                        LayoutNode::Widget(_) => {
                            // Halt!
                            current_id
                        }
                    }
                } else {
                    current_id
                }
            }

            // If it was NOT handled by the current widget, then move in the correct direction; we can rely
            // on the tree layout to help us decide where to go.
            // Movement logic is inspired by i3. When we enter a new column/row, we go to the *last* selected
            // element; if we can't, go to the nearest one.
            let proposed_id = match direction {
                MovementDirection::Left => {
                    // When we move "left":
                    // 1. Look for the parent of the current widget.
                    // 2. Depending on whether it is a Row or Col:
                    //  a) If we are in a Row, try to move to the child (it can be a Row, Col, or Widget) before it,
                    //     and update the last-selected index. If we can't (i.e. we are the first element), then
                    //     instead move to the parent, and try again to select the element before it. If there is
                    //     no parent (i.e. we hit the root), then just return the original index.
                    //  b) If we are in a Col, then just try to move to the parent. If there is no
                    //     parent (i.e. we hit the root), then just return the original index.
                    //  c) A Widget should be impossible to select.
                    // 3. Assuming we have now selected a new child, then depending on what the child is:
                    //  a) If we are in a Row or Col, then take the last selected index, and repeat step 3 until you hit
                    //     a Widget.
                    //  b) If we are in a Widget, return the corresponding NodeId.

                    fn find_left(
                        layout_tree: &mut Arena<LayoutNode>, current_id: NodeId,
                    ) -> NodeId {
                        if let Some((parent_id, child_id)) =
                            find_parent_row(layout_tree, current_id)
                        {
                            if let Some(prev_sibling) =
                                child_id.preceding_siblings(layout_tree).nth(1)
                            {
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Row(row) = parent.get_mut() {
                                        row.last_selected = Some(prev_sibling);
                                    }
                                }

                                descend_to_leaf(layout_tree, prev_sibling)
                            } else if parent_id != current_id {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_left(layout_tree, parent_id)
                            } else {
                                current_id
                            }
                        } else {
                            // Failed, just return the current ID.
                            current_id
                        }
                    }
                    find_left(layout_tree, current_widget_id)
                }
                MovementDirection::Right => {
                    // When we move "right", repeat the steps for "left", but instead try to move to the child *after*
                    // it in all cases.

                    fn find_right(
                        layout_tree: &mut Arena<LayoutNode>, current_id: NodeId,
                    ) -> NodeId {
                        if let Some((parent_id, child_id)) =
                            find_parent_row(layout_tree, current_id)
                        {
                            if let Some(following_sibling) =
                                child_id.following_siblings(layout_tree).nth(1)
                            {
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Row(row) = parent.get_mut() {
                                        row.last_selected = Some(following_sibling);
                                    }
                                }

                                descend_to_leaf(layout_tree, following_sibling)
                            } else if parent_id != current_id {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_right(layout_tree, parent_id)
                            } else {
                                current_id
                            }
                        } else {
                            // Failed, just return the current ID.
                            current_id
                        }
                    }
                    find_right(layout_tree, current_widget_id)
                }
                MovementDirection::Up => {
                    // When we move "up", copy the steps for "left", but switch "Row" and "Col".  We instead want to move
                    // vertically, so we want to now avoid Rows and look for Cols!

                    fn find_above(
                        layout_tree: &mut Arena<LayoutNode>, current_id: NodeId,
                    ) -> NodeId {
                        if let Some((parent_id, child_id)) =
                            find_parent_col(layout_tree, current_id)
                        {
                            if let Some(prev_sibling) =
                                child_id.preceding_siblings(layout_tree).nth(1)
                            {
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Col(row) = parent.get_mut() {
                                        row.last_selected = Some(prev_sibling);
                                    }
                                }

                                descend_to_leaf(layout_tree, prev_sibling)
                            } else if parent_id != current_id {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_above(layout_tree, parent_id)
                            } else {
                                current_id
                            }
                        } else {
                            // Failed, just return the current ID.
                            current_id
                        }
                    }
                    find_above(layout_tree, current_widget_id)
                }
                MovementDirection::Down => {
                    // See "up"'s steps, but now we're going for the child *after* the currently selected one in all
                    // cases.

                    fn find_below(
                        layout_tree: &mut Arena<LayoutNode>, current_id: NodeId,
                    ) -> NodeId {
                        if let Some((parent_id, child_id)) =
                            find_parent_col(layout_tree, current_id)
                        {
                            if let Some(following_sibling) =
                                child_id.following_siblings(layout_tree).nth(1)
                            {
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Col(row) = parent.get_mut() {
                                        row.last_selected = Some(following_sibling);
                                    }
                                }

                                descend_to_leaf(layout_tree, following_sibling)
                            } else if parent_id != current_id {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_below(layout_tree, parent_id)
                            } else {
                                current_id
                            }
                        } else {
                            // Failed, just return the current ID.
                            current_id
                        }
                    }
                    find_below(layout_tree, current_widget_id)
                }
            };

            if let Some(LayoutNode::Widget(_)) = layout_tree.get(proposed_id).map(|n| n.get()) {
                if let Some(proposed_widget) = widget_lookup_map.get_mut(&proposed_id) {
                    match proposed_widget.selectable_type() {
                        SelectableType::Unselectable => {
                            // FIXME: [URGENT] Test this; make sure this cannot recurse infinitely!  Maybe through a unit test too.
                            // Try to move again recursively.
                            move_widget_selection(
                                layout_tree,
                                widget_lookup_map,
                                proposed_id,
                                direction,
                            )
                        }
                        SelectableType::Selectable => MoveWidgetResult::NodeId(proposed_id),
                    }
                } else {
                    MoveWidgetResult::NodeId(current_widget_id)
                }
            } else {
                MoveWidgetResult::NodeId(current_widget_id)
            }
        }
    }
}

/// Generates the bounds for each node in the `arena, taking into account per-leaf desires,
/// and finally storing the calculated bounds in the given `arena`.
///
/// Stored bounds are given in *relative* coordinates - they are relative to their parents.
/// That is, you may have a child widget "start" at (0, 0), but its parent is actually at x = 5,s
/// so the absolute coordinate of the child widget is actually (5, 0).
///
/// The algorithm is mostly based on the algorithm used by Flutter, adapted to work for
/// our use case. For more information, check out both:
///
/// - [How the constraint system works in Flutter](https://flutter.dev/docs/development/ui/layout/constraints)
/// - [How Flutter does sublinear layout](https://flutter.dev/docs/resources/inside-flutter#sublinear-layout)
pub fn generate_layout(
    root: NodeId, arena: &mut Arena<LayoutNode>, area: Rect,
    lookup_map: &FxHashMap<NodeId, TmpBottomWidget>,
) {
    // TODO: [Optimization, Layout] Add some caching/dirty mechanisms to reduce calls.

    /// A [`Size`] is a set of widths and heights that a node in our layout wants to be.
    #[derive(Default, Clone, Copy, Debug)]
    struct Size {
        width: u16,
        height: u16,
    }

    /// A [`LayoutConstraint`] is just a set of maximal widths/heights.
    #[derive(Clone, Copy, Debug)]
    struct LayoutConstraints {
        max_width: u16,
        max_height: u16,
    }

    impl LayoutConstraints {
        fn new(max_width: u16, max_height: u16) -> Self {
            Self {
                max_width,
                max_height,
            }
        }

        /// Shrinks the width of itself given another width.
        fn shrink_width(&mut self, width: u16) {
            self.max_width = self.max_width.saturating_sub(width);
        }

        /// Shrinks the height of itself given another height.
        fn shrink_height(&mut self, height: u16) {
            self.max_height = self.max_height.saturating_sub(height);
        }

        /// Returns a new [`LayoutConstraints`] with a new width given a ratio.
        fn ratio_width(&self, numerator: u32, denominator: u32) -> Self {
            Self {
                max_width: (self.max_width as u32 * numerator / denominator) as u16,
                max_height: self.max_height,
            }
        }

        /// Returns a new [`LayoutConstraints`] with a new height given a ratio.
        fn ratio_height(&self, numerator: u32, denominator: u32) -> Self {
            Self {
                max_width: self.max_width,
                max_height: (self.max_height as u32 * numerator / denominator) as u16,
            }
        }
    }

    /// The internal recursive call to build a layout. Builds off of `arena` and stores bounds inside it.
    fn layout(
        node: NodeId, arena: &mut Arena<LayoutNode>,
        lookup_map: &FxHashMap<NodeId, TmpBottomWidget>, mut constraints: LayoutConstraints,
    ) -> Size {
        if let Some(layout_node) = arena.get(node).map(|n| n.get()) {
            match layout_node {
                LayoutNode::Row(row) => {
                    let children = node.children(arena).collect::<Vec<_>>();
                    let mut row_bounds = vec![Size::default(); children.len()];

                    if let LayoutRule::Length { length } = row.parent_rule {
                        constraints.max_height = length;
                    }

                    let (flexible_indices, inflexible_indices): (Vec<_>, Vec<_>) = children
                        .iter()
                        .enumerate()
                        .filter_map(|(itx, node)| {
                            if let Some(layout_node) = arena.get(*node).map(|n| n.get()) {
                                match layout_node {
                                    LayoutNode::Row(RowLayout { parent_rule, .. })
                                    | LayoutNode::Col(ColLayout { parent_rule, .. }) => {
                                        match parent_rule {
                                            LayoutRule::Expand { ratio } => {
                                                Some((itx, true, *ratio))
                                            }
                                            LayoutRule::Child => Some((itx, false, 0)),
                                            LayoutRule::Length { .. } => Some((itx, false, 0)),
                                        }
                                    }
                                    LayoutNode::Widget(_) => {
                                        if let Some(widget) = lookup_map.get(node) {
                                            match widget.width() {
                                                LayoutRule::Expand { ratio } => {
                                                    Some((itx, true, ratio))
                                                }
                                                LayoutRule::Child => Some((itx, false, 0)),
                                                LayoutRule::Length { .. } => Some((itx, false, 0)),
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                }
                            } else {
                                None
                            }
                        })
                        .partition(|(_itx, is_flex, _ratio)| *is_flex);

                    // First handle non-flexible children.
                    for (index, _, _) in inflexible_indices {
                        // The unchecked get is safe, since the index is obtained by iterating through the children
                        // vector in the first place.
                        let child = unsafe { children.get_unchecked(index) };
                        let desired_size = layout(*child, arena, lookup_map, constraints);

                        constraints.shrink_width(desired_size.width);

                        // This won't panic, since the two vectors are the same length.
                        row_bounds[index] = desired_size;
                    }

                    // Handle flexible children now.
                    let denominator: u32 = flexible_indices.iter().map(|(_, _, ratio)| ratio).sum();
                    let original_constraints = constraints;
                    let mut split_constraints = flexible_indices
                        .iter()
                        .map(|(_, _, numerator)| {
                            let constraint =
                                original_constraints.ratio_width(*numerator, denominator);
                            constraints.shrink_width(constraint.max_width);

                            constraint
                        })
                        .collect::<Vec<_>>();
                    (0..constraints.max_width)
                        .zip(&mut split_constraints)
                        .for_each(|(_, split_constraint)| {
                            split_constraint.max_width += 1;
                        });

                    for ((index, _, _), constraint) in
                        flexible_indices.into_iter().zip(split_constraints)
                    {
                        // The unchecked get is safe, since the index is obtained by iterating through the children
                        // vector in the first place.
                        let child = unsafe { children.get_unchecked(index) };
                        let desired_size = layout(*child, arena, lookup_map, constraint);

                        // This won't panic, since the two vectors are the same length.
                        row_bounds[index] = desired_size;
                    }

                    // Now let's turn each Size into a relative Rect!
                    let mut current_x = 0;
                    row_bounds.iter().zip(children).for_each(|(size, child)| {
                        let bound = Rect::new(current_x, 0, size.width, size.height);
                        current_x += size.width;
                        if let Some(node) = arena.get_mut(child) {
                            node.get_mut().set_bound(bound);
                        }
                    });

                    Size {
                        height: row_bounds.iter().map(|size| size.height).max().unwrap_or(0),
                        width: row_bounds.into_iter().map(|size| size.width).sum(),
                    }
                }
                LayoutNode::Col(col) => {
                    let children = node.children(arena).collect::<Vec<_>>();
                    let mut col_bounds = vec![Size::default(); children.len()];

                    if let LayoutRule::Length { length } = col.parent_rule {
                        constraints.max_width = length;
                    }

                    let (flexible_indices, inflexible_indices): (Vec<_>, Vec<_>) = children
                        .iter()
                        .enumerate()
                        .filter_map(|(itx, node)| {
                            if let Some(layout_node) = arena.get(*node).map(|n| n.get()) {
                                match layout_node {
                                    LayoutNode::Row(RowLayout { parent_rule, .. })
                                    | LayoutNode::Col(ColLayout { parent_rule, .. }) => {
                                        match parent_rule {
                                            LayoutRule::Expand { ratio } => {
                                                Some((itx, true, *ratio))
                                            }
                                            LayoutRule::Child => Some((itx, false, 0)),
                                            LayoutRule::Length { .. } => Some((itx, false, 0)),
                                        }
                                    }
                                    LayoutNode::Widget(_) => {
                                        if let Some(widget) = lookup_map.get(node) {
                                            match widget.height() {
                                                LayoutRule::Expand { ratio } => {
                                                    Some((itx, true, ratio))
                                                }
                                                LayoutRule::Child => Some((itx, false, 0)),
                                                LayoutRule::Length { length: _ } => {
                                                    Some((itx, false, 0))
                                                }
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                }
                            } else {
                                None
                            }
                        })
                        .partition(|(_itx, is_flex, _ratio)| *is_flex);

                    for (index, _, _) in inflexible_indices {
                        // The unchecked get is safe, since the index is obtained by iterating through the children
                        // vector in the first place.
                        let child = unsafe { children.get_unchecked(index) };
                        let desired_size = layout(*child, arena, lookup_map, constraints);

                        constraints.shrink_height(desired_size.height);

                        // This won't panic, since the two vectors are the same length.
                        col_bounds[index] = desired_size;
                    }

                    let denominator: u32 = flexible_indices.iter().map(|(_, _, ratio)| ratio).sum();
                    let original_constraints = constraints;
                    let mut split_constraints = flexible_indices
                        .iter()
                        .map(|(_, _, numerator)| {
                            let new_constraint =
                                original_constraints.ratio_height(*numerator, denominator);
                            constraints.shrink_height(new_constraint.max_height);

                            new_constraint
                        })
                        .collect::<Vec<_>>();
                    (0..constraints.max_height)
                        .zip(&mut split_constraints)
                        .for_each(|(_, split_constraint)| {
                            split_constraint.max_height += 1;
                        });

                    for ((index, _, _), constraint) in
                        flexible_indices.into_iter().zip(split_constraints)
                    {
                        // The unchecked get is safe, since the index is obtained by iterating through the children
                        // vector in the first place.
                        let child = unsafe { children.get_unchecked(index) };
                        let desired_size = layout(*child, arena, lookup_map, constraint);

                        // This won't panic, since the two vectors are the same length.
                        col_bounds[index] = desired_size;
                    }

                    // Now let's turn each Size into a relative Rect!
                    let mut current_y = 0;
                    col_bounds.iter().zip(children).for_each(|(size, child)| {
                        let bound = Rect::new(0, current_y, size.width, size.height);
                        current_y += size.height;
                        if let Some(node) = arena.get_mut(child) {
                            node.get_mut().set_bound(bound);
                        }
                    });

                    Size {
                        width: col_bounds.iter().map(|size| size.width).max().unwrap_or(0),
                        height: col_bounds.into_iter().map(|size| size.height).sum(),
                    }
                }
                LayoutNode::Widget(_) => {
                    if let Some(widget) = lookup_map.get(&node) {
                        let width = match widget.width() {
                            LayoutRule::Expand { ratio: _ } => constraints.max_width,
                            LayoutRule::Length { length } => min(length, constraints.max_width),
                            LayoutRule::Child => constraints.max_width,
                        };

                        let height = match widget.height() {
                            LayoutRule::Expand { ratio: _ } => constraints.max_height,
                            LayoutRule::Length { length } => min(length, constraints.max_height),
                            LayoutRule::Child => constraints.max_height,
                        };

                        Size { width, height }
                    } else {
                        Size::default()
                    }
                }
            }
        } else {
            Size::default()
        }
    }

    // And this is all you need to call, the layout function will do it all~
    layout(
        root,
        arena,
        lookup_map,
        LayoutConstraints::new(area.width, area.height),
    );
}
