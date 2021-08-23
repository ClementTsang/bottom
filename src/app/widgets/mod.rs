use std::time::Instant;

use crossterm::event::{KeyEvent, MouseEvent};
use enum_dispatch::enum_dispatch;
use indextree::{Arena, NodeId};
use tui::{layout::Rect, widgets::TableState};

use crate::{
    app::{
        event::{EventResult, SelectionAction},
        layout_manager::BottomWidgetType,
    },
    constants,
};

pub mod base;
pub use base::*;

pub mod process;
pub use process::*;

pub mod net;
pub use net::*;

pub mod mem;
pub use mem::*;

pub mod cpu;
pub use cpu::*;

pub mod disk;
pub use disk::*;

pub mod battery;
pub use self::battery::*;

pub mod temp;
pub use temp::*;

/// A trait for things that are drawn with state.
#[enum_dispatch]
#[allow(unused_variables)]
pub trait Component {
    /// Handles a [`KeyEvent`].
    ///
    /// Defaults to returning [`EventResult::NoRedraw`], indicating nothing should be done.
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        EventResult::NoRedraw
    }

    /// Handles a [`MouseEvent`].
    ///
    /// Defaults to returning [`EventResult::Continue`], indicating nothing should be done.
    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        EventResult::NoRedraw
    }

    /// Returns a [`Component`]'s bounding box.  Note that these are defined in *global*, *absolute*
    /// coordinates.
    fn bounds(&self) -> Rect;

    /// Updates a [`Component`]s bounding box to `new_bounds`.
    fn set_bounds(&mut self, new_bounds: Rect);
}

/// A trait for actual fully-fledged widgets to be displayed in bottom.
#[enum_dispatch]
pub trait Widget {
    /// Updates a [`Widget`] given some data.  Defaults to doing nothing.
    fn update(&mut self) {}

    /// Handles what to do when trying to respond to a widget selection movement to the left.
    /// Defaults to just moving to the next-possible widget in that direction.
    fn handle_widget_selection_left(&mut self) -> SelectionAction {
        SelectionAction::NotHandled
    }

    /// Handles what to do when trying to respond to a widget selection movement to the right.
    /// Defaults to just moving to the next-possible widget in that direction.
    fn handle_widget_selection_right(&mut self) -> SelectionAction {
        SelectionAction::NotHandled
    }

    /// Handles what to do when trying to respond to a widget selection movement upward.
    /// Defaults to just moving to the next-possible widget in that direction.
    fn handle_widget_selection_up(&mut self) -> SelectionAction {
        SelectionAction::NotHandled
    }

    /// Handles what to do when trying to respond to a widget selection movement downward.
    /// Defaults to just moving to the next-possible widget in that direction.
    fn handle_widget_selection_down(&mut self) -> SelectionAction {
        SelectionAction::NotHandled
    }

    fn get_pretty_name(&self) -> &'static str;
}

/// The "main" widgets that are used by bottom to display information!
#[enum_dispatch(Component, Widget)]
enum BottomWidget {
    MemGraph,
    TempTable,
    DiskTable,
    CpuGraph,
    NetGraph,
    OldNetGraph,
    ProcessManager,
    BatteryTable,
}

/// Checks whether points `(x, y)` intersect a given [`Rect`].
pub fn does_point_intersect_rect(x: u16, y: u16, rect: Rect) -> bool {
    x >= rect.left() && x <= rect.right() && y >= rect.top() && y <= rect.bottom()
}

/// A [`LayoutNode`] represents a single node in the overall widget hierarchy. Each node is one of:
/// - [`LayoutNode::Row`] (a a non-leaf that distributes its children horizontally)
/// - [`LayoutNode::Col`] (a non-leaf node that distributes its children vertically)
/// - [`LayoutNode::Widget`] (a leaf node that contains the ID of the widget it is associated with)
#[derive(PartialEq, Eq)]
pub enum LayoutNode {
    Row(BottomRow),
    Col(BottomCol),
    Widget,
}

/// A simple struct representing a row and its state.
#[derive(PartialEq, Eq)]
pub struct BottomRow {
    last_selected_index: usize,
}

/// A simple struct representing a column and its state.
#[derive(PartialEq, Eq)]
pub struct BottomCol {
    last_selected_index: usize,
}

/// Relative movement direction from the currently selected widget.
pub enum MovementDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Attempts to find and return the selected [`BottomWidgetId`] after moving in a direction.
///
/// Note this function assumes a properly built tree - if not, bad things may happen! We generally assume that:
/// - Only [`LayoutNode::Widget`]s are leaves.
/// - Only [`LayoutNode::Row`]s or [`LayoutNode::Col`]s are non-leaves.
fn move_widget_selection(
    layout_tree: &mut Arena<LayoutNode>, current_widget: &mut BottomWidget,
    current_widget_id: NodeId, direction: MovementDirection,
) -> NodeId {
    // We first give our currently-selected widget a chance to react to the movement - it may handle it internally!
    let handled = match direction {
        MovementDirection::Left => current_widget.handle_widget_selection_left(),
        MovementDirection::Right => current_widget.handle_widget_selection_right(),
        MovementDirection::Up => current_widget.handle_widget_selection_up(),
        MovementDirection::Down => current_widget.handle_widget_selection_down(),
    };

    match handled {
        SelectionAction::Handled => {
            // If it was handled by the widget, then we don't have to do anything - return the current one.
            current_widget_id
        }
        SelectionAction::NotHandled => {
            /// Keeps traversing up the `layout_tree` until it hits a parent where `current_id` is a child and parent
            /// is a [`LayoutNode::Row`], returning its parent's [`NodeId`] and the child's [`NodeId`] (in that order).
            /// If this crawl fails (i.e. hits a root, it is an invalid tree for some reason), it returns [`None`].
            fn find_first_row(
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
                        LayoutNode::Col(_) => find_first_row(layout_tree, parent_id),
                        LayoutNode::Widget => None,
                    })
            }

            /// Keeps traversing up the `layout_tree` until it hits a parent where `current_id` is a child and parent
            /// is a [`LayoutNode::Col`], returning its parent's [`NodeId`] and the child's [`NodeId`] (in that order).
            /// If this crawl fails (i.e. hits a root, it is an invalid tree for some reason), it returns [`None`].
            fn find_first_col(
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
                        LayoutNode::Row(_) => find_first_col(layout_tree, parent_id),
                        LayoutNode::Col(_) => Some((parent_id, current_id)),
                        LayoutNode::Widget => None,
                    })
            }

            /// Descends to a leaf.
            fn descend_to_leaf(layout_tree: &Arena<LayoutNode>, current_id: NodeId) -> NodeId {
                if let Some(current_node) = layout_tree.get(current_id) {
                    match current_node.get() {
                        LayoutNode::Row(BottomRow {
                            last_selected_index,
                        })
                        | LayoutNode::Col(BottomCol {
                            last_selected_index,
                        }) => {
                            if let Some(next_child) =
                                current_id.children(layout_tree).nth(*last_selected_index)
                            {
                                descend_to_leaf(layout_tree, next_child)
                            } else {
                                current_id
                            }
                        }
                        LayoutNode::Widget => {
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
            match direction {
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
                        if let Some((parent_id, child_id)) = find_first_row(layout_tree, current_id)
                        {
                            if let Some(prev_sibling) =
                                child_id.preceding_siblings(layout_tree).nth(1)
                            {
                                // Subtract one from the currently selected index...
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Row(row) = parent.get_mut() {
                                        row.last_selected_index =
                                            row.last_selected_index.saturating_sub(1);
                                    }
                                }

                                // Now descend downwards!
                                descend_to_leaf(layout_tree, prev_sibling)
                            } else {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_left(layout_tree, child_id)
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
                        if let Some((parent_id, child_id)) = find_first_row(layout_tree, current_id)
                        {
                            if let Some(prev_sibling) =
                                child_id.following_siblings(layout_tree).nth(1)
                            {
                                // Add one to the currently selected index...
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Row(row) = parent.get_mut() {
                                        row.last_selected_index += 1;
                                    }
                                }

                                // Now descend downwards!
                                descend_to_leaf(layout_tree, prev_sibling)
                            } else {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_right(layout_tree, child_id)
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
                        if let Some((parent_id, child_id)) = find_first_col(layout_tree, current_id)
                        {
                            if let Some(prev_sibling) =
                                child_id.preceding_siblings(layout_tree).nth(1)
                            {
                                // Subtract one from the currently selected index...
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Col(row) = parent.get_mut() {
                                        row.last_selected_index =
                                            row.last_selected_index.saturating_sub(1);
                                    }
                                }

                                // Now descend downwards!
                                descend_to_leaf(layout_tree, prev_sibling)
                            } else {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_above(layout_tree, child_id)
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
                        if let Some((parent_id, child_id)) = find_first_col(layout_tree, current_id)
                        {
                            if let Some(prev_sibling) =
                                child_id.following_siblings(layout_tree).nth(1)
                            {
                                // Add one to the currently selected index...
                                if let Some(parent) = layout_tree.get_mut(parent_id) {
                                    if let LayoutNode::Col(row) = parent.get_mut() {
                                        row.last_selected_index += 1;
                                    }
                                }

                                // Now descend downwards!
                                descend_to_leaf(layout_tree, prev_sibling)
                            } else {
                                // Darn, we can't go further back! Recurse on this ID.
                                find_below(layout_tree, child_id)
                            }
                        } else {
                            // Failed, just return the current ID.
                            current_id
                        }
                    }
                    find_below(layout_tree, current_widget_id)
                }
            }
        }
    }
}

// ----- Old stuff below -----

#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    Up,
    // DOWN means scrolling down --- this usually INCREMENTS
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}

#[derive(Debug)]
pub enum CursorDirection {
    Left,
    Right,
}

/// AppScrollWidgetState deals with fields for a scrollable app's current state.
#[derive(Default)]
pub struct AppScrollWidgetState {
    pub current_scroll_position: usize,
    pub previous_scroll_position: usize,
    pub scroll_direction: ScrollDirection,
    pub table_state: TableState,
}

#[derive(PartialEq)]
pub enum KillSignal {
    Cancel,
    Kill(usize),
}

impl Default for KillSignal {
    #[cfg(target_family = "unix")]
    fn default() -> Self {
        KillSignal::Kill(15)
    }
    #[cfg(target_os = "windows")]
    fn default() -> Self {
        KillSignal::Kill(1)
    }
}

#[derive(Default)]
pub struct AppDeleteDialogState {
    pub is_showing_dd: bool,
    pub selected_signal: KillSignal,
    /// tl x, tl y, br x, br y, index/signal
    pub button_positions: Vec<(u16, u16, u16, u16, usize)>,
    pub keyboard_signal_select: usize,
    pub last_number_press: Option<Instant>,
    pub scroll_pos: usize,
}

pub struct AppHelpDialogState {
    pub is_showing_help: bool,
    pub scroll_state: ParagraphScrollState,
    pub index_shortcuts: Vec<u16>,
}

impl Default for AppHelpDialogState {
    fn default() -> Self {
        AppHelpDialogState {
            is_showing_help: false,
            scroll_state: ParagraphScrollState::default(),
            index_shortcuts: vec![0; constants::HELP_TEXT.len()],
        }
    }
}

/// Meant for canvas operations involving table column widths.
#[derive(Default)]
pub struct CanvasTableWidthState {
    pub desired_column_widths: Vec<u16>,
    pub calculated_column_widths: Vec<u16>,
}

pub struct BasicTableWidgetState {
    // Since this is intended (currently) to only be used for ONE widget, that's
    // how it's going to be written.  If we want to allow for multiple of these,
    // then we can expand outwards with a normal BasicTableState and a hashmap
    pub currently_displayed_widget_type: BottomWidgetType,
    pub currently_displayed_widget_id: u64,
    pub widget_id: i64,
    pub left_tlc: Option<(u16, u16)>,
    pub left_brc: Option<(u16, u16)>,
    pub right_tlc: Option<(u16, u16)>,
    pub right_brc: Option<(u16, u16)>,
}

#[derive(Default)]
pub struct ParagraphScrollState {
    pub current_scroll_index: u16,
    pub max_scroll_index: u16,
}
