use std::time::Instant;

use crossterm::event::{KeyEvent, MouseEvent};
use enum_dispatch::enum_dispatch;
use tui::{backend::Backend, layout::Rect, widgets::TableState, Frame};

use crate::{
    app::{
        event::{WidgetEventResult, SelectionAction},
        layout_manager::BottomWidgetType,
    },
    canvas::Painter,
    constants,
};

mod tui_widgets;

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

use super::data_farmer::DataCollection;

/// A trait for things that are drawn with state.
#[enum_dispatch]
#[allow(unused_variables)]
pub trait Component {
    /// Handles a [`KeyEvent`].
    ///
    /// Defaults to returning [`EventResult::NoRedraw`], indicating nothing should be done.
    fn handle_key_event(&mut self, event: KeyEvent) -> WidgetEventResult {
        WidgetEventResult::NoRedraw
    }

    /// Handles a [`MouseEvent`].
    ///
    /// Defaults to returning [`EventResult::Continue`], indicating nothing should be done.
    fn handle_mouse_event(&mut self, event: MouseEvent) -> WidgetEventResult {
        WidgetEventResult::NoRedraw
    }

    /// Returns a [`Component`]'s bounding box.  Note that these are defined in *global*, *absolute*
    /// coordinates.
    fn bounds(&self) -> Rect;

    /// Updates a [`Component`]s bounding box to `new_bounds`.
    fn set_bounds(&mut self, new_bounds: Rect);

    /// Returns whether a [`MouseEvent`] intersects a [`Component`].
    fn does_intersect_mouse(&self, event: &MouseEvent) -> bool {
        let x = event.column;
        let y = event.row;
        let rect = self.bounds();
        x >= rect.left() && x <= rect.right() && y >= rect.top() && y <= rect.bottom()
    }
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

    /// Draws a [`Widget`]. Defaults to doing nothing.
    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
    ) {
        // TODO: Remove the default implementation in the future!
    }

    /// How a [`Widget`] updates its internal displayed data. Defaults to doing nothing.
    fn update_data(&mut self, data_collection: &DataCollection) {}
}

/// The "main" widgets that are used by bottom to display information!
#[allow(clippy::large_enum_variant)]
#[enum_dispatch(Component, Widget)]
pub enum TmpBottomWidget {
    MemGraph,
    TempTable,
    DiskTable,
    CpuGraph,
    NetGraph,
    OldNetGraph,
    ProcessManager,
    BatteryTable,
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
