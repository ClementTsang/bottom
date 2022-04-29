use tui::layout::Rect;

use crate::app::{self};
use std::{
    cmp::{max, min},
    time::Instant,
};

/// Return a (hard)-width vector for column widths.
///
/// * `total_width` is the, well, total width available.  **NOTE:** This function automatically
/// takes away 2 from the width as part of the left/right
/// bounds.
/// * `hard_widths` is inflexible column widths.  Use a `None` to represent a soft width.
/// * `soft_widths_min` is the lower limit for a soft width.  Use `None` if a hard width goes there.
/// * `soft_widths_max` is the upper limit for a soft width, in percentage of the total width.  Use
///   `None` if a hard width goes there.
/// * `soft_widths_desired` is the desired soft width.  Use `None` if a hard width goes there.
/// * `left_to_right` is a boolean whether to go from left to right if true, or right to left if
///   false.
///
/// **NOTE:** This function ASSUMES THAT ALL PASSED SLICES ARE OF THE SAME SIZE.
///
/// **NOTE:** The returned vector may not be the same size as the slices, this is because including
/// 0-constraints breaks tui-rs.
pub fn get_column_widths(
    total_width: u16, hard_widths: &[Option<u16>], soft_widths_min: &[Option<u16>],
    soft_widths_max: &[Option<f64>], soft_widths_desired: &[Option<u16>], left_to_right: bool,
) -> Vec<u16> {
    debug_assert!(
        hard_widths.len() == soft_widths_min.len(),
        "hard width length != soft width min length!"
    );
    debug_assert!(
        soft_widths_min.len() == soft_widths_max.len(),
        "soft width min length != soft width max length!"
    );
    debug_assert!(
        soft_widths_max.len() == soft_widths_desired.len(),
        "soft width max length != soft width desired length!"
    );

    if total_width > 2 {
        let initial_width = total_width - 2;
        let mut total_width_left = initial_width;
        let mut column_widths: Vec<u16> = vec![0; hard_widths.len()];
        let range: Vec<usize> = if left_to_right {
            (0..hard_widths.len()).collect()
        } else {
            (0..hard_widths.len()).rev().collect()
        };

        for itx in &range {
            if let Some(Some(hard_width)) = hard_widths.get(*itx) {
                // Hard width...
                let space_taken = min(*hard_width, total_width_left);

                // TODO [COLUMN MOVEMENT]: Remove this
                if *hard_width > space_taken {
                    break;
                }

                column_widths[*itx] = space_taken;
                total_width_left -= space_taken;
                total_width_left = total_width_left.saturating_sub(1);
            } else if let (
                Some(Some(soft_width_max)),
                Some(Some(soft_width_min)),
                Some(Some(soft_width_desired)),
            ) = (
                soft_widths_max.get(*itx),
                soft_widths_min.get(*itx),
                soft_widths_desired.get(*itx),
            ) {
                // Soft width...
                let soft_limit = max(
                    if soft_width_max.is_sign_negative() {
                        *soft_width_desired
                    } else {
                        (*soft_width_max * initial_width as f64).ceil() as u16
                    },
                    *soft_width_min,
                );
                let space_taken = min(min(soft_limit, *soft_width_desired), total_width_left);

                // TODO [COLUMN MOVEMENT]: Remove this
                if *soft_width_min > space_taken {
                    break;
                }

                column_widths[*itx] = space_taken;
                total_width_left -= space_taken;
                total_width_left = total_width_left.saturating_sub(1);
            }
        }

        while let Some(0) = column_widths.last() {
            column_widths.pop();
        }

        if !column_widths.is_empty() {
            // Redistribute remaining.
            let amount_per_slot = total_width_left / column_widths.len() as u16;
            total_width_left %= column_widths.len() as u16;
            for (index, width) in column_widths.iter_mut().enumerate() {
                if index < total_width_left.into() {
                    *width += amount_per_slot + 1;
                } else {
                    *width += amount_per_slot;
                }
            }
        }

        column_widths
    } else {
        vec![]
    }
}

pub fn get_search_start_position(
    num_columns: usize, cursor_direction: &app::CursorDirection, cursor_bar: &mut usize,
    current_cursor_position: usize, is_force_redraw: bool,
) -> usize {
    if is_force_redraw {
        *cursor_bar = 0;
    }

    match cursor_direction {
        app::CursorDirection::Right => {
            if current_cursor_position < *cursor_bar + num_columns {
                // If, using previous_scrolled_position, we can see the element
                // (so within that and + num_rows) just reuse the current previously scrolled position
                *cursor_bar
            } else if current_cursor_position >= num_columns {
                // Else if the current position past the last element visible in the list, omit
                // until we can see that element
                *cursor_bar = current_cursor_position - num_columns;
                *cursor_bar
            } else {
                // Else, if it is not past the last element visible, do not omit anything
                0
            }
        }
        app::CursorDirection::Left => {
            if current_cursor_position <= *cursor_bar {
                // If it's past the first element, then show from that element downwards
                *cursor_bar = current_cursor_position;
            } else if current_cursor_position >= *cursor_bar + num_columns {
                *cursor_bar = current_cursor_position - num_columns;
            }
            // Else, don't change what our start position is from whatever it is set to!
            *cursor_bar
        }
    }
}

pub fn get_start_position(
    num_rows: usize, scroll_direction: &app::ScrollDirection, scroll_position_bar: &mut usize,
    currently_selected_position: usize, is_force_redraw: bool,
) -> usize {
    if is_force_redraw {
        *scroll_position_bar = 0;
    }

    match scroll_direction {
        app::ScrollDirection::Down => {
            if currently_selected_position < *scroll_position_bar + num_rows {
                // If, using previous_scrolled_position, we can see the element
                // (so within that and + num_rows) just reuse the current previously scrolled position
                *scroll_position_bar
            } else if currently_selected_position >= num_rows {
                // Else if the current position past the last element visible in the list, omit
                // until we can see that element
                *scroll_position_bar = currently_selected_position - num_rows;
                *scroll_position_bar
            } else {
                // Else, if it is not past the last element visible, do not omit anything
                0
            }
        }
        app::ScrollDirection::Up => {
            if currently_selected_position <= *scroll_position_bar {
                // If it's past the first element, then show from that element downwards
                *scroll_position_bar = currently_selected_position;
            } else if currently_selected_position >= *scroll_position_bar + num_rows {
                *scroll_position_bar = currently_selected_position - num_rows;
            }
            // Else, don't change what our start position is from whatever it is set to!
            *scroll_position_bar
        }
    }
}

/// Calculate how many bars are to be drawn within basic mode's components.
pub fn calculate_basic_use_bars(use_percentage: f64, num_bars_available: usize) -> usize {
    std::cmp::min(
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_start_position() {
        use crate::app::ScrollDirection::{self, Down, Up};

        fn test(
            bar: usize, num: usize, direction: ScrollDirection, selected: usize, force: bool,
            expected_posn: usize, expected_bar: usize,
        ) {
            let mut bar = bar;
            assert_eq!(
                get_start_position(num, &direction, &mut bar, selected, force),
                expected_posn
            );
            assert_eq!(bar, expected_bar);
        }

        // Scrolling down from start
        test(0, 10, Down, 0, false, 0, 0);

        // Simple scrolling down
        test(0, 10, Down, 1, false, 0, 0);

        // Scrolling down from the middle high up
        test(0, 10, Down, 5, false, 0, 0);

        // Scrolling down into boundary
        test(0, 10, Down, 11, false, 1, 1);

        // Scrolling down from the with non-zero bar
        test(5, 10, Down, 15, false, 5, 5);

        // Force redraw scrolling down (e.g. resize)
        test(5, 15, Down, 15, true, 0, 0);

        // Test jumping down
        test(1, 10, Down, 20, true, 10, 10);

        // Scrolling up from bottom
        test(10, 10, Up, 20, false, 10, 10);

        // Simple scrolling up
        test(10, 10, Up, 19, false, 10, 10);

        // Scrolling up from the middle
        test(10, 10, Up, 10, false, 10, 10);

        // Scrolling up into boundary
        test(10, 10, Up, 9, false, 9, 9);

        // Force redraw scrolling up (e.g. resize)
        test(5, 10, Up, 15, true, 5, 5);

        // Test jumping up
        test(10, 10, Up, 0, false, 0, 0);
    }

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
        use crate::constants::*;
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

    #[test]
    fn test_width_calculation() {
        // TODO: Implement width calculation test; can reuse old ones as basis
        todo!()
    }

    #[test]
    fn test_zero_width() {
        assert_eq!(
            get_column_widths(
                0,
                &[Some(1), None, None],
                &[None, Some(1), Some(2)],
                &[None, Some(0.125), Some(0.5)],
                &[None, Some(10), Some(10)],
                true
            ),
            vec![],
        );
    }

    #[test]
    fn test_two_width() {
        assert_eq!(
            get_column_widths(
                2,
                &[Some(1), None, None],
                &[None, Some(1), Some(2)],
                &[None, Some(0.125), Some(0.5)],
                &[None, Some(10), Some(10)],
                true
            ),
            vec![],
        );
    }

    #[test]
    fn test_non_zero_width() {
        assert_eq!(
            get_column_widths(
                16,
                &[Some(1), None, None],
                &[None, Some(1), Some(2)],
                &[None, Some(0.125), Some(0.5)],
                &[None, Some(10), Some(10)],
                true
            ),
            vec![2, 2, 7],
        );
    }
}
