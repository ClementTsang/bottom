use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use itertools::Itertools;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{
        event::{
            ComponentEventResult::{self},
            ReturnSignal,
        },
        Component,
    },
    canvas::Painter,
};

enum CursorDirection {
    Left,
    Right,
}

/// We save the previous window index for future reference, but we must invalidate if the area changes.
#[derive(Default)]
struct WindowIndex {
    start_index: usize,
    cached_area: Rect,
}

/// A single-line component for taking text inputs.
pub struct TextInput {
    text: String,
    bounds: Rect,
    cursor: GraphemeCursor,

    cursor_direction: CursorDirection,
    window_index: WindowIndex,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            text: Default::default(),
            bounds: Default::default(),
            cursor: GraphemeCursor::new(0, 0, true),
            cursor_direction: CursorDirection::Right,
            window_index: Default::default(),
        }
    }
}

impl TextInput {
    /// Returns a reference to the current query.
    pub fn query(&self) -> &str {
        &self.text
    }

    fn move_back(&mut self) -> usize {
        let current_position = self.cursor.cur_cursor();
        if let Ok(Some(new_position)) = self.cursor.prev_boundary(&self.text[..current_position], 0)
        {
            self.cursor_direction = CursorDirection::Left;
            new_position
        } else {
            current_position
        }
    }

    fn move_forward(&mut self) -> usize {
        let current_position = self.cursor.cur_cursor();
        if let Ok(Some(new_position)) = self
            .cursor
            .next_boundary(&self.text[current_position..], current_position)
        {
            self.cursor_direction = CursorDirection::Right;
            new_position
        } else {
            current_position
        }
    }

    fn clear_text(&mut self) -> ComponentEventResult {
        if self.text.is_empty() {
            ComponentEventResult::NoRedraw
        } else {
            self.text = String::default();
            self.cursor = GraphemeCursor::new(0, 0, true);
            self.window_index = Default::default();
            self.cursor_direction = CursorDirection::Left;
            ComponentEventResult::Signal(ReturnSignal::Update)
        }
    }

    fn move_word_forward(&mut self) -> ComponentEventResult {
        let current_index = self.cursor.cur_cursor();

        if current_index < self.text.len() {
            for (index, _word) in self.text[current_index..].unicode_word_indices() {
                if index > 0 {
                    self.cursor.set_cursor(index + current_index);
                    self.cursor_direction = CursorDirection::Right;
                    return ComponentEventResult::Redraw;
                }
            }
            self.cursor.set_cursor(self.text.len());
        }

        ComponentEventResult::Redraw
    }

    fn move_word_back(&mut self) -> ComponentEventResult {
        let current_index = self.cursor.cur_cursor();

        for (index, _word) in self.text[..current_index].unicode_word_indices().rev() {
            if index < current_index {
                self.cursor.set_cursor(index);
                self.cursor_direction = CursorDirection::Left;
                return ComponentEventResult::Redraw;
            }
        }

        ComponentEventResult::NoRedraw
    }

    fn clear_word_from_cursor(&mut self) -> ComponentEventResult {
        // Fairly simple logic - create the word index iterator, skip the word that intersects with the current
        // cursor location, draw the rest, update the string.
        let current_index = self.cursor.cur_cursor();
        let mut start_delete_index = 0;
        let mut saw_non_whitespace = false;
        for (index, word) in self.text[..current_index].split_word_bound_indices().rev() {
            if word.trim().is_empty() {
                if saw_non_whitespace {
                    // It's whitespace!  Stop!
                    break;
                }
            } else {
                saw_non_whitespace = true;
                start_delete_index = index;
            }
        }

        if start_delete_index == current_index {
            ComponentEventResult::NoRedraw
        } else {
            self.text.drain(start_delete_index..current_index);
            self.cursor = GraphemeCursor::new(start_delete_index, self.text.len(), true);
            self.cursor_direction = CursorDirection::Left;
            ComponentEventResult::Signal(ReturnSignal::Update)
        }
    }

    fn clear_previous_grapheme(&mut self) -> ComponentEventResult {
        let current_index = self.cursor.cur_cursor();

        if current_index > 0 {
            let new_index = self.move_back();
            self.text.drain(new_index..current_index);

            self.cursor = GraphemeCursor::new(new_index, self.text.len(), true);
            self.cursor_direction = CursorDirection::Left;

            ComponentEventResult::Signal(ReturnSignal::Update)
        } else {
            ComponentEventResult::NoRedraw
        }
    }

    fn clear_current_grapheme(&mut self) -> ComponentEventResult {
        let current_index = self.cursor.cur_cursor();

        if current_index < self.text.len() {
            let current_index_bound = self.move_forward();
            self.text.drain(current_index..current_index_bound);

            self.cursor = GraphemeCursor::new(current_index, self.text.len(), true);
            self.cursor_direction = CursorDirection::Left;

            ComponentEventResult::Signal(ReturnSignal::Update)
        } else {
            ComponentEventResult::NoRedraw
        }
    }

    fn insert_character(&mut self, c: char) -> ComponentEventResult {
        let current_index = self.cursor.cur_cursor();
        self.text.insert(current_index, c);
        self.cursor = GraphemeCursor::new(current_index, self.text.len(), true);
        self.move_forward();

        ComponentEventResult::Signal(ReturnSignal::Update)
    }

    /// Updates the window indexes and returns the start index.
    pub fn update_window_index(&mut self, num_visible_columns: usize) -> usize {
        if self.window_index.cached_area != self.bounds {
            self.window_index.start_index = 0;
            self.window_index.cached_area = self.bounds;
        }

        let current_index = self.cursor.cur_cursor();

        match self.cursor_direction {
            CursorDirection::Right => {
                if current_index < self.window_index.start_index + num_visible_columns {
                    self.window_index.start_index
                } else if current_index >= num_visible_columns {
                    self.window_index.start_index = current_index - num_visible_columns + 1;
                    self.window_index.start_index
                } else {
                    0
                }
            }
            CursorDirection::Left => {
                if current_index <= self.window_index.start_index {
                    self.window_index.start_index = current_index;
                } else if current_index >= self.window_index.start_index + num_visible_columns {
                    self.window_index.start_index = current_index - num_visible_columns + 1;
                }
                self.window_index.start_index
            }
        }
    }

    /// Draws the [`TextInput`] on screen.
    pub fn draw_text_input<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, selected: bool,
    ) {
        self.set_bounds(area);

        const SEARCH_PROMPT: &str = "> ";
        let prompt = if area.width > 5 { SEARCH_PROMPT } else { "" };

        let num_visible_columns = area.width as usize - prompt.len();
        let start_position = self.update_window_index(num_visible_columns);
        let cursor_start = self.cursor.cur_cursor();

        let mut graphemes = self.text.grapheme_indices(true).peekable();
        let mut current_grapheme_posn = 0;

        graphemes
            .peeking_take_while(|(index, _)| *index < start_position)
            .for_each(|(_, s)| {
                current_grapheme_posn += UnicodeWidthStr::width(s);
            });

        let before_cursor = graphemes
            .peeking_take_while(|(index, _)| *index < cursor_start)
            .map(|(_, grapheme)| grapheme)
            .collect::<String>();

        let cursor = graphemes
            .next()
            .map(|(_, grapheme)| grapheme)
            .unwrap_or(" ");

        let after_cursor = graphemes.map(|(_, grapheme)| grapheme).collect::<String>();

        // FIXME: [AFTER REFACTOR] This is NOT done!  This is an incomplete (but kinda working) implementation, for now.

        let search_text = vec![Spans::from(vec![
            Span::styled(
                prompt,
                if selected {
                    painter.colours.highlighted_border_style
                } else {
                    painter.colours.text_style
                },
            ),
            Span::styled(before_cursor, painter.colours.text_style),
            Span::styled(cursor, painter.colours.currently_selected_text_style),
            Span::styled(after_cursor, painter.colours.text_style),
        ])];

        f.render_widget(
            Paragraph::new(search_text)
                .style(painter.colours.text_style)
                .alignment(Alignment::Left),
            area,
        );
    }

    fn move_left(&mut self) -> ComponentEventResult {
        let original_cursor = self.cursor.cur_cursor();
        if self.move_back() == original_cursor {
            ComponentEventResult::NoRedraw
        } else {
            ComponentEventResult::Redraw
        }
    }

    fn move_right(&mut self) -> ComponentEventResult {
        let original_cursor = self.cursor.cur_cursor();
        if self.move_forward() == original_cursor {
            ComponentEventResult::NoRedraw
        } else {
            ComponentEventResult::Redraw
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

    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT {
            match event.code {
                KeyCode::Left => self.move_left(),
                KeyCode::Right => self.move_right(),
                KeyCode::Backspace => self.clear_previous_grapheme(),
                KeyCode::Delete => self.clear_current_grapheme(),
                KeyCode::Char(c) => self.insert_character(c),
                _ => ComponentEventResult::Unhandled,
            }
        } else if let KeyModifiers::CONTROL = event.modifiers {
            match event.code {
                KeyCode::Char('a') => {
                    let prev_index = self.cursor.cur_cursor();
                    self.cursor.set_cursor(0);
                    if self.cursor.cur_cursor() == prev_index {
                        ComponentEventResult::NoRedraw
                    } else {
                        ComponentEventResult::Redraw
                    }
                }
                KeyCode::Char('e') => {
                    let prev_index = self.cursor.cur_cursor();
                    self.cursor.set_cursor(self.text.len());
                    if self.cursor.cur_cursor() == prev_index {
                        ComponentEventResult::NoRedraw
                    } else {
                        ComponentEventResult::Redraw
                    }
                }
                KeyCode::Char('u') => self.clear_text(),
                KeyCode::Char('w') => self.clear_word_from_cursor(),
                KeyCode::Char('h') => self.clear_previous_grapheme(),
                _ => ComponentEventResult::Unhandled,
            }
        } else if let KeyModifiers::ALT = event.modifiers {
            match event.code {
                KeyCode::Char('b') => self.move_word_back(),
                KeyCode::Char('f') => self.move_word_forward(),
                KeyCode::Char('h') => self.move_left(),
                KeyCode::Char('l') => self.move_right(),
                _ => ComponentEventResult::Unhandled,
            }
        } else {
            ComponentEventResult::Unhandled
        }
    }

    fn handle_mouse_event(&mut self, _event: MouseEvent) -> ComponentEventResult {
        // We are assuming this is within bounds...

        // TODO: [Feature] Add mouse input for text input cursor
        // let x = event.column;
        // let widget_x = self.bounds.x + 2;
        // if x >= widget_x {
        //     ComponentEventResult::Redraw
        // } else {
        //     ComponentEventResult::NoRedraw
        // }
        ComponentEventResult::Unhandled
    }
}
