use std::{cmp::min, time::Instant};

use tui::{
    layout::Rect,
    widgets::{Block, BorderType, Borders},
};

use super::SIDE_BORDERS;

/// Calculate how many bars are to be drawn within basic mode's components.
pub fn calculate_basic_use_bars(use_percentage: f64, num_bars_available: usize) -> usize {
    min(
        (num_bars_available as f64 * use_percentage / 100.0).round() as usize,
        num_bars_available,
    )
}

/// Determine whether a graph x-label should be hidden.
pub fn should_hide_x_label(
    always_hide_time: bool, autohide_time: bool, timer: &mut Option<Instant>, draw_loc: Rect,
) -> bool {
    use crate::constants::*;

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
pub fn widget_block(
    is_basic: bool, is_selected: bool, border_type: Option<BorderType>,
) -> Block<'static> {
    let mut block = Block::default().borders(Borders::empty());

    if let Some(border_type) = border_type {
        block = Block::default().border_type(border_type);

        if is_basic {
            if is_selected {
                block = block.borders(SIDE_BORDERS);
            } else {
                block = block.borders(Borders::empty());
            }
        } else {
            block = block.borders(Borders::all());
        }
    }

    block
}

/// Return a dialog block.
pub fn dialog_block(border_type: Option<BorderType>) -> Block<'static> {
    let mut block = Block::default().borders(Borders::empty());

    if let Some(border_type) = border_type {
        block = Block::default()
            .border_type(border_type)
            .borders(Borders::all());
    }

    block
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_calculate_basic_use_bars() {
        // Testing various breakpoints and edge cases.
        assert_eq!(calculate_basic_use_bars(0.0, 15), 0);
        assert_eq!(calculate_basic_use_bars(1.0, 15), 0);
        assert_eq!(calculate_basic_use_bars(5.0, 15), 1);
        assert_eq!(calculate_basic_use_bars(10.0, 15), 2);
        assert_eq!(calculate_basic_use_bars(40.0, 15), 6);
        assert_eq!(calculate_basic_use_bars(45.0, 15), 7);
        assert_eq!(calculate_basic_use_bars(50.0, 15), 8);
        assert_eq!(calculate_basic_use_bars(100.0, 15), 15);
        assert_eq!(calculate_basic_use_bars(150.0, 15), 15);
    }

    #[test]
    fn test_should_hide_x_label() {
        use std::time::{Duration, Instant};

        use tui::layout::Rect;

        use crate::constants::*;

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
}
