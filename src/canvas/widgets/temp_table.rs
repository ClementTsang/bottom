use lazy_static::lazy_static;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
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

const TEMP_HEADERS: [&str; 2] = ["Sensor", "Temp"];

lazy_static! {
    static ref TEMP_HEADERS_LENS: Vec<u16> = TEMP_HEADERS
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>();
}
pub trait TempTableWidget {
    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl TempTableWidget for Painter {
    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(temp_widget_state) = app_state.temp_state.widget_states.get_mut(&widget_id) {
            let temp_sensor_data: &mut [Vec<String>] = &mut app_state.canvas_data.temp_sensor_data;

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
            let sliced_vec = &temp_sensor_data[start_position..];
            let temperature_rows = sliced_vec.iter().map(|temp_row| Row::Data(temp_row.iter()));

            // Calculate widths
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
                    &temp_widget_state.table_width_state.desired_column_widths,
                    Some(&[0.80, -1.0]),
                    &[1, 0],
                );
            }

            let (border_and_title_style, highlight_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Temperatures ── Esc to go back ";
                format!(
                    " Temperatures ─{}─ Esc to go back ",
                    "─".repeat(
                        usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2)
                    )
                )
            } else if app_state.app_config_fields.use_basic_mode {
                String::new()
            } else {
                " Temperatures ".to_string()
            };
            let title_style = if app_state.is_expanded {
                border_and_title_style
            } else {
                self.colours.widget_title_style
            };

            let temp_block = if draw_border {
                Block::default()
                    .title(&title)
                    .title_style(title_style)
                    .borders(Borders::ALL)
                    .border_style(border_and_title_style)
            } else if is_on_widget {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

            // Draw
            f.render_stateful_widget(
                Table::new(TEMP_HEADERS.iter(), temperature_rows)
                    .block(temp_block)
                    .header_style(self.colours.table_header_style)
                    .highlight_style(highlight_style)
                    .style(self.colours.text_style)
                    .widths(
                        &(temp_widget_state
                            .table_width_state
                            .calculated_column_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    )
                    .header_gap(table_gap),
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
