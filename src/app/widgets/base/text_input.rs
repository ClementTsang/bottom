use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use tui::layout::Rect;

use crate::app::{
    event::EventResult::{self},
    Component,
};

#[derive(Default)]
/// A single-line component for taking text inputs.
pub struct TextInput {
    text: String,
    cursor_index: usize,
    bounds: Rect,
}

impl TextInput {
    /// Creates a new [`TextInput`].
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn set_cursor(&mut self, new_cursor_index: usize) -> EventResult {
        if self.cursor_index == new_cursor_index {
            EventResult::NoRedraw
        } else {
            self.cursor_index = new_cursor_index;
            EventResult::Redraw
        }
    }

    fn move_back(&mut self, amount_to_subtract: usize) -> EventResult {
        self.set_cursor(self.cursor_index.saturating_sub(amount_to_subtract))
    }

    fn move_forward(&mut self, amount_to_add: usize) -> EventResult {
        let new_cursor = self.cursor_index + amount_to_add;
        if new_cursor >= self.text.len() {
            self.set_cursor(self.text.len() - 1)
        } else {
            self.set_cursor(new_cursor)
        }
    }

    fn clear_text(&mut self) -> EventResult {
        if self.text.is_empty() {
            EventResult::NoRedraw
        } else {
            self.text = String::default();
            self.cursor_index = 0;
            EventResult::Redraw
        }
    }

    fn move_word_forward(&mut self) -> EventResult {
        // TODO: Implement this
        EventResult::NoRedraw
    }

    fn move_word_back(&mut self) -> EventResult {
        // TODO: Implement this
        EventResult::NoRedraw
    }

    fn clear_previous_word(&mut self) -> EventResult {
        // TODO: Implement this
        EventResult::NoRedraw
    }

    fn clear_previous_grapheme(&mut self) -> EventResult {
        // TODO: Implement this
        EventResult::NoRedraw
    }

    pub fn update(&mut self, new_text: String) {
        self.text = new_text;

        if self.cursor_index >= self.text.len() {
            self.cursor_index = self.text.len() - 1;
        }
    }
}

impl Component for TextInput {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Left => self.move_back(1),
                KeyCode::Right => self.move_forward(1),
                KeyCode::Backspace => self.clear_previous_grapheme(),
                _ => EventResult::NoRedraw,
            }
        } else if let KeyModifiers::CONTROL = event.modifiers {
            match event.code {
                KeyCode::Char('a') => self.set_cursor(0),
                KeyCode::Char('e') => self.set_cursor(self.text.len()),
                KeyCode::Char('u') => self.clear_text(),
                KeyCode::Char('w') => self.clear_previous_word(),
                KeyCode::Char('h') => self.clear_previous_grapheme(),
                _ => EventResult::NoRedraw,
            }
        } else if let KeyModifiers::ALT = event.modifiers {
            match event.code {
                KeyCode::Char('b') => self.move_word_forward(),
                KeyCode::Char('f') => self.move_word_back(),
                _ => EventResult::NoRedraw,
            }
        } else {
            EventResult::NoRedraw
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        // We are assuming this is within bounds...

        let x = event.column;
        let widget_x = self.bounds.x;
        let new_cursor_index = usize::from(x.saturating_sub(widget_x));

        if new_cursor_index >= self.text.len() {
            self.cursor_index = self.text.len() - 1;
        } else {
            self.cursor_index = new_cursor_index;
        }

        EventResult::Redraw
    }
}
