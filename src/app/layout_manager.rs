use crate::{
    app::SelectableType,
    data_conversion::ConvertedData,
    error::{BottomError, Result},
    options::layout_options::{FinalWidget, LayoutRow, LayoutRowChild, WidgetLayoutRule},
    tuine::*,
};
use anyhow::anyhow;
use indextree::{Arena, NodeId};
use rustc_hash::FxHashMap;
use std::str::FromStr;
use tui::layout::Rect;

use crate::app::widgets::Widget;

use super::{event::SelectionAction, AppState, OldBottomWidget, UsedWidgets};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BottomWidgetType {
    Empty,
    Cpu,
    Mem,
    Net,
    Proc,
    Temp,
    Disk,
    BasicCpu,
    BasicMem,
    BasicNet,
    Battery,
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
    pub parent_rule: WidgetLayoutRule,
    pub bound: Rect,
}

/// Represents a column in the layout tree.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColLayout {
    last_selected: Option<NodeId>,
    pub parent_rule: WidgetLayoutRule,
    pub bound: Rect,
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

/// The root of the widget layout intermediate representation. A wrapper around a `Vec` of [`WidgetLayoutNode`];
/// it ALWAYS represents a column.
#[derive(Debug, Clone)]
pub struct WidgetLayoutRoot {
    pub children: Vec<WidgetLayoutNode>,
}

impl WidgetLayoutRoot {
    pub fn build<Message>(
        &self, ctx: &mut BuildContext<'_>, app_state: &AppState, data: &mut ConvertedData<'_>,
    ) -> Element<Message> {
        Flex::column_with_children(
            self.children
                .iter()
                .map(|child| child.build(ctx, app_state, data))
                .collect(),
        )
        .into()
    }
}

/// An intermediate representation of the widget layout.
#[derive(Debug, Clone)]
pub enum WidgetLayoutNode {
    Row {
        children: Vec<WidgetLayoutNode>,
        parent_ratio: u16,
    },
    Col {
        children: Vec<WidgetLayoutNode>,
        parent_ratio: u16,
    },
    Carousel {
        children: Vec<BottomWidgetType>,
        selected: bool,
    },
    Widget {
        widget_type: BottomWidgetType,
        selected: bool,
        rule: WidgetLayoutRule,
    },
}

impl WidgetLayoutNode {
    fn new_row<Message>(
        ctx: &mut BuildContext<'_>, app_state: &AppState, data: &mut ConvertedData<'_>,
        children: &[WidgetLayoutNode], parent_ratio: u16,
    ) -> FlexElement<Message> {
        FlexElement::with_flex(
            Flex::row_with_children(
                children
                    .iter()
                    .map(|child| child.build(ctx, app_state, data))
                    .collect(),
            ),
            parent_ratio,
        )
    }

    fn new_col<Message>(
        ctx: &mut BuildContext<'_>, app_state: &AppState, data: &mut ConvertedData<'_>,
        children: &[WidgetLayoutNode], parent_ratio: u16,
    ) -> FlexElement<Message> {
        FlexElement::with_flex(
            Flex::column_with_children(
                children
                    .iter()
                    .map(|child| child.build(ctx, app_state, data))
                    .collect(),
            ),
            parent_ratio,
        )
    }

    fn new_carousel<Message>(
        ctx: &mut BuildContext<'_>, app_state: &AppState, data: &mut ConvertedData<'_>,
        children: &[BottomWidgetType], selected: bool,
    ) -> FlexElement<Message> {
        // FIXME: Carousel!
        FlexElement::new(Flex::row_with_children(
            children
                .iter()
                .map(|child| {
                    FlexElement::new(Self::make_element(ctx, app_state, data, child, false))
                })
                .collect(),
        ))
    }

    fn make_element<Message>(
        ctx: &mut BuildContext<'_>, app_state: &AppState, data: &mut ConvertedData<'_>,
        widget_type: &BottomWidgetType, selected: bool,
    ) -> Element<Message> {
        let painter = &app_state.painter;
        let config = &app_state.app_config;

        match widget_type {
            BottomWidgetType::Empty => Empty::default().into(),
            BottomWidgetType::Cpu => CpuGraph::build(ctx, painter, config, data).into(),
            BottomWidgetType::Mem => MemGraph::build(ctx, painter, config, data).into(),
            BottomWidgetType::Net => NetGraph::build(ctx, painter, config, data).into(),
            BottomWidgetType::Proc => ProcessTable::build(ctx, painter, config, data).into(),
            BottomWidgetType::Temp => TempTable::build(ctx, painter, config, data).into(),
            BottomWidgetType::Disk => DiskTable::build(ctx, painter, config, data).into(),
            BottomWidgetType::BasicCpu => CpuSimple::build(ctx, painter, config, data).into(),
            BottomWidgetType::BasicMem => MemSimple::build(ctx, painter, config, data).into(),
            BottomWidgetType::BasicNet => NetSimple::build(ctx, painter, config, data).into(),
            BottomWidgetType::Battery => BatteryTable::build(ctx, painter, config, data).into(),
        }
    }

    fn wrap_element<Message>(
        element: Element<Message>, rule: &WidgetLayoutRule,
    ) -> FlexElement<Message> {
        match rule {
            WidgetLayoutRule::Expand { ratio } => FlexElement::with_flex(element, *ratio),
            WidgetLayoutRule::Length { width, height } => {
                if width.is_some() || height.is_some() {
                    FlexElement::with_no_flex(
                        Container::with_child(element).width(*width).height(*height),
                    )
                } else {
                    FlexElement::with_flex(element, 1)
                }
            }
        }
    }

    pub fn build<Message>(
        &self, ctx: &mut BuildContext<'_>, app_state: &AppState, data: &mut ConvertedData<'_>,
    ) -> FlexElement<Message> {
        match self {
            WidgetLayoutNode::Row {
                children,
                parent_ratio,
            } => Self::new_row(ctx, app_state, data, children, *parent_ratio),
            WidgetLayoutNode::Col {
                children,
                parent_ratio,
            } => Self::new_col(ctx, app_state, data, children, *parent_ratio),
            WidgetLayoutNode::Carousel { children, selected } => {
                Self::new_carousel(ctx, app_state, data, children, *selected)
            }
            WidgetLayoutNode::Widget {
                widget_type,
                selected,
                rule,
            } => WidgetLayoutNode::wrap_element(
                Self::make_element(ctx, app_state, data, widget_type, *selected),
                rule,
            ),
        }
    }
}

/// Parses the layout in the config into an intermediate representation.
pub fn parse_widget_layout(
    layout_rows: &[LayoutRow],
) -> anyhow::Result<(WidgetLayoutRoot, UsedWidgets)> {
    let mut root_children = Vec::with_capacity(layout_rows.len());
    let mut used_widgets = UsedWidgets::default();

    for layout_row in layout_rows {
        if let Some(children) = &layout_row.child {
            let mut row_children = Vec::with_capacity(children.len());

            for child in children {
                match child {
                    LayoutRowChild::Widget(widget) => {
                        let FinalWidget {
                            rule,
                            widget_type,
                            default,
                        } = widget;

                        let widget_type = widget_type.parse::<BottomWidgetType>()?;
                        used_widgets.add(&widget_type);

                        row_children.push(WidgetLayoutNode::Widget {
                            widget_type,
                            selected: default.unwrap_or(false),
                            rule: rule.unwrap_or_default(),
                        });
                    }
                    LayoutRowChild::Carousel {
                        carousel_children,
                        default,
                    } => {
                        let mut car_children = Vec::with_capacity(carousel_children.len());
                        for widget_type in carousel_children {
                            let widget_type = widget_type.parse::<BottomWidgetType>()?;
                            used_widgets.add(&widget_type);

                            car_children.push(widget_type);
                        }

                        row_children.push(WidgetLayoutNode::Carousel {
                            children: car_children,
                            selected: default.unwrap_or(false),
                        });
                    }
                    LayoutRowChild::LayoutCol {
                        ratio,
                        child: children,
                    } => {
                        let mut col_children = Vec::with_capacity(children.len());
                        for widget in children {
                            let FinalWidget {
                                rule,
                                widget_type,
                                default,
                            } = widget;
                            let widget_type = widget_type.parse::<BottomWidgetType>()?;
                            used_widgets.add(&widget_type);

                            col_children.push(WidgetLayoutNode::Widget {
                                widget_type,
                                selected: default.unwrap_or(false),
                                rule: rule.unwrap_or_default(),
                            });
                        }

                        row_children.push(WidgetLayoutNode::Col {
                            children: col_children,
                            parent_ratio: ratio.unwrap_or(1),
                        });
                    }
                }
            }

            let row = WidgetLayoutNode::Row {
                children: row_children,
                parent_ratio: layout_row.ratio.unwrap_or(1),
            };
            root_children.push(row);
        }
    }

    if root_children.is_empty() {
        Err(anyhow!("The layout cannot be empty!"))
    } else {
        let root = WidgetLayoutRoot {
            children: root_children,
        };
        Ok((root, used_widgets))
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
