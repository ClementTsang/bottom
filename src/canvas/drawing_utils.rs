use crate::app;

/// A somewhat jury-rigged solution to simulate a variable intrinsic layout for
/// table widths.  Note that this will do one main pass to try to properly
/// allocate widths.  This will thus potentially cut off latter elements
/// (return size of 0) if it is too small (threshold), but will try its best.
///
/// `width thresholds` and `desired_widths_ratio` should be the same length.
/// Otherwise bad things happen.
pub fn get_variable_intrinsic_widths(
    total_width: u16, desired_widths_ratio: &[f64], width_thresholds: &[usize],
) -> (Vec<u16>, usize) {
    let num_widths = desired_widths_ratio.len();
    let mut resulting_widths: Vec<u16> = vec![0; num_widths];
    let mut last_index = 0;

    let mut remaining_width = (total_width - (num_widths as u16 - 1)) as i32; // Required for spaces...
    let desired_widths = desired_widths_ratio
        .iter()
        .map(|&desired_width_ratio| (desired_width_ratio * total_width as f64) as i32)
        .collect::<Vec<_>>();

    for (itx, desired_width) in desired_widths.into_iter().enumerate() {
        resulting_widths[itx] = if desired_width < width_thresholds[itx] as i32 {
            // Try to take threshold, else, 0
            if remaining_width < width_thresholds[itx] as i32 {
                0
            } else {
                remaining_width -= width_thresholds[itx] as i32;
                width_thresholds[itx] as u16
            }
        } else {
            // Take as large as possible
            if remaining_width < desired_width {
                // Check the biggest chunk possible
                if remaining_width < width_thresholds[itx] as i32 {
                    0
                } else {
                    let temp_width = remaining_width;
                    remaining_width = 0;
                    temp_width as u16
                }
            } else {
                remaining_width -= desired_width;
                desired_width as u16
            }
        };

        if resulting_widths[itx] == 0 {
            break;
        } else {
            last_index += 1;
        }
    }

    // Simple redistribution tactic - if there's any space left, split it evenly amongst all members
    if last_index < num_widths && last_index != 0 {
        let for_all_widths = (remaining_width / last_index as i32) as u16;
        let mut remainder = remaining_width % last_index as i32;

        for resulting_width in &mut resulting_widths {
            *resulting_width += for_all_widths;
            if remainder > 0 {
                *resulting_width += 1;
                remainder -= 1;
            }
        }
    }

    (resulting_widths, last_index)
}

#[allow(dead_code, unused_variables)]
pub fn get_search_start_position(
    num_columns: usize, cursor_direction: &app::CursorDirection, cursor_bar: &mut usize,
    current_cursor_position: usize, is_resized: bool,
) -> usize {
    if is_resized {
        *cursor_bar = 0;
    }

    match cursor_direction {
        app::CursorDirection::RIGHT => {
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
        app::CursorDirection::LEFT => {
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
    num_rows: u64, scroll_direction: &app::ScrollDirection, scroll_position_bar: &mut u64,
    currently_selected_position: u64, is_resized: bool,
) -> u64 {
    if is_resized {
        *scroll_position_bar = 0;
    }

    match scroll_direction {
        app::ScrollDirection::DOWN => {
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
        app::ScrollDirection::UP => {
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
