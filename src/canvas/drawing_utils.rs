use std::time::Instant;

use tui::{
    layout::Rect,
    widgets::{Block, BorderType, Borders},
};

pub const SIDE_BORDERS: Borders = Borders::LEFT.union(Borders::RIGHT);
pub const AUTOHIDE_TIMEOUT_MILLISECONDS: u64 = 5000; // 5 seconds to autohide

/// Determine whether a graph x-label should be hidden.
pub fn should_hide_x_label(
    always_hide_time: bool, autohide_time: bool, timer: &mut Option<Instant>, draw_loc: Rect,
) -> bool {
    const TIME_LABEL_HEIGHT_LIMIT: u16 = 7;

    if always_hide_time || (autohide_time && timer.is_none()) {
        true
    } else if let Some(time) = timer {
        if Instant::now().duration_since(*time).as_millis() < AUTOHIDE_TIMEOUT_MILLISECONDS.into() {
            false
        } else {
            *timer = None;
            true
        }
    } else {
        draw_loc.height < TIME_LABEL_HEIGHT_LIMIT
    }
}

/// Return a widget block.
pub fn widget_block(is_basic: bool, is_selected: bool, border_type: BorderType) -> Block<'static> {
    let mut block = Block::default().border_type(border_type);

    if is_basic {
        if is_selected {
            block = block.borders(SIDE_BORDERS);
        } else {
            block = block.borders(Borders::empty());
        }
    } else {
        block = block.borders(Borders::all());
    }

    block
}

/// Return a dialog block.
pub fn dialog_block(border_type: BorderType) -> Block<'static> {
    Block::default()
        .border_type(border_type)
        .borders(Borders::all())
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_should_hide_x_label() {
        use std::time::{Duration, Instant};

        use tui::layout::Rect;

        let rect = Rect::new(0, 0, 10, 10);
        let small_rect = Rect::new(0, 0, 10, 6);

        let mut under_timer = Some(Instant::now());
        let mut over_timer =
            Instant::now().checked_sub(Duration::from_millis(AUTOHIDE_TIMEOUT_MILLISECONDS + 100));

        assert!(should_hide_x_label(true, false, &mut None, rect));
        assert!(should_hide_x_label(false, true, &mut None, rect));
        assert!(should_hide_x_label(false, false, &mut None, small_rect));

        assert!(!should_hide_x_label(
            false,
            true,
            &mut under_timer,
            small_rect
        ));
        assert!(under_timer.is_some());

        assert!(should_hide_x_label(
            false,
            true,
            &mut over_timer,
            small_rect
        ));
        assert!(over_timer.is_none());
    }

    /// This test exists because previously, [`SIDE_BORDERS`] was set
    /// incorrectly after I moved from tui-rs to ratatui.
    #[test]
    fn assert_side_border_bits_match() {
        assert_eq!(
            SIDE_BORDERS,
            Borders::ALL.difference(Borders::TOP.union(Borders::BOTTOM))
        )
    }
}
