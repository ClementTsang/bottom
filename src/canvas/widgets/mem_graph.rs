use std::time::Instant;

use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
};

use crate::{
    app::{App, GraphStyle, data::Values},
    canvas::{
        Painter,
        components::time_graph::{GraphData, variants::percent::PercentTimeGraph},
        drawing_utils::should_hide_x_label,
    },
    collection::memory::MemData,
    get_binary_unit_and_denominator,
};

/// Convert memory info into a combined memory label.
#[inline]
fn memory_legend_label(name: &str, data: Option<&MemData>) -> String {
    if let Some(data) = data {
        let total_bytes = data.total_bytes.get();
        let percentage = data.used_bytes as f64 / total_bytes as f64 * 100.0;
        let (unit, denominator) = get_binary_unit_and_denominator(total_bytes);
        let used = data.used_bytes as f64 / denominator;
        let total = total_bytes as f64 / denominator;

        format!("{name}:{percentage:3.0}%   {used:.1}{unit}/{total:.1}{unit}")
    } else {
        format!("{name}:   0%   0.0B/0.0B")
    }
}

/// Get graph data.
#[inline]
fn graph_data<'a>(
    out: &mut Vec<GraphData<'a>>, name: &str, last_harvest: Option<&'a MemData>,
    time: &'a [Instant], values: &'a Values, style: Style, filled: bool,
) {
    if !values.no_elements() {
        let label = memory_legend_label(name, last_harvest).into();

        out.push(
            GraphData::default()
                .name(label)
                .time(time)
                .values(values)
                .style(style)
                .filled(filled),
        );
    }
}

#[derive(Clone)]
enum GraphType {
    Ram,
    #[cfg(feature = "gpu")]
    Gpu,
}

impl Painter {
    pub fn draw_memory_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(mem_state) = app_state.states.mem_state.widget_states.get_mut(&widget_id) {
            // Pre-declare total_vals to extend lifetime outside the loop
            #[allow(unused_assignments)]
            // It IS used in the loop, but maybe compiler is confused by re-assignment?
            // Actually, we should just declare it.
            let mut total_vals_storage: Vec<f64>;

            let data = app_state.data_store.get_data();
            let mut active_graphs = vec![GraphType::Ram];

            if !data.gpu_harvest.is_empty() {
                active_graphs.push(GraphType::Gpu);
            }

            let constraints: Vec<Constraint> = active_graphs
                .iter()
                .map(|_| Constraint::Ratio(1, active_graphs.len() as u32))
                .collect();

            let parent_block = crate::canvas::drawing_utils::widget_block(
                false,
                app_state.current_widget.widget_id == widget_id,
                self.styles.border_type,
            )
            .title(" Memory ")
            .border_style(self.styles.widget_title_style);

            let inner_area = parent_block.inner(draw_loc);
            f.render_widget(parent_block, draw_loc);

            // If we have multiple graphs, split the inner area
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(inner_area);

            let filled = matches!(app_state.app_config_fields.graph_style, GraphStyle::Filled);
            let time = &data.timeseries_data.time;

            for (i, graph_type) in active_graphs.iter().enumerate() {
                // If we are drawing inside the main block, we want NO borders on the components
                // EXCEPT maybe a divider for the GPU chart.
                // However, without borders, PercentTimeGraph might complain or look weird?
                // We added `borders` field exactly for this.
                let loc = layout[i];
                let mut points = Vec::new();
                let mut title: std::borrow::Cow<'_, str> = "".into();
                let mut borders = if active_graphs.len() > 1 && i == 0 {
                    // Top graph (Memory): Draw bottom border as separator
                    tui::widgets::Borders::BOTTOM
                } else if active_graphs.len() > 1 && i > 0 {
                    // Bottom graph (GPU): No borders (separator is drawn by Top)
                    tui::widgets::Borders::NONE
                } else {
                    // Single graph: No borders (parent handles it)
                    tui::widgets::Borders::NONE
                };

                // Adjust loc for divider:
                // If Top graph draws Bottom border, it uses the last row of its area.
                // The Bottom graph starts at next row.
                // This creates a nice graphical split.
                // However, titles?
                // GPU graph needs "GPU" title.
                // If Borders::NONE, title isn't drawn by Block.
                // So for GPU, maybe we want Borders::TOP?

                if matches!(graph_type, GraphType::Gpu) {
                    title = " GPU ".into();
                    // If we use Borders::TOP, we get a line + title.
                    borders = tui::widgets::Borders::TOP;

                    // But if Top Graph already drew Borders::BOTTOM...
                    // If Top graph ends at Y=10 (exclusive, so 0..10, last row 9). Border at 9.
                    // Bottom graph starts at 10. Top border at 10.
                    // This creates a double line (Row 9 and Row 10).
                    // This might be acceptable? Or we can turn off Top's bottom border.

                    if i > 0 {
                        // Correct previous graph border to NONE if we want Bottom to handle the divider
                        // But we can't change previous iteration.
                        // So let's set Top Graph to Borders::NONE.
                        // And Bottom Graph to Borders::TOP.
                    }
                }

                // Final Decision:
                // Memory (Top): Borders::NONE.
                // GPU (Bottom): Borders::TOP. (Functions as divider + title container).

                if i == 0 {
                    borders = tui::widgets::Borders::NONE;
                }

                match graph_type {
                    GraphType::Ram => {
                        // ... RAM/Swap stacking logic ...
                        // Calculate Total = RAM + Swap for stacking
                        // Only if swap exists, otherwise just RAM
                        if data.swap_harvest.is_some() {
                            // We need to calculate the sum of RAM + Swap for each time point
                            // Iterating through ChunkedData is messy, but we can iterate through the timeseries
                            // and get the values.
                            // Actually, let's just collect them into a Vec<f64>.
                            // Actually, let's just collect them into a Vec<f64>.
                            let raw_ram: Vec<f64> =
                                data.timeseries_data.ram.iter().copied().collect();
                            let raw_swap: Vec<f64> =
                                data.timeseries_data.swap.iter().copied().collect();

                            // Ensure lengths match or take min
                            let min_len = std::cmp::min(raw_ram.len(), raw_swap.len());
                            total_vals_storage = Vec::with_capacity(min_len);
                            for i in 0..min_len {
                                total_vals_storage.push(raw_ram[i] + raw_swap[i]);
                            }

                            // Draw Swap (Total height) first
                            points.push(
                                GraphData::default()
                                    .name(
                                        memory_legend_label("SWP", data.swap_harvest.as_ref())
                                            .into(),
                                    )
                                    .time(&time[0..min_len]) // Align time
                                    .custom_values(&total_vals_storage) // Use custom total
                                    .style(self.styles.swap_style)
                                    .filled(filled),
                            );

                            // Draw RAM on top
                            graph_data(
                                &mut points,
                                "RAM",
                                data.ram_harvest.as_ref(),
                                time,
                                &data.timeseries_data.ram,
                                self.styles.ram_style,
                                filled,
                            );
                        } else {
                            // Standard RAM only
                            graph_data(
                                &mut points,
                                "RAM",
                                data.ram_harvest.as_ref(),
                                time,
                                &data.timeseries_data.ram,
                                self.styles.ram_style,
                                filled,
                            );
                        }

                        #[cfg(not(target_os = "windows"))]
                        graph_data(
                            &mut points,
                            "CACHE",
                            data.cache_harvest.as_ref(),
                            time,
                            &data.timeseries_data.cache_mem,
                            self.styles.cache_style,
                            false,
                        );

                        #[cfg(feature = "zfs")]
                        {
                            graph_data(
                                &mut points,
                                "ARC",
                                data.arc_harvest.as_ref(),
                                time,
                                &data.timeseries_data.arc_mem,
                                self.styles.arc_style,
                                filled,
                            );
                        }
                    }
                    #[cfg(feature = "gpu")]
                    GraphType::Gpu => {
                        // Title handled above
                        let mut colour_index = 0;
                        let gpu_styles = &self.styles.gpu_colours;

                        for (name, harvest) in &data.gpu_harvest {
                            if let Some(gpu_data) = data.timeseries_data.gpu_mem.get(name) {
                                let style = if gpu_styles.is_empty() {
                                    Style::default()
                                } else {
                                    let colour = gpu_styles[colour_index % gpu_styles.len()];
                                    colour_index += 1;
                                    colour
                                };
                                graph_data(
                                    &mut points,
                                    name,
                                    Some(harvest),
                                    time,
                                    gpu_data,
                                    style,
                                    filled,
                                );
                            }
                        }
                    }
                }

                let hide_x_labels = should_hide_x_label(
                    app_state.app_config_fields.hide_time,
                    app_state.app_config_fields.autohide_time,
                    &mut mem_state.autohide_timer,
                    loc,
                );

                PercentTimeGraph {
                    display_range: mem_state.current_display_time,
                    hide_x_labels,
                    app_config_fields: &app_state.app_config_fields,
                    current_widget: app_state.current_widget.widget_id,
                    is_expanded: app_state.is_expanded,
                    title,
                    styles: &self.styles,
                    widget_id,
                    legend_position: app_state.app_config_fields.memory_legend_position,
                    legend_constraints: Some((Constraint::Ratio(3, 4), Constraint::Ratio(3, 4))),
                    borders,
                }
                .build()
                .draw(f, loc, points);
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
