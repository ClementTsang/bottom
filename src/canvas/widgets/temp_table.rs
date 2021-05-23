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

const TEMP_HEADERS: [&str; 2] = ["Sensor", "Temp"];

static TEMP_HEADERS_LENS: Lazy<Vec<u16>> = Lazy::new(|| {
    TEMP_HEADERS
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>()
});

pub trait TempTableWidget {
    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::AppState, draw_loc: Rect,
        draw_border: bool, widget_id: u64,
    );
}

impl TempTableWidget for Painter {
    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::AppState, draw_loc: Rect,
        draw_border: bool, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(temp_widget_state) = app_state.temp_state.widget_states.get_mut(&widget_id) {
            let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                app_state.app_config_fields.table_gap
            };
            let start_position = get_start_position(
                usize::from(
                    (draw_loc.height + (1 - table_gap)).saturating_sub(self.table_height_offset),
                ),
                &temp_widget_state.scroll_state.scroll_direction,
                &mut temp_widget_state.scroll_state.previous_scroll_position,
                temp_widget_state.scroll_state.current_scroll_position,
                app_state.is_force_redraw,
            );
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let temp_table_state = &mut temp_widget_state.scroll_state.table_state;
            temp_table_state.select(Some(
                temp_widget_state
                    .scroll_state
                    .current_scroll_position
                    .saturating_sub(start_position),
            ));
            let sliced_vec = &app_state.canvas_data.temp_sensor_data[start_position..];

            // Calculate widths
            let hard_widths = [None, None];
            if recalculate_column_widths {
                temp_widget_state.table_width_state.desired_column_widths = {
                    let mut column_widths = TEMP_HEADERS_LENS.clone();
                    for row in sliced_vec {
                        for (col, entry) in row.iter().enumerate() {
                            if entry.len() as u16 > column_widths[col] {
                                column_widths[col] = entry.len() as u16;
                            }
                        }
                    }

                    column_widths
                };
                temp_widget_state.table_width_state.calculated_column_widths = get_column_widths(
                    draw_loc.width,
                    &hard_widths,
                    &(TEMP_HEADERS_LENS
                        .iter()
                        .map(|width| Some(*width))
                        .collect::<Vec<_>>()),
                    &[Some(0.80), Some(-1.0)],
                    &temp_widget_state
                        .table_width_state
                        .desired_column_widths
                        .iter()
                        .map(|width| Some(*width))
                        .collect::<Vec<_>>(),
                    false,
                );
            }

            let dcw = &temp_widget_state.table_width_state.desired_column_widths;
            let ccw = &temp_widget_state.table_width_state.calculated_column_widths;
            let temperature_rows =
                sliced_vec.iter().map(|temp_row| {
                    let truncated_data = temp_row.iter().zip(&hard_widths).enumerate().map(
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
                                            Text::raw(format!("{}…", first_n))
                                        } else {
                                            Text::raw(entry)
                                        }
                                    } else {
                                        Text::raw(entry)
                                    }
                                } else {
                                    Text::raw(entry)
                                }
                            } else {
                                Text::raw(entry)
                            }
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
                    " Temperatures ({} of {}) ",
                    temp_widget_state
                        .scroll_state
                        .current_scroll_position
                        .saturating_add(1),
                    app_state.canvas_data.temp_sensor_data.len()
                );

                if title_string.len() <= draw_loc.width as usize {
                    title_string
                } else {
                    " Temperatures ".to_string()
                }
            } else {
                " Temperatures ".to_string()
            };

            let title = if app_state.is_expanded {
                const ESCAPE_ENDING: &str = "── Esc to go back ";

                let (chosen_title_base, expanded_title_base) = {
                    let temp_title_base = format!("{}{}", title_base, ESCAPE_ENDING);

                    if temp_title_base.len() > draw_loc.width as usize {
                        (
                            " Temperatures ".to_string(),
                            format!("{}{}", " Temperatures ".to_string(), ESCAPE_ENDING),
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

            let temp_block = if draw_border {
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

            // Draw
            f.render_stateful_widget(
                Table::new(temperature_rows)
                    .header(
                        Row::new(TEMP_HEADERS.to_vec())
                            .style(self.colours.table_header_style)
                            .bottom_margin(table_gap),
                    )
                    .block(temp_block)
                    .highlight_style(highlight_style)
                    .style(self.colours.text_style)
                    .widths(
                        &(temp_widget_state
                            .table_width_state
                            .calculated_column_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    ),
                margined_draw_loc,
                temp_table_state,
            );

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                // Note there is no difference between this and using draw_loc, but I'm too lazy to fix it.
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
