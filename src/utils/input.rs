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
    fn walk_forward(&mut self) {
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
    fn walk_backward(&mut self) {
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

    /// Tests that inserting ASCII chars appends them to the query and advances the cursor by 1 byte each.
    #[test]
    fn insert_char_ascii() {
        let mut state = InputFieldState::default();

        state.insert_char('H');
        assert_eq!(state.current_query(), "H");
        assert_eq!(state.cursor_index(), 1);

        state.insert_char('i');
        assert_eq!(state.current_query(), "Hi");
        assert_eq!(state.cursor_index(), 2);
    }

    /// Tests that inserting multi-byte Unicode chars (e.g. CJK) advances the cursor by the correct byte width.
    #[test]
    fn insert_char_unicode() {
        let mut state = InputFieldState::default();

        state.insert_char('你'); // 3-byte UTF-8
        assert_eq!(state.current_query(), "你");
        assert_eq!(state.cursor_index(), 3);

        state.insert_char('好');
        assert_eq!(state.current_query(), "你好");
        assert_eq!(state.cursor_index(), 6);
    }

    /// Tests that inserting a 4-byte emoji advances the cursor to byte offset 4.
    #[test]
    fn insert_char_emoji() {
        let mut state = InputFieldState::default();

        state.insert_char('🦀'); // 4-byte UTF-8
        assert_eq!(state.current_query(), "🦀");
        assert_eq!(state.cursor_index(), 4);
    }

    /// Tests that inserting a char at a mid-string cursor position shifts the rest of the string
    /// right and places the cursor immediately after the newly inserted char.
    #[test]
    fn insert_char_at_middle() {
        let mut state = InputFieldState::default();
        state.insert_char('H');
        state.insert_char('i');
        state.insert_char('!');

        // Move back to before '!'
        state.move_left();
        assert_eq!(state.cursor_index(), 2);

        state.insert_char(' ');
        assert_eq!(state.current_query(), "Hi !");
        assert_eq!(state.cursor_index(), 3);
    }

    /// Tests that inserting an ASCII string at once places the entire string in the query
    /// and lands the cursor at the end.
    #[test]
    fn insert_string_ascii() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());
        assert_eq!(state.current_query(), "Hello");
        assert_eq!(state.cursor_index(), 5);
    }

    /// Tests that inserting a mixed multi-byte + emoji string reflects the correct total byte length
    /// in the cursor position.
    #[test]
    fn insert_string_unicode() {
        let mut state = InputFieldState::default();
        state.insert_string("你好🇨🇦🦀".to_string());
        assert_eq!(state.current_query(), "你好🇨🇦🦀");
        // '你'=3, '好'=3, '🦀'=4, '🇨🇦'=8, so 14 bytes
        assert_eq!(state.cursor_index(), 18);
    }

    /// Tests that inserting a string at position 0 prepends it, leaving the cursor after the
    /// inserted portion and the rest of the original string intact.
    #[test]
    fn insert_string_at_middle() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());
        state.skip_to_beginning();
        state.insert_string("Say ".to_string());
        assert_eq!(state.current_query(), "Say Hello");
        assert_eq!(state.cursor_index(), 4);
    }

    /// Tests that [`InputFieldState::delete_at_cursor`] removes the grapheme under the cursor
    /// without moving the cursor position.
    #[test]
    fn delete_at_cursor_basic() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());
        state.skip_to_beginning();

        state.delete_at_cursor(); // removes 'H'
        assert_eq!(state.current_query(), "ello");
        assert_eq!(state.cursor_index(), 0);

        state.delete_at_cursor(); // removes 'e'
        assert_eq!(state.current_query(), "llo");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::delete_at_cursor`] is a no-op when the cursor is already
    /// at the end of the string.
    #[test]
    fn delete_at_cursor_at_end_is_noop() {
        let mut state = InputFieldState::default();
        state.insert_string("Hi".to_string());
        // cursor is already at end after inserting

        state.delete_at_cursor();
        assert_eq!(state.current_query(), "Hi");
        assert_eq!(state.cursor_index(), 2);
    }

    /// Tests that [`InputFieldState::delete_at_cursor`] correctly removes a full multi-byte
    /// grapheme cluster in one operation.
    #[test]
    fn delete_at_cursor_unicode() {
        let mut state = InputFieldState::default();
        state.insert_string("你好".to_string());
        state.skip_to_beginning();

        state.delete_at_cursor(); // removes '你' (3 bytes)
        assert_eq!(state.current_query(), "好");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::delete_behind_cursor`] removes the grapheme immediately
    /// before the cursor and moves the cursor back accordingly.
    #[test]
    fn delete_behind_cursor_basic() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());

        state.delete_behind_cursor(); // removes 'o'
        assert_eq!(state.current_query(), "Hell");
        assert_eq!(state.cursor_index(), 4);

        state.delete_behind_cursor(); // removes 'l'
        assert_eq!(state.current_query(), "Hel");
        assert_eq!(state.cursor_index(), 3);
    }

    /// Tests that [`InputFieldState::delete_behind_cursor`] is a no-op when the cursor is at
    /// position 0.
    #[test]
    fn delete_behind_cursor_at_start_is_noop() {
        let mut state = InputFieldState::default();
        state.insert_string("Hi".to_string());
        state.skip_to_beginning();

        state.delete_behind_cursor();
        assert_eq!(state.current_query(), "Hi");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::delete_behind_cursor`] correctly removes multi-byte grapheme
    /// clusters one at a time, adjusting the byte cursor each time.
    #[test]
    fn delete_behind_cursor_unicode() {
        let mut state = InputFieldState::default();
        state.insert_string("你好".to_string());

        state.delete_behind_cursor(); // removes '好' (3 bytes)
        assert_eq!(state.current_query(), "你");
        assert_eq!(state.cursor_index(), 3);

        state.delete_behind_cursor(); // removes '你' (3 bytes)
        assert_eq!(state.current_query(), "");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::move_left`] and [`InputFieldState::move_right`] step one
    /// byte per ASCII grapheme and are clamped at both ends of the string.
    #[test]
    fn move_left_right_ascii() {
        let mut state = InputFieldState::default();
        state.insert_string("abc".to_string());
        assert_eq!(state.cursor_index(), 3);

        state.move_left();
        assert_eq!(state.cursor_index(), 2);
        state.move_left();
        assert_eq!(state.cursor_index(), 1);
        state.move_left();
        assert_eq!(state.cursor_index(), 0);

        // At the start — no further movement
        state.move_left();
        assert_eq!(state.cursor_index(), 0);

        state.move_right();
        assert_eq!(state.cursor_index(), 1);
        state.move_right();
        assert_eq!(state.cursor_index(), 2);
        state.move_right();
        assert_eq!(state.cursor_index(), 3);

        // At the end — no further movement
        state.move_right();
        assert_eq!(state.cursor_index(), 3);
    }

    /// Tests that [`InputFieldState::move_left`] and [`InputFieldState::move_right`] jump the
    /// full byte width of each grapheme, including multi-byte CJK characters.
    #[test]
    fn move_left_right_unicode() {
        let mut state = InputFieldState::default();
        state.insert_string("a你b".to_string()); // 1 + 3 + 1 = 5 bytes
        assert_eq!(state.cursor_index(), 5);

        state.move_left(); // over 'b' (1 byte)
        assert_eq!(state.cursor_index(), 4);

        state.move_left(); // over '你' (3 bytes)
        assert_eq!(state.cursor_index(), 1);

        state.move_left(); // over 'a' (1 byte)
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::skip_to_beginning`] moves the cursor to byte 0 and
    /// [`InputFieldState::skip_to_end`] moves it past the last byte.
    #[test]
    fn skip_to_beginning_and_end() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());

        state.skip_to_beginning();
        assert_eq!(state.cursor_index(), 0);

        state.skip_to_end();
        assert_eq!(state.cursor_index(), 5);
    }

    /// Tests that [`InputFieldState::skip_to_beginning`] sets the cursor direction to
    /// [`CursorDirection::Left`] so that scrolling logic behaves correctly.
    #[test]
    fn skip_to_beginning_sets_direction_left() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());
        state.skip_to_beginning();
        assert!(matches!(state.cursor_direction, CursorDirection::Left));
    }

    /// Tests that [`InputFieldState::skip_to_end`] sets the cursor direction to
    /// [`CursorDirection::Right`] so that scrolling logic behaves correctly.
    #[test]
    fn skip_to_end_sets_direction_right() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());
        state.skip_to_beginning();
        state.skip_to_end();
        assert!(matches!(state.cursor_direction, CursorDirection::Right));
    }

    /// Tests that [`InputFieldState::delete_previous_word`] removes a single word with no
    /// preceding whitespace, leaving an empty query.
    #[test]
    fn delete_previous_word_single_word() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello".to_string());

        state.delete_previous_word();
        assert_eq!(state.current_query(), "");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::delete_previous_word`] removes exactly one word per call
    /// when the query contains multiple words separated by a space.
    #[test]
    fn delete_previous_word_two_words() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello World".to_string());

        state.delete_previous_word(); // deletes "World"
        assert_eq!(state.current_query(), "Hello ");
        assert_eq!(state.cursor_index(), 6);

        state.delete_previous_word(); // deletes "Hello "
        assert_eq!(state.current_query(), "");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::delete_previous_word`] skips trailing whitespace before
    /// deleting the preceding non-whitespace word.
    #[test]
    fn delete_previous_word_trailing_spaces() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello   ".to_string()); // trailing spaces

        // Should skip spaces first, then delete "Hello"
        state.delete_previous_word();
        assert_eq!(state.current_query(), "");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::delete_previous_word`] only removes the portion of a word
    /// that lies behind the cursor when the cursor is mid-word.
    #[test]
    fn delete_previous_word_from_middle() {
        let mut state = InputFieldState::default();
        state.insert_string("Hello World".to_string());
        state.skip_to_beginning();
        // Advance 3 chars ('H','e','l')
        state.move_right();
        state.move_right();
        state.move_right();
        assert_eq!(state.cursor_index(), 3);

        state.delete_previous_word(); // deletes "Hel"
        assert_eq!(state.current_query(), "lo World");
        assert_eq!(state.cursor_index(), 0);
    }

    /// Tests that [`InputFieldState::reset`] returns all fields — query, cursor position,
    /// display start index, and size mappings — to their default values.
    #[test]
    fn reset_clears_state() {
        let mut state = InputFieldState::default();
        state.insert_string("Something".to_string());
        assert_eq!(state.current_query(), "Something");

        state.reset();
        assert_eq!(state.current_query(), "");
        assert_eq!(state.cursor_index(), 0);
        assert_eq!(state.display_start_index(), 0);
        assert!(state.size_mappings().is_empty());
    }

    /// Tests that the cursor moves correctly when moving left and right.
    /// In particular, tests [`InputFieldState::move_left`] and [`InputFieldState::move_right`].
    #[test]
    fn search_cursor_moves() {
        let mut state = InputFieldState::default();
        state.insert_string("Hi, 你好! 🇨🇦".to_string());
        state.skip_to_beginning();

        // Moving right.
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 0);
        assert_eq!(state.display_start_char_index, 0);

        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 1);
        assert_eq!(state.display_start_char_index, 0);

        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 2);
        assert_eq!(state.display_start_char_index, 0);

        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 3);
        assert_eq!(state.display_start_char_index, 0);

        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 4);
        assert_eq!(state.display_start_char_index, 2);

        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 7);
        assert_eq!(state.display_start_char_index, 4);

        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 10);
        assert_eq!(state.display_start_char_index, 7);

        state.move_right();
        state.move_right();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 12);
        assert_eq!(state.display_start_char_index, 10);

        // Moving left.
        state.move_left();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 11);
        assert_eq!(state.display_start_char_index, 10);

        state.move_left();
        state.move_left();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 7);
        assert_eq!(state.display_start_char_index, 7);

        state.move_left();
        state.move_left();
        state.move_left();
        state.move_left();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 1);
        assert_eq!(state.display_start_char_index, 1);

        state.move_left();
        state.get_start_position(4, false);
        assert_eq!(state.grapheme_cursor.cur_cursor(), 0);
        assert_eq!(state.display_start_char_index, 0);
    }
}
