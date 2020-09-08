use crate::app;
use std::cmp::min;

/// Return a (hard)-width vector for column widths.
/// * `total_width` is how much width we have to work with overall.
/// * `desired_widths` is the width that is *desired*, but may not be reached.
/// * `max_widths` is the maximal percentage we allow a column to take in the table.  If it is
///    negative, we assume it can take whatever size it wants.
/// * `min_widths` is the minimal hard width we allow a column to take in the table.  If it is
///   smaller than this, we just don't use it.
/// * `column_bias` is how we determine which columns are more important.  A higher value on a
///   column means it is more important.
/// * `spare_bias` is how we determine which column gets spare space first.  Again, higher value
///   means it is more important.
///
/// **NOTE:** This function ASSUMES THAT ALL PASSED SLICES ARE OF THE SAME SIZE.
///
/// **NOTE:** The returned vector may not be the same size as the slices, this is because including
/// 0-constraints breaks tui-rs.
///
/// **NOTE:** This function automatically takes away 2 from the width as part of the left/right
/// bounds.
pub fn get_column_widths(
    total_width: u16, desired_widths: &[u16], max_widths: Option<&[f64]>,
    min_widths: Option<&[u16]>, column_bias: &[usize], spare_bias: &[usize],
) -> Vec<u16> {
    let mut total_width_left = total_width.saturating_sub(desired_widths.len() as u16) + 1 - 2;
    let mut column_widths: Vec<u16> = vec![0; desired_widths.len()];

    // Let's sort out our bias into a sorted list (reverse to get descending order).
    let mut bias_list = column_bias.iter().enumerate().collect::<Vec<_>>();
    bias_list.sort_by(|a, b| a.1.cmp(b.1));
    bias_list.reverse();

    // Now, let's do a first pass.
    for itx in column_bias {
        let itx = *itx;
        let desired_width = if let Some(width_thresholds) = max_widths {
            if width_thresholds[itx].is_sign_negative() {
                desired_widths[itx]
            } else {
                min(
                    desired_widths[itx],
                    (width_thresholds[itx] * total_width as f64).ceil() as u16,
                )
            }
        } else {
            desired_widths[itx]
        };
        let remaining_width = min(total_width_left, desired_width);
        if let Some(min_widths) = min_widths {
            if remaining_width >= min_widths[itx] {
                column_widths[itx] = remaining_width;
                total_width_left -= remaining_width;
            }
        } else {
            column_widths[itx] = remaining_width;
            total_width_left -= remaining_width;
        }
    }

    // Second pass to fill in gaps and spaces
    while total_width_left > 0 {
        for itx in spare_bias {
            if column_widths[*itx] > 0 {
                column_widths[*itx] += 1;
                total_width_left -= 1;
                if total_width_left == 0 {
                    break;
                }
            }
        }
    }

    column_widths.into_iter().filter(|x| *x > 0).collect()
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
                *cursor_bar
            } else if current_cursor_position >= *cursor_bar + num_columns {
                *cursor_bar = current_cursor_position - num_columns;
                *cursor_bar
            } else {
                // Else, don't change what our start position is from whatever it is set to!
                *cursor_bar
            }
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
                *scroll_position_bar
            } else if currently_selected_position >= *scroll_position_bar + num_rows {
                *scroll_position_bar = currently_selected_position - num_rows;
                *scroll_position_bar
            } else {
                // Else, don't change what our start position is from whatever it is set to!
                *scroll_position_bar
            }
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
