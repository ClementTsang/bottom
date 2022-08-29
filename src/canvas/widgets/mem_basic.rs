use crate::{
    app::App,
    canvas::{drawing_utils::*, Painter},
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    terminal::Frame,
    text::Span,
    text::Spans,
    widgets::{Block, Paragraph},
};

impl Painter {
    pub fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let mem_data: &[(f64, f64)] = &app_state.converted_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.converted_data.swap_data;
        let mut swap_used = false;
        let mut size = 1;

        let margined_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(1)
            .split(draw_loc);

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                Block::default()
                    .borders(SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                draw_loc,
            );
        }

        let ram_use_percentage = if let Some(mem) = mem_data.last() {
            mem.1
        } else {
            0.0
        };
        let swap_use_percentage = if let Some(swap) = swap_data.last() {
            swap.1
        } else {
            0.0
        };

        const EMPTY_MEMORY_FRAC_STRING: &str = "0.0B/0.0B";

        let trimmed_memory_frac =
            if let Some((_label_percent, label_frac)) = &app_state.converted_data.mem_labels {
                label_frac.trim()
            } else {
                EMPTY_MEMORY_FRAC_STRING
            };

        let trimmed_swap_frac =
            if let Some((_label_percent, label_frac)) = &app_state.converted_data.swap_labels {
                size += 1;
                swap_used = true;
                label_frac.trim()
            } else {
                EMPTY_MEMORY_FRAC_STRING
            };

        // +7 due to 3 + 2 + 2 columns for the name & space + bar bounds + margin spacing
        // Then + length of fraction
        let ram_bar_length =
            usize::from(draw_loc.width.saturating_sub(7)).saturating_sub(trimmed_memory_frac.len());
        let swap_bar_length =
            usize::from(draw_loc.width.saturating_sub(7)).saturating_sub(trimmed_swap_frac.len());

        let num_bars_ram = calculate_basic_use_bars(ram_use_percentage, ram_bar_length);
        let num_bars_swap = calculate_basic_use_bars(swap_use_percentage, swap_bar_length);
        // TODO: Use different styling for the frac.
        let mem_label = if app_state.basic_mode_use_percent {
            format!(
                "RAM[{}{}{:3.0}%]\n",
                "|".repeat(num_bars_ram),
                " ".repeat(ram_bar_length - num_bars_ram + trimmed_memory_frac.len() - 4),
                ram_use_percentage.round()
            )
        } else {
            format!(
                "RAM[{}{}{}]\n",
                "|".repeat(num_bars_ram),
                " ".repeat(ram_bar_length - num_bars_ram),
                trimmed_memory_frac
            )
        };
        let swap_label = if app_state.basic_mode_use_percent {
            format!(
                "SWP[{}{}{:3.0}%]",
                "|".repeat(num_bars_swap),
                " ".repeat(swap_bar_length - num_bars_swap + trimmed_swap_frac.len() - 4),
                swap_use_percentage.round()
            )
        } else {
            format!(
                "SWP[{}{}{}]",
                "|".repeat(num_bars_swap),
                " ".repeat(swap_bar_length - num_bars_swap),
                trimmed_swap_frac
            )
        };

        #[cfg(feature = "zfs")]
        let (arc_used, arc_label) = {
            let arc_data: &[(f64, f64)] = &app_state.converted_data.arc_data;
            let arc_use_percentage = if let Some(arc) = arc_data.last() {
                arc.1
            } else {
                0.0
            };
            let mut arc_used = false;
            let trimmed_arc_frac =
                if let Some((_label_percent, label_frac)) = &app_state.converted_data.arc_labels {
                    size += 1;
                    arc_used = true;
                    label_frac.trim()
                } else {
                    EMPTY_MEMORY_FRAC_STRING
                };
            let arc_bar_length = usize::from(draw_loc.width.saturating_sub(7))
                .saturating_sub(trimmed_arc_frac.len());
            let num_bars_arc = calculate_basic_use_bars(arc_use_percentage, arc_bar_length);
            let arc_label = if app_state.basic_mode_use_percent {
                format!(
                    "ARC[{}{}{:3.0}%]",
                    "|".repeat(num_bars_arc),
                    " ".repeat(arc_bar_length - num_bars_arc + trimmed_arc_frac.len() - 4),
                    arc_use_percentage.round()
                )
            } else {
                format!(
                    "ARC[{}{}{}]",
                    "|".repeat(num_bars_arc),
                    " ".repeat(arc_bar_length - num_bars_arc),
                    trimmed_arc_frac
                )
            };
            (
                arc_used,
                Spans::from(Span::styled(arc_label, self.colours.arc_style)),
            )
        };

        #[cfg(feature = "gpu")]
        let (gpu_used, gpu_labels) = {
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
                    let gpu_data: &[(f64, f64)] = gpu_data_vec.points.as_slice();
                    let gpu_use_percentage = if let Some(gpu) = gpu_data.last() {
                        gpu.1
                    } else {
                        0.0
                    };
                    let trimmed_gpu_frac = gpu_data_vec.mem_total.trim();
                    let gpu_bar_length = usize::from(draw_loc.width.saturating_sub(7))
                        .saturating_sub(trimmed_gpu_frac.len());
                    let num_bars_gpu = calculate_basic_use_bars(gpu_use_percentage, gpu_bar_length);
                    let gpu_label = if app_state.basic_mode_use_percent {
                        format!(
                            "GPU[{}{}{:3.0}%]",
                            "|".repeat(num_bars_gpu),
                            " ".repeat(gpu_bar_length - num_bars_gpu + trimmed_gpu_frac.len() - 4),
                            gpu_use_percentage.round()
                        )
                    } else {
                        format!(
                            "GPU[{}{}{}]",
                            "|".repeat(num_bars_gpu),
                            " ".repeat(gpu_bar_length - num_bars_gpu),
                            trimmed_gpu_frac
                        )
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
                    gpu_labels.push(Spans::from(Span::styled(gpu_label, style)));
                });
                (gpu_used, gpu_labels)
            } else {
                (false, vec![])
            }
        };

        let mut mem_text = Vec::with_capacity(size);

        mem_text.push(Spans::from(Span::styled(mem_label, self.colours.ram_style)));

        if swap_used {
            mem_text.push(Spans::from(Span::styled(
                swap_label,
                self.colours.swap_style,
            )));
        }

        #[cfg(feature = "zfs")]
        if arc_used {
            mem_text.push(arc_label);
        }

        #[cfg(feature = "gpu")]
        if gpu_used {
            for item in gpu_labels {
                mem_text.push(item);
            }
        }

        f.render_widget(
            Paragraph::new(mem_text).block(Block::default()),
            margined_loc[0],
        );

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
