//! Code for a temperature graph widget.

use std::time::Instant;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{Frame, layout::Rect};

use crate::app::{App, data::TemperatureType};

use super::EventHandled;

/// A temperature graph widget.
///
/// The current implementation of this uses tuine-style code; this will
/// become the standard later on for all widgets.
#[derive(Default)]
pub struct TemperatureGraph {
    current_display_time: u64,
    autohide_timer: Option<Instant>,
    current_max_temperature: u32,
    temperature_unit: TemperatureType,
}

impl TemperatureGraph {
    pub fn temperature_unit(mut self, unit: TemperatureType) -> Self {
        self.temperature_unit = unit;
        self
    }

    /// How this widget handles key events.
    ///
    /// TODO: This may merge with [`Self::handle_mouse_event`] in the future.
    pub fn handle_key_event(event: KeyEvent) -> EventHandled {
        EventHandled::NotHandled
    }

    /// How this widget handles mouse events.
    ///
    /// TODO: This may merge with [`Self::handle_key_event`] in the future.
    pub fn handle_mouse_event(event: MouseEvent) -> EventHandled {
        EventHandled::NotHandled
    }

    // /// How to lay out this widget in terms of sizing.
    //
    // For now, this is not implemented, and we will directly give sizes.
    // pub fn layout() {}

    /// How to draw this widget.
    ///
    /// This implementation is a bit of a hack/placeholder, in the future we don't want to bring in stuff like [`App`].
    pub fn draw(&mut self, frame: &mut Frame<'_>, app_state: &mut App, draw_area: Rect) {
        let data = &app_state.data_store.get_data().timeseries_data.temp;

        // Inspect the newest entry of each sensor to determine the current max temperature.
    }
}
