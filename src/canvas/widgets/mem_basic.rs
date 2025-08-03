use std::borrow::Cow;

use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    app::App,
    canvas::{Painter, components::pipe_gauge::PipeGauge, drawing_utils::widget_block},
    collection::memory::MemData,
    get_binary_unit_and_denominator,
};

/// Convert memory info into a string representing a fraction.
#[inline]
fn memory_fraction_label(data: &MemData) -> Cow<'static, str> {
    let total_bytes = data.total_bytes.get();
    let (unit, denominator) = get_binary_unit_and_denominator(total_bytes);
    let used = data.used_bytes as f64 / denominator;
    let total = total_bytes as f64 / denominator;

    format!("{used:.1}{unit}/{total:.1}{unit}").into()
}

/// Convert memory info into a string representing a percentage.
#[inline]
fn memory_percentage_label(data: &MemData) -> Cow<'static, str> {
    let total_bytes = data.total_bytes.get();
    let percentage = data.used_bytes as f64 / total_bytes as f64 * 100.0;
    format!("{percentage:3.0}%").into()
}

#[inline]
fn memory_label(data: &MemData, is_percentage: bool) -> Cow<'static, str> {
    if is_percentage {
        memory_percentage_label(data)
    } else {
        memory_fraction_label(data)
    }
}

impl Painter {
    pub fn draw_basic_memory(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let mut draw_widgets: Vec<PipeGauge<'_>> = Vec::new();

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                widget_block(true, true, self.styles.border_type)
                    .border_style(self.styles.highlighted_border_style),
                draw_loc,
            );
        }

        let data = app_state.data_store.get_data();

        let (ram_percentage, ram_label) = if let Some(ram_harvest) = &data.ram_harvest {
            (
                ram_harvest.percentage(),
                memory_label(ram_harvest, app_state.basic_mode_use_percent),
            )
        } else {
            (
                0.0,
                if app_state.basic_mode_use_percent {
                    "0.0B/0.0B".into()
                } else {
                    "  0%".into()
                },
            )
        };

        draw_widgets.push(
            PipeGauge::default()
                .ratio(ram_percentage / 100.0)
                .start_label("RAM")
                .inner_label(ram_label)
                .label_style(self.styles.ram_style)
                .gauge_style(self.styles.ram_style),
        );

        if let Some(swap_harvest) = &data.swap_harvest {
            let swap_percentage = swap_harvest.percentage();
            let swap_label = memory_label(swap_harvest, app_state.basic_mode_use_percent);

            draw_widgets.push(
                PipeGauge::default()
                    .ratio(swap_percentage / 100.0)
                    .start_label("SWP")
                    .inner_label(swap_label)
                    .label_style(self.styles.swap_style)
                    .gauge_style(self.styles.swap_style),
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(cache_harvest) = &data.cache_harvest {
                let cache_percentage = cache_harvest.percentage();
                let cache_fraction_label =
                    memory_label(cache_harvest, app_state.basic_mode_use_percent);

                draw_widgets.push(
                    PipeGauge::default()
                        .ratio(cache_percentage / 100.0)
                        .start_label("CHE")
                        .inner_label(cache_fraction_label)
                        .label_style(self.styles.cache_style)
                        .gauge_style(self.styles.cache_style),
                );
            }
        }

        #[cfg(feature = "zfs")]
        {
            if let Some(arc_harvest) = &data.arc_harvest {
                let arc_percentage = arc_harvest.percentage();
                let arc_fraction_label =
                    memory_label(arc_harvest, app_state.basic_mode_use_percent);

                draw_widgets.push(
                    PipeGauge::default()
                        .ratio(arc_percentage / 100.0)
                        .start_label("ARC")
                        .inner_label(arc_fraction_label)
                        .label_style(self.styles.arc_style)
                        .gauge_style(self.styles.arc_style),
                );
            }
        }

        #[cfg(feature = "gpu")]
        {
            let gpu_styles = &self.styles.gpu_colours;
            let mut colour_index = 0;

            for (_, harvest) in data.gpu_harvest.iter() {
                let percentage = harvest.percentage();
                let label = memory_label(harvest, app_state.basic_mode_use_percent);

                let style = {
                    if gpu_styles.is_empty() {
                        tui::style::Style::default()
                    } else {
                        let colour = gpu_styles[colour_index % gpu_styles.len()];
                        colour_index += 1;

                        colour
                    }
                };

                draw_widgets.push(
                    PipeGauge::default()
                        .ratio(percentage / 100.0)
                        .start_label("GPU")
                        .inner_label(label)
                        .label_style(style)
                        .gauge_style(style),
                );
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
