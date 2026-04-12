//! Generalized input logic, to make it easier to reuse logic like unicode handling
//! and cursor movement.

use std::ops::Range;

use indexmap::IndexMap;
use unicode_segmentation::GraphemeCursor;

use crate::{app::CursorDirection, utils::int_hash::IntHashMap};

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
    size_mappings: IntHashMap<usize, Range<usize>>,
}

impl InputFieldState {
    /// Get a reference to the current query.
    #[inline]
    pub fn current_query(&self) -> &str {
        &self.current_search_query
    }
}
