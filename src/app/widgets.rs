use std::{fmt::Debug, time::Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use enum_dispatch::enum_dispatch;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    app::event::{ComponentEventResult, SelectionAction},
    canvas::Painter,
    options::layout_options::WidgetLayoutRule,
};

mod tui_stuff;

pub mod base;
pub use base::*;

pub mod dialogs;
pub use dialogs::*;

pub mod bottom_widgets;
pub use bottom_widgets::*;

use self::tui_stuff::BlockBuilder;

use super::data_farmer::DataCollection;

/// A trait for things that are drawn with state.
#[enum_dispatch]
#[allow(unused_variables)]
pub trait Component {
    /// Handles a [`KeyEvent`].
    ///
    /// Defaults to returning [`ComponentEventResult::Unhandled`], indicating the component does not handle this event.
    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        ComponentEventResult::Unhandled
    }

    /// Handles a [`MouseEvent`].
    ///
    /// Defaults to returning [`ComponentEventResult::Unhandled`], indicating the component does not handle this event.
    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        ComponentEventResult::Unhandled
    }

    /// Returns a [`Component`]'s bounding box.  Note that these are defined in *global*, *absolute*
    /// coordinates.
    fn bounds(&self) -> Rect;

    /// Updates a [`Component`]'s bounding box to `new_bounds`.
    fn set_bounds(&mut self, new_bounds: Rect);

    /// Returns a [`Component`]'s bounding box, *including the border*. Defaults to just returning the normal bounds.
    ///   Note that these are defined in *global*, *absolute* coordinates.
    fn border_bounds(&self) -> Rect {
        self.bounds()
    }

    /// Updates a [`Component`]'s bounding box to `new_bounds`.  Defaults to just setting the normal bounds.
    fn set_border_bounds(&mut self, new_bounds: Rect) {
        self.set_bounds(new_bounds);
    }

    /// Returns whether a [`MouseEvent`] intersects a [`Component`]'s bounds.
    fn does_bounds_intersect_mouse(&self, event: &MouseEvent) -> bool {
        let x = event.column;
        let y = event.row;
        let bounds = self.bounds();

        does_bound_intersect_coordinate(x, y, bounds)
    }

    /// Returns whether a [`MouseEvent`] intersects a [`Component`]'s bounds, including any borders, if there are.
    fn does_border_intersect_mouse(&self, event: &MouseEvent) -> bool {
        let x = event.column;
        let y = event.row;
        let bounds = self.border_bounds();

        does_bound_intersect_coordinate(x, y, bounds)
    }
}

pub fn does_bound_intersect_coordinate(x: u16, y: u16, bounds: Rect) -> bool {
    x >= bounds.left() && x < bounds.right() && y >= bounds.top() && y < bounds.bottom()
}

/// A trait for actual fully-fledged widgets to be displayed in bottom.
#[enum_dispatch]
#[allow(unused_variables)]
pub trait Widget {
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

    /// Returns a [`Widget`]'s "pretty" display name.
    fn get_pretty_name(&self) -> &'static str;

    /// Returns a new [`BlockBuilder`], which can become a [`tui::widgets::Block`] if [`BlockBuilder::build`] is called.
    /// The default implementation builds a [`Block`] that has all 4 borders with no selection or expansion.
    fn block(&self) -> BlockBuilder {
        BlockBuilder::new(self.get_pretty_name())
    }

    /// Draws a [`Widget`]. The default implementation draws nothing.
    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
        expanded: bool,
    ) {
    }

    /// How a [`Widget`] updates its internal data that'll be displayed. Called after every data harvest.
    /// The default implementation does nothing with the data.
    fn update_data(&mut self, data_collection: &DataCollection) {}

    /// Returns the desired width from the [`Widget`].
    fn width(&self) -> WidgetLayoutRule;

    /// Returns the desired height from the [`Widget`].
    fn height(&self) -> WidgetLayoutRule;

    /// Returns whether this [`Widget`] can be selected. The default implementation returns [`SelectableType::Selectable`].
    fn selectable_type(&self) -> SelectableType {
        SelectableType::Selectable
    }

    /// Resets state in a [`Widget`]; used when a reset signal is given. The default implementation does nothing.
    fn reset(&mut self) {}
}

/// Whether a widget can be selected, not selected, or redirected upon selection.
pub enum SelectableType {
    Selectable,
    Unselectable,
}

/// The "main" widgets that are used by bottom to display information!
#[allow(clippy::large_enum_variant)]
#[enum_dispatch(Component, Widget)]
pub enum OldBottomWidget {
    MemGraph,
    TempTable,
    DiskTable,
    CpuGraph,
    NetGraph,
    OldNetGraph,
    ProcessManager,
    BatteryTable,
    BasicCpu,
    BasicMem,
    BasicNet,
    Carousel,
    Empty,
}

impl Debug for OldBottomWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemGraph(_) => write!(f, "MemGraph"),
            Self::TempTable(_) => write!(f, "TempTable"),
            Self::DiskTable(_) => write!(f, "DiskTable"),
            Self::CpuGraph(_) => write!(f, "CpuGraph"),
            Self::NetGraph(_) => write!(f, "NetGraph"),
            Self::OldNetGraph(_) => write!(f, "OldNetGraph"),
            Self::ProcessManager(_) => write!(f, "ProcessManager"),
            Self::BatteryTable(_) => write!(f, "BatteryTable"),
            Self::BasicCpu(_) => write!(f, "BasicCpu"),
            Self::BasicMem(_) => write!(f, "BasicMem"),
            Self::BasicNet(_) => write!(f, "BasicNet"),
            Self::Carousel(_) => write!(f, "Carousel"),
            Self::Empty(_) => write!(f, "Empty"),
        }
    }
}

// ----- FIXME: Delete the old stuff below -----

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
