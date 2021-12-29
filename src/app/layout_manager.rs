use crate::{
    app::{
        BasicCpu, BasicMem, BasicNet, BatteryTable, Carousel, DiskTable, Empty, MemGraph, NetGraph,
        OldNetGraph, ProcessManager, SelectableType, TempTable,
    },
    error::{BottomError, Result},
    options::{
        layout_options::{LayoutRow, LayoutRowChild, LayoutRule},
        ProcessDefaults,
    },
    tuine::{Element, Flex},
};
use indextree::{Arena, NodeId};
use rustc_hash::FxHashMap;
use std::str::FromStr;
use tui::layout::Rect;

use crate::app::widgets::Widget;

use super::{
    event::SelectionAction, AppConfig, AppState, CpuGraph, OldBottomWidget, TimeGraph, UsedWidgets,
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

impl FromStr for BottomWidgetType {
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

/// Relative movement direction from the currently selected widget.
pub enum MovementDirection {
    Left,
    Right,
    Up,
    Down,
}

pub fn initialize_widget_layout<Message>(
    layout_rows: &[LayoutRow], app: &AppState,
) -> anyhow::Result<Element<Message>> {
    let mut root = Flex::column();

    for layout_row in layout_rows {
        let mut row = Flex::row();
        if let Some(children) = &layout_row.child {
            for child in children {
                match child {
                    LayoutRowChild::Widget(widget) => {}
                    LayoutRowChild::Carousel {
                        carousel_children,
                        default,
                    } => {}
                    LayoutRowChild::LayoutCol {
                        ratio,
                        child: children,
                    } => for child in children {},
                }
            }
        }

        root = root.with_child(row);
    }

    Ok(root.into())
}

/// A wrapper struct to simplify the output of [`create_layout_tree`].
pub struct LayoutCreationOutput {
    pub layout_tree: Arena<LayoutNode>,
    pub root: NodeId,
    pub widget_lookup_map: FxHashMap<NodeId, OldBottomWidget>,
    pub selected: NodeId,
    pub used_widgets: UsedWidgets,
}

/// Creates a new [`Arena<LayoutNode>`] from the given config and returns it, along with the [`NodeId`] representing
/// the root of the newly created [`Arena`], a mapping from [`NodeId`]s to [`BottomWidget`]s, and optionally, a default
/// selected [`NodeId`].
// FIXME: [AFTER REFACTOR] This is currently jury-rigged "glue" just to work with the existing config system! We are NOT keeping it like this, it's too awful to keep like this!
pub fn create_layout_tree(
    rows: &[LayoutRow], process_defaults: ProcessDefaults, app_config_fields: &AppConfig,
) -> Result<LayoutCreationOutput> {
    fn add_widget_to_map(
        widget_lookup_map: &mut FxHashMap<NodeId, OldBottomWidget>, widget_type: BottomWidgetType,
        widget_id: NodeId, process_defaults: &ProcessDefaults, app_config_fields: &AppConfig,
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
                    LayoutRowChild::Widget(widget) => {
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
                    LayoutRowChild::Carousel {
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
                    LayoutRowChild::LayoutCol {
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
    widget_lookup_map: &mut FxHashMap<NodeId, OldBottomWidget>, current_widget_id: NodeId,
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
    widget_lookup_map: &mut FxHashMap<NodeId, OldBottomWidget>, current_widget_id: NodeId,
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
