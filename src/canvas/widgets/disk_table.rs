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

const DISK_HEADERS: [&str; 7] = ["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"];

lazy_static! {
    static ref DISK_HEADERS_LENS: Vec<u16> = DISK_HEADERS
        .iter()
        .map(|entry| entry.len() as u16)
        .collect::<Vec<_>>();
}

pub trait DiskTableWidget {
    fn draw_disk_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl DiskTableWidget for Painter {
    fn draw_disk_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(disk_widget_state) = app_state.disk_state.widget_states.get_mut(&widget_id) {
            let disk_data: &mut [Vec<String>] = &mut app_state.canvas_data.disk_data;
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
            let sliced_vec = &disk_data[start_position..];
            let disk_rows = sliced_vec.iter().map(|disk| Row::Data(disk.iter()));

            // Calculate widths
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
                disk_widget_state.table_width_state.calculated_column_widths = get_column_widths(
                    draw_loc.width,
                    &disk_widget_state.table_width_state.desired_column_widths,
                    Some(&[0.2, 0.15, 0.13, 0.13, 0.13, 0.13, 0.13]),
                    Some(&[4, 5, 3, 4, 2, 2, 2]),
                    &[4, 3, 2, 1, 0, 5, 6],
                    &[4, 3, 2, 1, 0, 5, 6],
                );
            }

            // TODO: This seems to be bugged?  The selected text style gets "stuck"?  I think this gets fixed with tui 0.10?
            let (border_and_title_style, highlight_style) = if is_on_widget {
                (
                    self.colours.highlighted_border_style,
                    self.colours.currently_selected_text_style,
                )
            } else {
                (self.colours.border_style, self.colours.text_style)
            };

            // let title = if app_state.is_expanded {
            //     const TITLE_BASE: &str = " Disk ── Esc to go back ";
            //     Span::styled(
            //         format!(
            //             " Disk ─{}─ Esc to go back ",
            //             "─".repeat(
            //                 usize::from(draw_loc.width)
            //                     .saturating_sub(TITLE_BASE.chars().count() + 2)
            //             )
            //         ),
            //         border_and_title_style,
            //     )
            // } else if app_state.app_config_fields.use_basic_mode {
            //     Span::from(String::new())
            // } else {
            //     Span::styled(" Disk ".to_string(), self.colours.widget_title_style)
            // };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Disk ── Esc to go back ";
                format!(
                    " Disk ─{}─ Esc to go back ",
                    "─".repeat(
                        usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2)
                    )
                )
            } else if app_state.app_config_fields.use_basic_mode {
                String::new()
            } else {
                " Disk ".to_string()
            };

            let title_style = if app_state.is_expanded {
                border_and_title_style
            } else {
                self.colours.widget_title_style
            };

            let disk_block = if draw_border {
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

            // Draw!
            f.render_stateful_widget(
                Table::new(DISK_HEADERS.iter(), disk_rows)
                    .block(disk_block)
                    .header_style(self.colours.table_header_style)
                    .highlight_style(highlight_style)
                    .style(self.colours.text_style)
                    .widths(
                        &(disk_widget_state
                            .table_width_state
                            .calculated_column_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    )
                    .header_gap(table_gap),
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
