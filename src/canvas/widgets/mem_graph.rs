use std::cmp::max;

use crate::{
    app::{App, WidgetPosition},
    canvas::Painter,
};

use tui::{
    backend::Backend,
    layout::Rect,
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget},
};

pub trait MemGraphWidget {
    fn draw_memory_graph<B: Backend>(&self, f: &mut Frame<'_, B>, app_state: &App, draw_loc: Rect);
}

impl MemGraphWidget for Painter {
    fn draw_memory_graph<B: Backend>(&self, f: &mut Frame<'_, B>, app_state: &App, draw_loc: Rect) {
        let mem_data: &[(f64, f64)] = &app_state.canvas_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.canvas_data.swap_data;

        let display_time_labels = [
            format!("{}s", app_state.mem_state.display_time / 1000),
            "0s".to_string(),
        ];
        let x_axis = Axis::default()
            .bounds([0.0, app_state.mem_state.display_time as f64])
            .style(self.colours.graph_style)
            .labels_style(self.colours.graph_style)
            .labels(&display_time_labels);

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
                    .border_style(match app_state.current_widget_selected {
                        WidgetPosition::Mem => self.colours.highlighted_border_style,
                        _ => self.colours.border_style,
                    }),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
            .datasets(&mem_canvas_vec)
            .render(f, draw_loc);
    }
}
