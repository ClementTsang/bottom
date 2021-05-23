use once_cell::sync::Lazy;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::Span,
    text::{Spans, Text},
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    app,
    canvas::{
        drawing_utils::{get_column_widths, get_start_position},
        Painter,
    },
    constants::*,
};
use unicode_segmentation::UnicodeSegmentation;

const DISK_HEADERS: [&str; 7] = ["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"];

static DISK_HEADERS_LENS: Lazy<Vec<u16>> = Lazy::new(|| {
    DISK_HEADERS
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>()
});

pub trait DiskTableWidget {
    fn draw_disk_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::AppState, draw_loc: Rect,
        draw_border: bool, widget_id: u64,
    );
}

impl DiskTableWidget for Painter {
    fn draw_disk_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::AppState, draw_loc: Rect,
        draw_border: bool, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(disk_widget_state) = app_state.disk_state.widget_states.get_mut(&widget_id) {
            let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                app_state.app_config_fields.table_gap
            };
            let start_position = get_start_position(
                usize::from(
                    (draw_loc.height + (1 - table_gap)).saturating_sub(self.table_height_offset),
                ),
                &disk_widget_state.scroll_state.scroll_direction,
                &mut disk_widget_state.scroll_state.previous_scroll_position,
                disk_widget_state.scroll_state.current_scroll_position,
                app_state.is_force_redraw,
            );
            let is_on_widget = app_state.current_widget.widget_id == widget_id;
            let disk_table_state = &mut disk_widget_state.scroll_state.table_state;
            disk_table_state.select(Some(
                disk_widget_state
                    .scroll_state
                    .current_scroll_position
                    .saturating_sub(start_position),
            ));
            let sliced_vec = &app_state.canvas_data.disk_data[start_position..];

            // Calculate widths
            let hard_widths = [None, None, Some(4), Some(6), Some(6), Some(7), Some(7)];
            if recalculate_column_widths {
                disk_widget_state.table_width_state.desired_column_widths = {
                    let mut column_widths = DISK_HEADERS_LENS.clone();
                    for row in sliced_vec {
                        for (col, entry) in row.iter().enumerate() {
                            if entry.len() as u16 > column_widths[col] {
                                column_widths[col] = entry.len() as u16;
                            }
                        }
                    }
                    column_widths
                };
                disk_widget_state.table_width_state.desired_column_widths = disk_widget_state
                    .table_width_state
                    .desired_column_widths
                    .iter()
                    .zip(&hard_widths)
                    .map(|(current, hard)| {
                        if let Some(hard) = hard {
                            if *hard > *current {
                                *hard
                            } else {
                                *current
                            }
                        } else {
                            *current
                        }
                    })
                    .collect::<Vec<_>>();

                disk_widget_state.table_width_state.calculated_column_widths = get_column_widths(
                    draw_loc.width,
                    &hard_widths,
                    &(DISK_HEADERS_LENS
                        .iter()
                        .map(|w| Some(*w))
                        .collect::<Vec<_>>()),
                    &[Some(0.2), Some(0.2), None, None, None, None, None],
                    &(disk_widget_state
                        .table_width_state
                        .desired_column_widths
                        .iter()
                        .map(|w| Some(*w))
                        .collect::<Vec<_>>()),
                    true,
                );
            }

            let dcw = &disk_widget_state.table_width_state.desired_column_widths;
            let ccw = &disk_widget_state.table_width_state.calculated_column_widths;
            let disk_rows =
                sliced_vec.iter().map(|disk_row| {
                    let truncated_data = disk_row.iter().zip(&hard_widths).enumerate().map(
                        |(itx, (entry, width))| {
                            if width.is_none() {
                                if let (Some(desired_col_width), Some(calculated_col_width)) =
                                    (dcw.get(itx), ccw.get(itx))
                                {
                                    if *desired_col_width > *calculated_col_width
                                        && *calculated_col_width > 0
                                    {
                                        let graphemes =
                                            UnicodeSegmentation::graphemes(entry.as_str(), true)
                                                .collect::<Vec<&str>>();

                                        if graphemes.len() > *calculated_col_width as usize
                                            && *calculated_col_width > 1
                                        {
                                            // Truncate with ellipsis
                                            let first_n = graphemes
                                                [..(*calculated_col_width as usize - 1)]
                                                .concat();
                                            return Text::raw(format!("{}…", first_n));
                                        }
                                    }
                                }
                            }

                            Text::raw(entry)
                        },
                    );

                    Row::new(truncated_data)
                });

            let (border_style, highlight_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };

            let title_base = if app_state.app_config_fields.show_table_scroll_position {
                let title_string = format!(
                    " Disk ({} of {}) ",
                    disk_widget_state
                        .scroll_state
                        .current_scroll_position
                        .saturating_add(1),
                    app_state.canvas_data.disk_data.len()
                );

                if title_string.len() <= draw_loc.width as usize {
                    title_string
                } else {
                    " Disk ".to_string()
                }
            } else {
                " Disk ".to_string()
            };

            let title = if app_state.is_expanded {
                const ESCAPE_ENDING: &str = "── Esc to go back ";

                let (chosen_title_base, expanded_title_base) = {
                    let temp_title_base = format!("{}{}", title_base, ESCAPE_ENDING);

                    if temp_title_base.len() > draw_loc.width as usize {
                        (
                            " Disk ".to_string(),
                            format!("{}{}", " Disk ".to_string(), ESCAPE_ENDING),
                        )
                    } else {
                        (title_base, temp_title_base)
                    }
                };

                Spans::from(vec![
                    Span::styled(chosen_title_base, self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to go back ",
                            "─".repeat(
                                usize::from(draw_loc.width).saturating_sub(
                                    UnicodeSegmentation::graphemes(
                                        expanded_title_base.as_str(),
                                        true
                                    )
                                    .count()
                                        + 2
                                )
                            )
                        ),
                        border_style,
                    ),
                ])
            } else {
                Spans::from(Span::styled(title_base, self.colours.widget_title_style))
            };

            let disk_block = if draw_border {
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            } else if is_on_widget {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

            // Draw!
            f.render_stateful_widget(
                Table::new(disk_rows)
                    .block(disk_block)
                    .header(
                        Row::new(DISK_HEADERS.to_vec())
                            .style(self.colours.table_header_style)
                            .bottom_margin(table_gap),
                    )
                    .highlight_style(highlight_style)
                    .style(self.colours.text_style)
                    .widths(
                        &(disk_widget_state
                            .table_width_state
                            .calculated_column_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    ),
                margined_draw_loc,
                disk_table_state,
            );

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                    widget.top_left_corner = Some((margined_draw_loc.x, margined_draw_loc.y));
                    widget.bottom_right_corner = Some((
                        margined_draw_loc.x + margined_draw_loc.width,
                        margined_draw_loc.y + margined_draw_loc.height,
                    ));
                }
            }
        }
    }
}
