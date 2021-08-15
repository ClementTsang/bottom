use std::time::Instant;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{layout::Rect, widgets::TableState};

use crate::{
    app::{event::EventResult, layout_manager::BottomWidgetType},
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

#[allow(unused_variables)]
pub trait Widget {
    type UpdateState;

    /// Handles a [`KeyEvent`].
    ///
    /// Defaults to returning [`EventResult::Continue`], indicating nothing should be done.
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        EventResult::Continue
    }

    /// Handles a [`MouseEvent`].  `x` and `y` represent *relative* mouse coordinates to the [`Widget`] - those should
    /// be used as opposed to the coordinates in the `event` unless you need absolute coordinates for some reason!
    ///
    /// Defaults to returning [`EventResult::Continue`], indicating nothing should be done.
    fn handle_mouse_event(&mut self, event: MouseEvent, x: u16, y: u16) -> EventResult {
        EventResult::Continue
    }

    /// Updates a [`Widget`].  Defaults to doing nothing.
    fn update(&mut self, update_state: Self::UpdateState) {}

    /// Returns a [`Widget`]'s bounding box, if possible.  Defaults to returning [`None`].
    fn bounding_box(&self) -> Option<Rect> {
        None
    }
}

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
