use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Block,
    Frame,
};

use crate::{
    app::App,
    canvas::{components::pipe_gauge::PipeGauge, Painter},
    constants::*,
};

impl Painter {
    pub fn draw_basic_memory(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let mem_data = &app_state.converted_data.mem_data;
        let mut draw_widgets: Vec<PipeGauge<'_>> = Vec::new();

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                Block::default()
                    .borders(SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                draw_loc,
            );
        }

        let ram_percentage = if let Some(mem) = mem_data.last() {
            mem.1
        } else {
            0.0
        };

        const EMPTY_MEMORY_FRAC_STRING: &str = "0.0B/0.0B";

        let memory_fraction_label =
            if let Some((_, label_frac)) = &app_state.converted_data.mem_labels {
                if app_state.basic_mode_use_percent {
                    format!("{:3.0}%", ram_percentage.round())
                } else {
                    label_frac.trim().to_string()
                }
            } else {
                EMPTY_MEMORY_FRAC_STRING.to_string()
            };

        draw_widgets.push(
            PipeGauge::default()
                .ratio(ram_percentage / 100.0)
                .start_label("RAM")
                .inner_label(memory_fraction_label)
                .label_style(self.colours.ram_style)
                .gauge_style(self.colours.ram_style),
        );

        #[cfg(not(target_os = "windows"))]
        {
            if let Some((_, label_frac)) = &app_state.converted_data.cache_labels {
                let cache_data = &app_state.converted_data.cache_data;

                let cache_percentage = if let Some(cache) = cache_data.last() {
                    cache.1
                } else {
                    0.0
                };

                let cache_fraction_label = if app_state.basic_mode_use_percent {
                    format!("{:3.0}%", cache_percentage.round())
                } else {
                    label_frac.trim().to_string()
                };
                draw_widgets.push(
                    PipeGauge::default()
                        .ratio(cache_percentage / 100.0)
                        .start_label("CHE")
                        .inner_label(cache_fraction_label)
                        .label_style(self.colours.cache_style)
                        .gauge_style(self.colours.cache_style),
                );
            }
        }

        let swap_data = &app_state.converted_data.swap_data;

        let swap_percentage = if let Some(swap) = swap_data.last() {
            swap.1
        } else {
            0.0
        };

        if let Some((_, label_frac)) = &app_state.converted_data.swap_labels {
            let swap_fraction_label = if app_state.basic_mode_use_percent {
                format!("{:3.0}%", swap_percentage.round())
            } else {
                label_frac.trim().to_string()
            };
            draw_widgets.push(
                PipeGauge::default()
                    .ratio(swap_percentage / 100.0)
                    .start_label("SWP")
                    .inner_label(swap_fraction_label)
                    .label_style(self.colours.swap_style)
                    .gauge_style(self.colours.swap_style),
            );
        }

        #[cfg(feature = "zfs")]
        {
            let arc_data = &app_state.converted_data.arc_data;
            let arc_percentage = if let Some(arc) = arc_data.last() {
                arc.1
            } else {
                0.0
            };
            if let Some((_, label_frac)) = &app_state.converted_data.arc_labels {
                let arc_fraction_label = if app_state.basic_mode_use_percent {
                    format!("{:3.0}%", arc_percentage.round())
                } else {
                    label_frac.trim().to_string()
                };
                draw_widgets.push(
                    PipeGauge::default()
                        .ratio(arc_percentage / 100.0)
                        .start_label("ARC")
                        .inner_label(arc_fraction_label)
                        .label_style(self.colours.arc_style)
                        .gauge_style(self.colours.arc_style),
                );
            }
        }

        #[cfg(feature = "gpu")]
        {
            if let Some(gpu_data) = &app_state.converted_data.gpu_data {
                let gpu_styles = &self.colours.gpu_colours;
                let mut color_index = 0;

                gpu_data.iter().for_each(|gpu_data_vec| {
                    let gpu_data = gpu_data_vec.points.as_slice();
                    let gpu_percentage = if let Some(gpu) = gpu_data.last() {
                        gpu.1
                    } else {
                        0.0
                    };
                    let trimmed_gpu_frac = {
                        if app_state.basic_mode_use_percent {
                            format!("{:3.0}%", gpu_percentage.round())
                        } else {
                            gpu_data_vec.mem_total.trim().to_string()
                        }
                    };
                    let style = {
                        if gpu_styles.is_empty() {
                            tui::style::Style::default()
                        } else if color_index >= gpu_styles.len() {
                            // cycle styles
                            color_index = 1;
                            gpu_styles[color_index - 1]
                        } else {
                            color_index += 1;
                            gpu_styles[color_index - 1]
                        }
                    };
                    draw_widgets.push(
                        PipeGauge::default()
                            .ratio(gpu_percentage / 100.0)
                            .start_label("GPU")
                            .inner_label(trimmed_gpu_frac)
                            .label_style(style)
                            .gauge_style(style),
                    );
                });
            }
        }

        let margined_loc = Layout::default()
            .constraints(vec![Constraint::Length(1); draw_widgets.len()])
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .split(draw_loc);

        draw_widgets
            .into_iter()
            .enumerate()
            .for_each(|(index, widget)| {
                f.render_widget(widget, margined_loc[index]);
            });

        // Update draw loc in widget map
        if app_state.should_get_widget_bounds() {
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}
