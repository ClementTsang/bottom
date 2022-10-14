use crate::{
    app::App, canvas::Painter, components::tui_widget::pipe_gauge::PipeGauge, constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::Block,
};

impl Painter {
    pub fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let mem_data = &app_state.converted_data.mem_data;
        let swap_data = &app_state.converted_data.swap_data;
        let mut swap_used = false;
        let mut size = 1;

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                Block::default()
                    .borders(SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                draw_loc,
            );
        }

        let ram_ratio = if let Some(mem) = mem_data.last() {
            mem.1
        } else {
            0.0
        };
        let swap_ratio = if let Some(swap) = swap_data.last() {
            swap.1
        } else {
            0.0
        };

        const EMPTY_MEMORY_FRAC_STRING: &str = "0.0B/0.0B";

        let memory_fraction_label =
            if let Some((_, label_frac)) = &app_state.converted_data.mem_labels {
                if app_state.basic_mode_use_percent {
                    format!("{:3.0}%", ram_ratio.round())
                } else {
                    label_frac.trim().to_string()
                }
            } else {
                EMPTY_MEMORY_FRAC_STRING.to_string()
            };

        let swap_fraction_label =
            if let Some((_, label_frac)) = &app_state.converted_data.swap_labels {
                size += 1;
                swap_used = true;
                if app_state.basic_mode_use_percent {
                    format!("{:3.0}%", swap_ratio.round())
                } else {
                    label_frac.trim().to_string()
                }
            } else {
                EMPTY_MEMORY_FRAC_STRING.to_string()
            };

        #[cfg(feature = "zfs")]
        let (arc_used, arc_fraction_label, arc_ratio) = {
            let arc_data = &app_state.converted_data.arc_data;
            let arc_ratio = if let Some(arc) = arc_data.last() {
                arc.1
            } else {
                0.0
            };
            let mut arc_used = false;
            let arc_label = {
                if let Some((_, label_frac)) = &app_state.converted_data.arc_labels {
                    size += 1;
                    arc_used = true;
                    if app_state.basic_mode_use_percent {
                        format!("{:3.0}%", arc_ratio.round())
                    } else {
                        label_frac.trim().to_string()
                    }
                } else {
                    EMPTY_MEMORY_FRAC_STRING.to_string()
                }
            };
            (arc_used, arc_label, arc_ratio)
        };

        #[cfg(feature = "gpu")]
        let (gpu_used, gpu_fraction_labels) = {
            let mut gpu_used = false;
            let mut gpu_labels = match &app_state.converted_data.gpu_data {
                Some(data) => Vec::with_capacity(data.len()),
                None => Vec::with_capacity(0),
            };
            let gpu_styles = &self.colours.gpu_colour_styles;
            let mut color_index = 0;
            if let Some(gpu_data) = &app_state.converted_data.gpu_data {
                gpu_data.iter().for_each(|gpu_data_vec| {
                    size += 1;
                    gpu_used = true;
                    let gpu_data = gpu_data_vec.points.as_slice();
                    let gpu_ratio = if let Some(gpu) = gpu_data.last() {
                        gpu.1
                    } else {
                        0.0
                    };
                    let trimmed_gpu_frac = {
                        if app_state.basic_mode_use_percent {
                            format!("{:3.0}%", gpu_ratio.round())
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
                    gpu_labels.push(
                        PipeGauge::default()
                            .ratio(gpu_ratio / 100.0)
                            .start_label("GPU")
                            .inner_label(trimmed_gpu_frac)
                            .label_style(style)
                            .gauge_style(style),
                    );
                });
                (gpu_used, gpu_labels)
            } else {
                (false, vec![])
            }
        };

        let constraint_layout: Vec<Constraint> = std::iter::repeat(Constraint::Length(1))
            .take(size)
            .collect();
        let margined_loc = Layout::default()
            .constraints(constraint_layout)
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .split(draw_loc);

        let mut draw_index = 0;

        f.render_widget(
            PipeGauge::default()
                .ratio(ram_ratio / 100.0)
                .start_label("RAM")
                .inner_label(memory_fraction_label)
                .label_style(self.colours.ram_style)
                .gauge_style(self.colours.ram_style),
            margined_loc[draw_index],
        );

        if swap_used {
            draw_index += 1;
            f.render_widget(
                PipeGauge::default()
                    .ratio(swap_ratio / 100.0)
                    .start_label("SWP")
                    .inner_label(swap_fraction_label)
                    .label_style(self.colours.swap_style)
                    .gauge_style(self.colours.swap_style),
                margined_loc[draw_index],
            );
        }

        #[cfg(feature = "zfs")]
        if arc_used {
            draw_index += 1;
            f.render_widget(
                PipeGauge::default()
                    .ratio(arc_ratio / 100.0)
                    .start_label("ARC")
                    .inner_label(arc_fraction_label)
                    .label_style(self.colours.arc_style)
                    .gauge_style(self.colours.arc_style),
                margined_loc[draw_index],
            );
        }

        #[cfg(feature = "gpu")]
        if gpu_used {
            for item in gpu_fraction_labels {
                draw_index += 1;
                f.render_widget(item, margined_loc[draw_index]);
            }
        }

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
