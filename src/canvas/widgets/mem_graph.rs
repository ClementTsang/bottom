use crate::{app::App, canvas::Painter, constants::*};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
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
                        .labels(&display_time_labels)
                        .labels_style(self.colours.graph_style)
                } else {
                    mem_widget_state.autohide_timer = None;
                    Axis::default().bounds([-(mem_widget_state.current_display_time as f64), 0.0])
                }
            } else if draw_loc.height < TIME_LABEL_HEIGHT_LIMIT {
                Axis::default().bounds([-(mem_widget_state.current_display_time as f64), 0.0])
            } else {
                Axis::default()
                    .bounds([-(mem_widget_state.current_display_time as f64), 0.0])
                    .style(self.colours.graph_style)
                    .labels(&display_time_labels)
                    .labels_style(self.colours.graph_style)
            };

            let y_axis = Axis::default()
                .style(self.colours.graph_style)
                .bounds([0.0, 100.5])
                .labels(&["0%", "100%"])
                .labels_style(self.colours.graph_style);

            let mut mem_canvas_vec: Vec<Dataset<'_>> = vec![];
            let mem_label = format!(
                "RAM:{}{}",
                app_state.canvas_data.mem_label_percent, app_state.canvas_data.mem_label_frac
            );
            mem_canvas_vec.push(
                Dataset::default()
                    .name(&mem_label)
                    .marker(if app_state.app_config_fields.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(self.colours.ram_style)
                    .data(&mem_data)
                    .graph_type(tui::widgets::GraphType::Line),
            );

            // FIXME: [SWAP] Hide this if denominator is 0...
            let swap_label = format!(
                "SWP:{}{}",
                app_state.canvas_data.swap_label_percent, app_state.canvas_data.swap_label_frac
            );
            mem_canvas_vec.push(
                Dataset::default()
                    .name(&swap_label)
                    .marker(if app_state.app_config_fields.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(self.colours.swap_style)
                    .data(&swap_data)
                    .graph_type(tui::widgets::GraphType::Line),
            );

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Memory ── Esc to go back ";
                format!(
                    " Memory ─{}─ Esc to go back ",
                    "─".repeat(
                        usize::from(draw_loc.width).saturating_sub(TITLE_BASE.chars().count() + 2)
                    )
                )
            } else {
                " Memory ".to_string()
            };
            let title_style = if app_state.is_expanded {
                self.colours.highlighted_border_style
            } else {
                self.colours.widget_title_style
            };

            f.render_widget(
                // Chart::new(mem_canvas_vec)
                Chart::default()
                    .datasets(&mem_canvas_vec)
                    .block(
                        Block::default()
                            .title(&title)
                            .title_style(title_style)
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
