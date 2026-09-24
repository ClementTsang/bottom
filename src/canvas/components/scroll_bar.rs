//! A shared helper for drawing a vertical scroll bar.

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::{self, scrollbar},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
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

/// Returns a [`Rect`] for a vertical scroll bar drawn just inside the right
/// border of a dialog block.
pub fn dialog_scroll_bar_area(block_area: Rect) -> Rect {
    Rect {
        x: block_area.x + block_area.width.saturating_sub(2),
        y: block_area.y + 1,
        width: 1,
        height: block_area.height.saturating_sub(2),
    }
}

/// Draw a vertical scroll bar in `area`.
pub fn draw_scroll_bar(f: &mut Frame<'_>, area: Rect, args: ScrollBarArgs) {
    draw_scroll_bar_buffer(f.buffer_mut(), area, args);
}

/// Draw a vertical scroll bar directly into a buffer.
pub fn draw_scroll_bar_buffer(buffer: &mut Buffer, area: Rect, args: ScrollBarArgs) {
    if args.content_length <= args.viewport_length || area.width == 0 || area.height == 0 {
        return;
    }

    const SYMBOLS: scrollbar::Set<'_> = scrollbar::Set {
        track: "",
        thumb: symbols::block::FULL,
        begin: "▲",
        end: "▼",
    };

    // If the height is only 2, then there's no room for the thumb,
    // so instead we just draw a track with no arrows.
    let scrollbar = {
        let tmp = Scrollbar::new(ScrollbarOrientation::VerticalRight).style(args.style);

        if area.height > 2 {
            tmp.symbols(SYMBOLS)
        } else {
            tmp.track_symbol(Some(SYMBOLS.track))
                .thumb_symbol(SYMBOLS.thumb)
                .begin_symbol(None)
                .end_symbol(None)
        }
    };

    let mut state = ScrollbarState::new(args.content_length)
        .position(args.position)
        .viewport_content_length(args.viewport_length);

    scrollbar.render(area, buffer, &mut state);
}

#[cfg(test)]
mod test {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_test_bar(height: u16, content_length: usize, position: usize) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(1, height)).unwrap();
        terminal
            .draw(|f| {
                draw_scroll_bar(
                    f,
                    Rect::new(0, 0, 1, height),
                    ScrollBarArgs {
                        content_length,
                        viewport_length: 2,
                        position,
                        style: Style::default(),
                    },
                );
            })
            .unwrap();

        let buf = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| buf[(0, y)].symbol().to_string())
            .collect()
    }

    /// Make sure that a short scrollbar (height <= 2) is still drawn, just without the head/tail arrows.
    #[test]
    fn test_small_height_scroll_still_drawn() {
        assert_eq!(render_test_bar(1, 3, 0), ["█"]);
        assert_eq!(render_test_bar(2, 3, 0), ["█", " "]);
        assert_eq!(render_test_bar(2, 3, 2), [" ", "█"]);
    }

    #[test]
    fn test_normal_height_scroll_all_drawn() {
        assert_eq!(render_test_bar(3, 3, 0), ["▲", "█", "▼"]);
        assert_eq!(render_test_bar(4, 3, 0), ["▲", "█", " ", "▼"]);
        assert_eq!(render_test_bar(4, 3, 2), ["▲", " ", "█", "▼"]);
    }

    #[test]
    fn test_no_scroll_bar_when_list_fits() {
        assert_eq!(render_test_bar(4, 2, 0), [" ", " ", " ", " "]);
        assert_eq!(render_test_bar(2, 1, 0), [" ", " "]);
    }
}
