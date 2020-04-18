use std::cmp::max;

use crate::{app::App, canvas::Painter, constants::*};

use tui::{
    backend::Backend,
    layout::Rect,
    symbols::Marker,
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};

pub trait MemGraphWidget {
    fn draw_memory_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl MemGraphWidget for Painter {
    fn draw_memory_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(mem_widget_state) = app_state.mem_state.widget_states.get_mut(&widget_id) {
            let mem_data: &[(f64, f64)] = &app_state.canvas_data.mem_data;
            let swap_data: &[(f64, f64)] = &app_state.canvas_data.swap_data;

            let display_time_labels = [
                format!("{}s", mem_widget_state.current_display_time / 1000),
                "0s".to_string(),
            ];
            let x_axis = if app_state.app_config_fields.hide_time
                || (app_state.app_config_fields.autohide_time
                    && mem_widget_state.autohide_timer.is_none())
            {
                Axis::default().bounds([-(mem_widget_state.current_display_time as f64), 0.0])
            } else if let Some(time) = mem_widget_state.autohide_timer {
                if std::time::Instant::now().duration_since(time).as_millis()
                    < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
                {
                    Axis::default()
                        .bounds([-(mem_widget_state.current_display_time as f64), 0.0])
                        .style(self.colours.graph_style)
                        .labels_style(self.colours.graph_style)
                        .labels(&display_time_labels)
                } else {
                    mem_widget_state.autohide_timer = None;
                    Axis::default().bounds([-(mem_widget_state.current_display_time as f64), 0.0])
                }
            } else {
                Axis::default()
                    .bounds([-(mem_widget_state.current_display_time as f64), 0.0])
                    .style(self.colours.graph_style)
                    .labels_style(self.colours.graph_style)
                    .labels(&display_time_labels)
            };

            // Offset as the zero value isn't drawn otherwise...
            let y_axis: Axis<'_, &str> = Axis::default()
                .style(self.colours.graph_style)
                .labels_style(self.colours.graph_style)
                .bounds([-0.5, 100.5])
                .labels(&["0%", "100%"]);

            let mem_canvas_vec: Vec<Dataset<'_>> = vec![
                Dataset::default()
                    .name(&app_state.canvas_data.mem_label)
                    .marker(if app_state.app_config_fields.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(self.colours.ram_style)
                    .data(&mem_data),
                Dataset::default()
                    .name(&app_state.canvas_data.swap_label)
                    .marker(if app_state.app_config_fields.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(self.colours.swap_style)
                    .data(&swap_data),
            ];

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Memory ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!(
                    " Memory ─{}─ Esc to go back ",
                    "─".repeat(repeat_num as usize)
                );

                result_title
            } else {
                " Memory ".to_string()
            };

            f.render_widget(
                Chart::default()
                    .block(
                        Block::default()
                            .title(&title)
                            .title_style(if app_state.is_expanded {
                                self.colours.highlighted_border_style
                            } else {
                                self.colours.widget_title_style
                            })
                            .borders(Borders::ALL)
                            .border_style(if app_state.current_widget.widget_id == widget_id {
                                self.colours.highlighted_border_style
                            } else {
                                self.colours.border_style
                            }),
                    )
                    .x_axis(x_axis)
                    .y_axis(y_axis)
                    .datasets(&mem_canvas_vec),
                draw_loc,
            );
        }
    }
}
