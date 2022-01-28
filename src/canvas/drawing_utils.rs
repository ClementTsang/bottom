use crate::app;
use std::cmp::{max, min};

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
                if (index as u16) < total_width_left {
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

/// Calculate how many bars are to be
/// drawn within basic mode's components.
pub fn calculate_basic_use_bars(use_percentage: f64, num_bars_available: usize) -> usize {
    std::cmp::min(
        (num_bars_available as f64 * use_percentage / 100.0).round() as usize,
        num_bars_available,
    )
}

/// Interpolates between two points.  Mainly used to help fill in tui-rs blanks in certain situations.
/// It is expected point_one is "further left" compared to point_two.
/// A point is two floats, in (x, y) form.  x is time, y is value.
pub fn interpolate_points(point_one: &(f64, f64), point_two: &(f64, f64), time: f64) -> f64 {
    let delta_x = point_two.0 - point_one.0;
    let delta_y = point_two.1 - point_one.1;
    let slope = delta_y / delta_x;

    (point_one.1 + (time - point_one.0) * slope).max(0.0)
}

#[cfg(test)]
mod test {
    use super::*;

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
            "vector should be empty"
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
            "vector should be empty"
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
            "vector should not be empty"
        );
    }
}
