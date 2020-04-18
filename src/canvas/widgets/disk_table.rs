use lazy_static::lazy_static;
use std::cmp::max;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    app,
    canvas::{
        drawing_utils::{get_start_position, get_variable_intrinsic_widths},
        Painter,
    },
    constants::*,
};

const DISK_HEADERS: [&str; 7] = ["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"];

lazy_static! {
    static ref DISK_HEADERS_LENS: Vec<usize> = DISK_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
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
        if let Some(disk_widget_state) = app_state.disk_state.widget_states.get_mut(&widget_id) {
            let disk_data: &mut [Vec<String>] = &mut app_state.canvas_data.disk_data;
            let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
            let start_position = get_start_position(
                num_rows,
                &disk_widget_state.scroll_state.scroll_direction,
                &mut disk_widget_state.scroll_state.previous_scroll_position,
                disk_widget_state.scroll_state.current_scroll_position,
                app_state.is_resized,
            );

            let sliced_vec = &mut disk_data[start_position as usize..];
            let mut disk_counter: i64 = 0;

            let current_widget_id = app_state.current_widget.widget_id;
            let disk_rows = sliced_vec.iter().map(|disk| {
                Row::StyledData(
                    disk.iter(),
                    if current_widget_id == widget_id
                        && disk_widget_state.scroll_state.current_scroll_position >= start_position
                    {
                        if disk_counter as u64
                            == disk_widget_state.scroll_state.current_scroll_position
                                - start_position
                        {
                            disk_counter = -1;
                            self.colours.currently_selected_text_style
                        } else {
                            if disk_counter >= 0 {
                                disk_counter += 1;
                            }
                            self.colours.text_style
                        }
                    } else {
                        self.colours.text_style
                    },
                )
            });

            // Calculate widths
            // TODO: [PRETTY] Ellipsis on strings?
            let width = f64::from(draw_loc.width);
            let width_ratios = [0.2, 0.15, 0.13, 0.13, 0.13, 0.13, 0.13];
            let variable_intrinsic_results =
                get_variable_intrinsic_widths(width as u16, &width_ratios, &DISK_HEADERS_LENS);
            let intrinsic_widths = &variable_intrinsic_results.0[0..variable_intrinsic_results.1];

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Disk ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!(
                    " Disk ─{}─ Esc to go back ",
                    "─".repeat(repeat_num as usize)
                );
                result_title
            } else if app_state.app_config_fields.use_basic_mode {
                String::new()
            } else {
                " Disk ".to_string()
            };

            let border_and_title_style = if app_state.current_widget.widget_id == widget_id {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let disk_block = if draw_border {
                Block::default()
                    .title(&title)
                    .title_style(if app_state.is_expanded {
                        border_and_title_style
                    } else {
                        self.colours.widget_title_style
                    })
                    .borders(Borders::ALL)
                    .border_style(border_and_title_style)
            } else if app_state.current_widget.widget_id == widget_id {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .horizontal_margin(
                    if app_state.current_widget.widget_id == widget_id || draw_border {
                        0
                    } else {
                        1
                    },
                )
                .direction(Direction::Horizontal)
                .split(draw_loc);

            // Draw!
            f.render_widget(
                Table::new(DISK_HEADERS.iter(), disk_rows)
                    .block(disk_block)
                    .header_style(self.colours.table_header_style)
                    .widths(
                        &(intrinsic_widths
                            .iter()
                            .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                            .collect::<Vec<_>>()),
                    ),
                margined_draw_loc[0],
            );
        }
    }
}
