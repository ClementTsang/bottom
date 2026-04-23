//! A shared helper for drawing a vertical scroll bar.

use tui::{
    Frame,
    layout::Rect,
    style::Style,
    symbols::{self, scrollbar},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Arguments for [`draw_scroll_bar`].
pub struct ScrollBarArgs {
    /// Total number of items in the content.
    pub content_length: usize,
    /// Number of items that can be seen in the viewport.
    pub viewport_length: usize,
    /// Current scroll position in the list of items.
    pub position: usize,
    /// Style to be applied to the scrollbar.
    pub style: Style,
}

/// Draw a vertical scroll bar in `area`.
pub fn draw_scroll_bar(f: &mut Frame<'_>, area: Rect, args: ScrollBarArgs) {
    if args.content_length <= args.viewport_length || area.width == 0 || area.height == 0 {
        return;
    }

    const SYMBOLS: scrollbar::Set<'_> = scrollbar::Set {
        track: "",
        thumb: symbols::block::FULL,
        begin: "▲",
        end: "▼",
    };

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .style(args.style)
        .symbols(SYMBOLS);

    let mut state = ScrollbarState::new(args.content_length)
        .position(args.position)
        .viewport_content_length(args.viewport_length);

    f.render_stateful_widget(scrollbar, area, &mut state);
}
