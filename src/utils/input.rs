//! Generalized input logic, to make it easier to reuse logic like unicode handling
//! and cursor movement.

use std::ops::Range;

use concat_string::concat_string;
use unicode_ellipsis::grapheme_width;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

use crate::{app::CursorDirection, utils::int_hash::IntIndexMap};

/// An input field's state.
pub struct InputFieldState {
    /// The search query itself, what is shown.
    current_search_query: String,

    /// The internal grapheme cursor to track the current location.
    grapheme_cursor: GraphemeCursor,

    /// The direction the cursor is heading at the moment. Modified
    /// by user actions, e.g. adding text moves it right, deleting
    /// text moves it left, the user scrolling changes it based
    /// on where they scroll, etc.
    cursor_direction: CursorDirection,

    /// Determines where we start _displaying_ the search based on
    /// the user's scroll. For example, if they move the cursor 5
    /// units to the right from 0, the index should be 5.
    display_start_char_index: usize,

    /// Used for internal tracking of _byte_ indices to the widths
    /// of the graphemes they represent. This is mostly used to cache
    /// and avoid having to re-calculate widths each time it needs to
    /// be accessed.
    ///
    /// Should always be updated after the search query updates in any way.
    size_mappings: IntIndexMap<usize, Range<usize>>,
}

impl Default for InputFieldState {
    fn default() -> Self {
        Self {
            current_search_query: String::default(),
            grapheme_cursor: GraphemeCursor::new(0, 0, true),
            cursor_direction: CursorDirection::Right,
            display_start_char_index: 0,
            size_mappings: IntIndexMap::default(),
        }
    }
}

impl InputFieldState {
    /// Get a reference to the current query.
    #[inline]
    pub fn current_query(&self) -> &str {
        &self.current_search_query
    }

    /// Get the current cursor index.
    #[inline]
    pub fn cursor_index(&self) -> usize {
        self.grapheme_cursor.cur_cursor()
    }

    /// Get the display start index.
    #[inline]
    pub fn display_start_index(&self) -> usize {
        self.display_start_char_index
    }

    /// Get the size mappings.
    ///
    /// TODO: We may want to reconsider exposing this, or expose something more encapsulated?
    #[inline]
    pub fn size_mappings(&self) -> &IntIndexMap<usize, Range<usize>> {
        &self.size_mappings
    }

    /// Reset the input field state.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Sets the starting grapheme index to draw from.
    ///
    /// TODO: This is kinda weird, we might want to decouple this in some way such that this
    /// is clear this only matters for drawing... but it also changes states...
    pub fn get_start_position(&mut self, available_width: usize, is_force_redraw: bool) {
        // Remember - the number of columns != the number of grapheme slots/sizes, you
        // cannot use index to determine this reliably!

        let start_index = if is_force_redraw {
            0
        } else {
            self.display_start_char_index
        };
        let cursor_index = self.cursor_index();

        if let Some(start_range) = self.size_mappings.get(&start_index) {
            let cursor_range = self
                .size_mappings
                .get(&cursor_index)
                .cloned()
                .unwrap_or_else(|| {
                    self.size_mappings
                        .last()
                        .map(|(_, r)| r.end..(r.end + 1))
                        .unwrap_or(start_range.end..(start_range.end + 1))
                });

            // Cases to handle in both cases:
            // - The current start index can show the cursor's word.
            // - The current start index cannot show the cursor's word.
            //
            // What differs is how we "scroll" based on the cursor movement direction.

            self.display_start_char_index = match self.cursor_direction {
                CursorDirection::Right => {
                    if start_range.start + available_width >= cursor_range.end {
                        // Use the current index.
                        start_index
                    } else if cursor_range.end >= available_width {
                        // If the current position is past the last visible element, skip until we
                        // see it.

                        let mut index = 0;
                        for i in 0..(cursor_index + 1) {
                            if let Some(r) = self.size_mappings.get(&i) {
                                if r.start + available_width >= cursor_range.end {
                                    index = i;
                                    break;
                                }
                            }
                        }

                        index
                    } else {
                        0
                    }
                }
                CursorDirection::Left => {
                    if cursor_range.start < start_range.end {
                        let mut index = 0;
                        for i in cursor_index..(self.current_search_query.len()) {
                            if let Some(r) = self.size_mappings.get(&i) {
                                if r.start + available_width >= cursor_range.end {
                                    index = i;
                                    break;
                                }
                            }
                        }
                        index
                    } else {
                        start_index
                    }
                }
            };
        } else {
            // If we fail here somehow, just reset to 0 index + scroll left.
            self.display_start_char_index = 0;
            self.cursor_direction = CursorDirection::Left;
        };
    }

    /// Move the cursor one _grapheme_ forward.
    pub(crate) fn walk_forward(&mut self) {
        let start_position = self.cursor_index();
        let chunk = &self.current_search_query[start_position..];

        match self.grapheme_cursor.next_boundary(chunk, start_position) {
            Ok(_) => {}
            Err(err) => match err {
                GraphemeIncomplete::PreContext(ctx) => {
                    // Provide the entire string as context. Not efficient but should resolve
                    // failures.
                    self.grapheme_cursor
                        .provide_context(&self.current_search_query[0..ctx], 0);

                    self.grapheme_cursor
                        .next_boundary(chunk, start_position)
                        .expect("another grapheme boundary should exist after the cursor with the provided context");
                }
                _ => panic!("{err:?}"),
            },
        }
    }

    /// Move the cursor one _grapheme_ backward.
    pub(crate) fn walk_backward(&mut self) {
        let start_position = self.cursor_index();
        let chunk = &self.current_search_query[..start_position];

        match self.grapheme_cursor.prev_boundary(chunk, 0) {
            Ok(_) => {}
            Err(err) => match err {
                GraphemeIncomplete::PreContext(ctx) => {
                    // Provide the entire string as context. Not efficient but should resolve
                    // failures.
                    self.grapheme_cursor
                        .provide_context(&self.current_search_query[0..ctx], 0);

                    self.grapheme_cursor
                        .prev_boundary(chunk, 0)
                        .expect("another grapheme boundary should exist before the cursor with the provided context");
                }
                _ => panic!("{err:?}"),
            },
        }
    }

    fn update_sizes(&mut self) {
        self.size_mappings.clear();
        let mut curr_offset = 0;
        for (index, grapheme) in
            UnicodeSegmentation::grapheme_indices(self.current_search_query.as_str(), true)
        {
            let width = grapheme_width(grapheme);
            let end = curr_offset + width;

            self.size_mappings.insert(index, curr_offset..end);

            curr_offset = end;
        }

        self.size_mappings.shrink_to_fit();
    }

    /// Delete whatever the cursor is currently highlighting, if anything. This is analogous to pressing `Delete`.
    pub fn delete_at_cursor(&mut self) {
        let current_cursor = self.cursor_index();
        if current_cursor < self.current_search_query.len() {
            self.walk_forward();
            let new_cursor = self.cursor_index();

            let _ = self.current_search_query.drain(current_cursor..new_cursor);

            self.grapheme_cursor =
                GraphemeCursor::new(current_cursor, self.current_search_query.len(), true);

            self.update_sizes();
        }
    }

    /// Delete what is _behind_ the cursor. This is analogous to pressing `Backspace`.
    pub fn delete_behind_cursor(&mut self) {
        let current_cursor = self.cursor_index();

        if current_cursor > 0 {
            self.walk_backward();
            let new_cursor = self.cursor_index();

            // Remove the indices in between.
            let _ = self.current_search_query.drain(new_cursor..current_cursor);

            self.grapheme_cursor =
                GraphemeCursor::new(new_cursor, self.current_search_query.len(), true);

            self.cursor_direction = CursorDirection::Left;

            self.update_sizes();
        }
    }

    /// Move the cursor left one unit if possible.
    pub fn move_left(&mut self) {
        let current_cursor = self.cursor_index();
        self.walk_backward();
        if self.cursor_index() < current_cursor {
            self.cursor_direction = CursorDirection::Left;
        }
    }

    /// Move the cursor right one unit if possible.
    pub fn move_right(&mut self) {
        let current_cursor = self.cursor_index();
        self.walk_forward();
        if self.cursor_index() > current_cursor {
            self.cursor_direction = CursorDirection::Right;
        }
    }

    /// Move the cursor to the start.
    pub fn skip_to_beginning(&mut self) {
        self.grapheme_cursor = GraphemeCursor::new(0, self.current_search_query.len(), true);
        self.cursor_direction = CursorDirection::Left;
    }

    /// Move the cursor to the end.
    pub fn skip_to_end(&mut self) {
        let query_len = self.current_search_query.len();
        self.grapheme_cursor = GraphemeCursor::new(query_len, query_len, true);
        self.cursor_direction = CursorDirection::Right;
    }

    /// Delete the previous "word".
    pub fn delete_previous_word(&mut self) {
        // Traverse backwards from the current cursor location until you hit
        // non-whitespace characters, then continue to traverse (and
        // delete) backwards until you hit a whitespace character.  Halt.

        // So... first, let's get our current cursor position in terms of char
        // indices. This is the "end" index we care about.
        let current_cursor = self.cursor_index();

        // Then, let's crawl backwards until we hit our location, and store the
        // "head"...
        let query = self.current_query();
        let mut start_index = 0;
        let mut saw_non_whitespace = false;

        for (itx, c) in query
            .chars()
            .rev()
            .enumerate()
            .skip(query.len() - current_cursor)
        {
            if c.is_whitespace() {
                if saw_non_whitespace {
                    start_index = query.len() - itx;
                    break;
                }
            } else {
                saw_non_whitespace = true;
            }
        }

        let _ = self.current_search_query.drain(start_index..current_cursor);

        self.grapheme_cursor =
            GraphemeCursor::new(start_index, self.current_search_query.len(), true);

        self.cursor_direction = CursorDirection::Left;

        self.update_sizes();
    }

    /// Insert a single [`char`].
    pub fn insert_char(&mut self, ch: char) {
        self.current_search_query.insert(self.cursor_index(), ch);

        self.grapheme_cursor =
            GraphemeCursor::new(self.cursor_index(), self.current_search_query.len(), true);

        self.walk_forward();
        self.cursor_direction = CursorDirection::Right;

        self.update_sizes();
    }

    /// Insert a [`String`].
    pub fn insert_string(&mut self, s: String) {
        // Partially copy-pasted from the single-char variant; should probably clean up
        // this process in the future. In particular, encapsulate this entire
        // logic and add some tests to make it less potentially error-prone.

        let left_bound = self.cursor_index();

        let curr_query = &mut self.current_search_query;
        let (left, right) = curr_query.split_at(left_bound);
        let num_runes = UnicodeSegmentation::graphemes(s.as_str(), true).count();

        *curr_query = concat_string!(left, s, right);

        self.grapheme_cursor = GraphemeCursor::new(left_bound, curr_query.len(), true);

        // TODO: We could probably do something smarter here...
        for _ in 0..num_runes {
            self.walk_forward();
        }

        self.cursor_direction = CursorDirection::Right;

        self.update_sizes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn move_right(state: &mut InputFieldState) {
        state.walk_forward();
        state.cursor_direction = CursorDirection::Right;
    }

    fn move_left(state: &mut InputFieldState) {
        state.walk_backward();
        state.cursor_direction = CursorDirection::Left;
    }

    #[test]
    fn search_cursor_moves() {
        let mut state = InputFieldState::default();
        state.current_search_query = "Hi, 你好! 🇦🇶".to_string();
        state.grapheme_cursor = GraphemeCursor::new(0, state.current_search_query.len(), true);
        state.update_sizes();

        // Moving right.
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 0);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 1);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 2);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 3);
        assert_eq!(state.display_start_char_index, 0);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 4);
        assert_eq!(state.display_start_char_index, 2);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 7);
        assert_eq!(state.display_start_char_index, 4);

        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 10);
        assert_eq!(state.display_start_char_index, 7);

        move_right(&mut state);
        move_right(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 12);
        assert_eq!(state.display_start_char_index, 10);

        // Moving left.
        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 11);
        assert_eq!(state.display_start_char_index, 10);

        move_left(&mut state);
        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 7);
        assert_eq!(state.display_start_char_index, 7);

        move_left(&mut state);
        move_left(&mut state);
        move_left(&mut state);
        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 1);
        assert_eq!(state.display_start_char_index, 1);

        move_left(&mut state);
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 0);
        assert_eq!(state.display_start_char_index, 0);
    }
}
