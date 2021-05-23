use crate::{
    app::AppState,
    canvas::{drawing_utils::interpolate_points, Painter},
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    symbols::Marker,
    terminal::Frame,
    text::Span,
    text::Spans,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};
use unicode_segmentation::UnicodeSegmentation;

pub trait MemGraphWidget {
    fn draw_memory_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    );
}

impl MemGraphWidget for Painter {
    fn draw_memory_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(mem_widget_state) = app_state.mem_state.widget_states.get_mut(&widget_id) {
            let mem_data: &mut [(f64, f64)] = &mut app_state.canvas_data.mem_data;
            let swap_data: &mut [(f64, f64)] = &mut app_state.canvas_data.swap_data;

            let time_start = -(mem_widget_state.current_display_time as f64);

            let display_time_labels = vec![
                Span::styled(
                    format!("{}s", mem_widget_state.current_display_time / 1000),
                    self.colours.graph_style,
                ),
                Span::styled("0s".to_string(), self.colours.graph_style),
            ];
            let y_axis_label = vec![
                Span::styled("  0%", self.colours.graph_style),
                Span::styled("100%", self.colours.graph_style),
            ];

            let x_axis = if app_state.app_config_fields.hide_time
                || (app_state.app_config_fields.autohide_time
                    && mem_widget_state.autohide_timer.is_none())
            {
                Axis::default().bounds([time_start, 0.0])
            } else if let Some(time) = mem_widget_state.autohide_timer {
                if std::time::Instant::now().duration_since(time).as_millis()
                    < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
                {
                    Axis::default()
                        .bounds([time_start, 0.0])
                        .style(self.colours.graph_style)
                        .labels(display_time_labels)
                } else {
                    mem_widget_state.autohide_timer = None;
                    Axis::default().bounds([time_start, 0.0])
                }
            } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
                Axis::default().bounds([time_start, 0.0])
            } else {
                Axis::default()
                    .bounds([time_start, 0.0])
                    .style(self.colours.graph_style)
                    .labels(display_time_labels)
            };

            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .bounds([0.0, 100.5])
                .labels(y_axis_label);

            // Interpolate values to avoid ugly gaps
            let interpolated_mem_point = if let Some(end_pos) = mem_data
                .iter()
                .position(|(time, _data)| *time >= time_start)
            {
                if end_pos > 1 {
                    let start_pos = end_pos - 1;
                    let outside_point = mem_data.get(start_pos);
                    let inside_point = mem_data.get(end_pos);

                    if let (Some(outside_point), Some(inside_point)) = (outside_point, inside_point)
                    {
                        let old = *outside_point;

                        let new_point = (
                            time_start,
                            interpolate_points(outside_point, inside_point, time_start),
                        );

                        if let Some(to_replace) = mem_data.get_mut(start_pos) {
                            *to_replace = new_point;
                            Some((start_pos, old))
                        } else {
                            None // Failed to get mutable reference.
                        }
                    } else {
                        None // Point somehow doesn't exist in our data
                    }
                } else {
                    None // Point is already "leftmost", no need to interpolate.
                }
            } else {
                None // There is no point.
            };

            let interpolated_swap_point = if let Some(end_pos) = swap_data
                .iter()
                .position(|(time, _data)| *time >= time_start)
            {
                if end_pos > 1 {
                    let start_pos = end_pos - 1;
                    let outside_point = swap_data.get(start_pos);
                    let inside_point = swap_data.get(end_pos);

                    if let (Some(outside_point), Some(inside_point)) = (outside_point, inside_point)
                    {
                        let old = *outside_point;

                        let new_point = (
                            time_start,
                            interpolate_points(outside_point, inside_point, time_start),
                        );

                        if let Some(to_replace) = swap_data.get_mut(start_pos) {
                            *to_replace = new_point;
                            Some((start_pos, old))
                        } else {
                            None // Failed to get mutable reference.
                        }
                    } else {
                        None // Point somehow doesn't exist in our data
                    }
                } else {
                    None // Point is already "leftmost", no need to interpolate.
                }
            } else {
                None // There is no point.
            };

            let mut mem_canvas_vec: Vec<Dataset<'_>> = vec![];

            if let Some((label_percent, label_frac)) = &app_state.canvas_data.mem_labels {
                let mem_label = format!("RAM:{}{}", label_percent, label_frac);
                mem_canvas_vec.push(
                    Dataset::default()
                        .name(mem_label)
                        .marker(if app_state.app_config_fields.use_dot {
                            Marker::Dot
                        } else {
                            Marker::Braille
                        })
                        .style(self.colours.ram_style)
                        .data(&mem_data)
                        .graph_type(tui::widgets::GraphType::Line),
                );
            }

            if let Some((label_percent, label_frac)) = &app_state.canvas_data.swap_labels {
                let swap_label = format!("SWP:{}{}", label_percent, label_frac);
                mem_canvas_vec.push(
                    Dataset::default()
                        .name(swap_label)
                        .marker(if app_state.app_config_fields.use_dot {
                            Marker::Dot
                        } else {
                            Marker::Braille
                        })
                        .style(self.colours.swap_style)
                        .data(&swap_data)
                        .graph_type(tui::widgets::GraphType::Line),
                );
            }

            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let border_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Memory ── Esc to go back ";
                Spans::from(vec![
                    Span::styled(" Memory ", self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to go back ",
                            "─".repeat(usize::from(draw_loc.width).saturating_sub(
                                UnicodeSegmentation::graphemes(TITLE_BASE, true).count() + 2
                            ))
                        ),
                        border_style,
                    ),
                ])
            } else {
                Spans::from(Span::styled(
                    " Memory ".to_string(),
                    self.colours.widget_title_style,
                ))
            };

            f.render_widget(
                Chart::new(mem_canvas_vec)
                    .block(
                        Block::default()
                            .title(title)
                            .borders(Borders::ALL)
                            .border_style(if app_state.current_widget.widget_id == widget_id {
                                self.colours.highlighted_border_style
                            } else {
                                self.colours.border_style
                            }),
                    )
                    .x_axis(x_axis)
                    .y_axis(y_axis)
                    .hidden_legend_constraints((Constraint::Ratio(3, 4), Constraint::Ratio(3, 4))),
                draw_loc,
            );

            // Now if you're done, reset any interpolated points!
            if let Some((index, old_value)) = interpolated_mem_point {
                if let Some(to_replace) = mem_data.get_mut(index) {
                    *to_replace = old_value;
                }
            }

            if let Some((index, old_value)) = interpolated_swap_point {
                if let Some(to_replace) = swap_data.get_mut(index) {
                    *to_replace = old_value;
                }
            }
        }

        if app_state.should_get_widget_bounds() {
            // Update draw loc in widget map
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}
