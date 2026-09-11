use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    app::{App, data::InnerData, layout_manager::WidgetDirection},
    canvas::{
        Painter,
        components::{
            data_table::{DrawInfo, SelectionState},
            time_series::{GraphData, LegendConstraints},
        },
        drawing_utils::should_hide_x_label,
    },
    components::time_series::GraphDrawCtx,
    options::config::cpu::CpuLegendMode,
    widgets::{CpuWidgetState, cpu_legend_label},
};

const AVG_POSITION: usize = 1;
const ALL_POSITION: usize = 0;

impl Painter {
    pub fn draw_cpu(&self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64) {
        // In overlay/hidden legend modes, the CPU chart always uses the full width;
        // only the classic table mode may carve out space for the side legend.
        let use_side_table = app_state
            .app_config_fields
            .cpu_legend_mode
            .uses_side_table();
        let legend_width = if use_side_table {
            (draw_loc.width as f64 * 0.15) as u16
        } else {
            0
        };

        if !use_side_table || legend_width < 6 {
            // Either the user asked for an in-chart/hidden legend (so there is no
            // side legend widget at all), or the widget is too small to fit the
            // side table. Either way, the graph takes the full widget width and
            // the side legend is not present.
            //
            // Note: `is_legend_hidden` means "no interactive side legend widget
            // exists"; every consumer of it is gated on
            // `BottomWidgetType::CpuLegend` (see `App`), which is only in the
            // layout when the side table is used.
            if app_state.current_widget.widget_id == (widget_id + 1) {
                if app_state.app_config_fields.cpu_left_legend {
                    app_state.move_widget_selection(&WidgetDirection::Right);
                } else {
                    app_state.move_widget_selection(&WidgetDirection::Left);
                }
            }
            self.draw_cpu_graph(f, app_state, draw_loc, widget_id);
            if let Some(cpu_widget_state) =
                app_state.states.cpu_state.widget_states.get_mut(&widget_id)
            {
                cpu_widget_state.is_legend_hidden = true;
            }

            // Update draw loc in widget map
            if app_state.should_get_widget_bounds()
                && let Some(bottom_widget) = app_state.widget_map.get_mut(&widget_id)
            {
                bottom_widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                bottom_widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        } else {
            let graph_width = draw_loc.width - legend_width;
            let (graph_index, legend_index, constraints) =
                if app_state.app_config_fields.cpu_left_legend {
                    (
                        1,
                        0,
                        [
                            Constraint::Length(legend_width),
                            Constraint::Length(graph_width),
                        ],
                    )
                } else {
                    (
                        0,
                        1,
                        [
                            Constraint::Length(graph_width),
                            Constraint::Length(legend_width),
                        ],
                    )
                };

            let partitioned_draw_loc = Layout::default()
                .margin(0)
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(draw_loc);

            self.draw_cpu_graph(f, app_state, partitioned_draw_loc[graph_index], widget_id);
            self.draw_cpu_legend(
                f,
                app_state,
                partitioned_draw_loc[legend_index],
                widget_id + 1,
            );

            if app_state.should_get_widget_bounds() {
                // Update draw loc in widget map
                if let Some(cpu_widget) = app_state.widget_map.get_mut(&widget_id) {
                    cpu_widget.top_left_corner = Some((
                        partitioned_draw_loc[graph_index].x,
                        partitioned_draw_loc[graph_index].y,
                    ));
                    cpu_widget.bottom_right_corner = Some((
                        partitioned_draw_loc[graph_index].x
                            + partitioned_draw_loc[graph_index].width,
                        partitioned_draw_loc[graph_index].y
                            + partitioned_draw_loc[graph_index].height,
                    ));
                }

                if let Some(legend_widget) = app_state.widget_map.get_mut(&(widget_id + 1)) {
                    legend_widget.top_left_corner = Some((
                        partitioned_draw_loc[legend_index].x,
                        partitioned_draw_loc[legend_index].y,
                    ));
                    legend_widget.bottom_right_corner = Some((
                        partitioned_draw_loc[legend_index].x
                            + partitioned_draw_loc[legend_index].width,
                        partitioned_draw_loc[legend_index].y
                            + partitioned_draw_loc[legend_index].height,
                    ));
                }
            }
        }
    }

    fn generate_points<'a>(
        &self, cpu_widget_state: &'a CpuWidgetState, data: &'a InnerData, show_avg_cpu: bool,
        show_decimal: bool, name_entries: bool,
    ) -> Vec<GraphData<'a>> {
        let show_avg_offset = if show_avg_cpu { AVG_POSITION } else { 0 };
        let current_scroll_position = cpu_widget_state.table.state.current_index;
        let cpu_entries = &data.cpu_harvest;
        let cpu_points = &data.time_series_data.cpu;
        let time = &data.time_series_data.time;

        if current_scroll_position == ALL_POSITION {
            // This case ensures the other cases cannot have the position be
            // equal to 0.
            //
            // The in-chart legend only ever shows a single entry here (the
            // default selected row); the other datasets keep `name = None` so
            // they are drawn but not listed. `cpu_entries[0]` is the average when
            // it is shown, and the first per-CPU entry when it is hidden, so the
            // legend is never left empty.
            let all_view_name = if name_entries {
                cpu_entries
                    .first()
                    .map(|entry| cpu_legend_label(entry.data_type, entry.usage, show_decimal))
            } else {
                None
            };

            cpu_points
                .iter()
                .enumerate()
                .map(|(itx, values)| {
                    let style = if show_avg_cpu && itx == 0 {
                        self.styles.avg_cpu_colour
                    } else {
                        self.styles.cpu_colour_styles
                            [(itx - show_avg_offset) % self.styles.cpu_colour_styles.len()]
                    };

                    let mut gd = GraphData::default().style(style).time(time).values(values);
                    if itx == 0
                        && let Some(name) = &all_view_name
                    {
                        gd = gd.name(name.clone());
                    }
                    gd
                })
                .rev()
                .collect()
        } else if let Some(entry) = cpu_entries.get(current_scroll_position - 1) {
            // We generally subtract one from current scroll position because of
            // the all entry. TODO: Do this a bit better (e.g. we
            // can just do if let Some(_) = cpu_points.get())

            let style = if show_avg_cpu && current_scroll_position == AVG_POSITION {
                self.styles.avg_cpu_colour
            } else {
                let offset_position = current_scroll_position - 1;
                self.styles.cpu_colour_styles
                    [(offset_position - show_avg_offset) % self.styles.cpu_colour_styles.len()]
            };

            // Guard against `cpu_points` and `cpu_entries` diverging; a missing
            // point just means there is nothing to draw yet for this entry.
            let Some(values) = cpu_points.get(current_scroll_position - 1) else {
                return vec![];
            };

            let mut gd = GraphData::default().style(style).time(time).values(values);
            if name_entries {
                gd = gd.name(cpu_legend_label(entry.data_type, entry.usage, show_decimal));
            }
            vec![gd]
        } else {
            vec![]
        }
    }

    fn draw_cpu_graph(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(cpu_widget_state) = app_state.states.cpu_state.widget_states.get_mut(&widget_id)
        {
            let data = app_state.data_store.get_data();

            let hide_x_labels = should_hide_x_label(
                app_state.app_config_fields.hide_time,
                app_state.app_config_fields.autohide_time,
                cpu_widget_state.graph.state_mut().autohide_timer_mut(),
                draw_loc,
            );

            let legend_mode = app_state.app_config_fields.cpu_legend_mode;
            // Derive everything the draw path needs from the mode in one
            // exhaustive match, so adding a mode is a compile error here rather
            // than silently disabling a constraint elsewhere.
            let (legend_position, name_entries, legend_constraints) = match legend_mode {
                CpuLegendMode::Table => (None, false, None),
                CpuLegendMode::Overlay(pos) => (
                    Some(pos),
                    true,
                    Some(LegendConstraints {
                        width: Constraint::Ratio(3, 4),
                        height: Constraint::Ratio(3, 4),
                    }),
                ),
                CpuLegendMode::Hidden => (None, false, None),
            };

            let graph_data = self.generate_points(
                cpu_widget_state,
                data,
                app_state.app_config_fields.show_average_cpu,
                app_state.app_config_fields.show_cpu_decimal,
                name_entries,
            );

            // TODO: Maybe hide load avg if too long? Or maybe the CPU part.
            let title = {
                #[cfg(unix)]
                {
                    let load_avg = &data.load_avg_harvest;
                    let load_avg_str = format!(
                        "─ {:.2} {:.2} {:.2} ",
                        load_avg[0], load_avg[1], load_avg[2]
                    );

                    concat_string::concat_string!(" CPU ", load_avg_str).into()
                }
                #[cfg(not(target_family = "unix"))]
                {
                    " CPU ".into()
                }
            };

            let border_style = self.get_border_style(widget_id, app_state.current_widget.widget_id);
            let marker = self.get_marker(app_state.app_config_fields.use_dot);

            cpu_widget_state.graph.draw(
                f,
                draw_loc,
                GraphDrawCtx {
                    title,
                    border_style,
                    title_style: self.styles.widget_title_style,
                    graph_style: self.styles.graph_style,
                    general_widget_style: self.styles.general_widget_style,
                    border_type: self.styles.border_type,
                    marker,
                    hide_x_labels,
                    is_selected: app_state.current_widget.widget_id == widget_id,
                    is_expanded: app_state.is_expanded,
                    legend_position,
                    legend_constraints,
                },
                graph_data,
            );
        }
    }

    fn draw_cpu_legend(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let recalculate_column_widths = app_state.should_get_widget_bounds();
        if let Some(cpu_widget_state) = app_state
            .states
            .cpu_state
            .widget_states
            .get_mut(&(widget_id - 1))
        {
            // TODO: This line (and the one above, see caller) is pretty dumb
            // but I guess needed for now. Refactor if possible!
            cpu_widget_state.is_legend_hidden = false;

            let is_on_widget = widget_id == app_state.current_widget.widget_id;

            let draw_info = DrawInfo {
                loc: draw_loc,
                force_redraw: app_state.is_force_redraw,
                recalculate_column_widths,
                // TODO: Bug with this, shouldn't be selected on expand!
                selection_state: SelectionState::new(app_state.is_expanded, is_on_widget),
            };

            cpu_widget_state.table.draw(
                f,
                &draw_info,
                app_state.widget_map.get_mut(&widget_id),
                self,
            );
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use timeless::data::ChunkedData;

    use super::*;
    use crate::{
        app::layout_manager::BottomLayout,
        collection::cpu::{CpuData, CpuDataType},
        options::config::{cpu::CpuDefault, style::Styles},
    };

    fn painter() -> Painter {
        Painter::init(
            BottomLayout {
                rows: vec![],
                total_row_height_ratio: 1,
            },
            Styles::default(),
        )
        .expect("painter can be created")
    }

    fn cpu_points(num: usize) -> Vec<ChunkedData<f64>> {
        (0..num)
            .map(|i| {
                let mut points = ChunkedData::default();
                points.push((i + 1) as f64);
                points
            })
            .collect()
    }

    /// Builds a widget state and data for the given CPU entries, with the
    /// selected table index set to `index`.
    fn setup(index: usize, entries: &[(CpuDataType, f32)]) -> (CpuWidgetState, InnerData) {
        let config = crate::app::AppConfigFields::default();
        let mut state = CpuWidgetState::new(&config, CpuDefault::All, None, &Styles::default());
        state.table.state.current_index = index;

        let mut data = InnerData::default();
        data.cpu_harvest = entries
            .iter()
            .map(|&(data_type, usage)| CpuData { data_type, usage })
            .collect();
        data.time_series_data.cpu = cpu_points(entries.len());
        data.time_series_data.time = vec![Instant::now()];

        (state, data)
    }

    fn named(graph_data: &[GraphData<'_>]) -> Vec<String> {
        graph_data
            .iter()
            .filter_map(|gd| gd.get_name().map(|n| n.to_string()))
            .collect()
    }

    #[test]
    fn all_view_names_the_average_when_shown() {
        let (state, data) = setup(
            ALL_POSITION,
            &[
                (CpuDataType::Avg, 50.0),
                (CpuDataType::Cpu(0), 10.0),
                (CpuDataType::Cpu(1), 20.0),
            ],
        );
        let points = painter().generate_points(&state, &data, true, false, true);

        // Only the average (the default selected row) is labelled; the rest are
        // drawn but not listed.
        assert_eq!(points.len(), 3);
        assert_eq!(named(&points), ["AVG  50%"]);
    }

    #[test]
    fn all_view_falls_back_to_first_cpu_when_average_hidden() {
        // With `--hide_avg_cpu` the harvest has no average entry, so the legend
        // must not be left empty.
        let (state, data) = setup(
            ALL_POSITION,
            &[(CpuDataType::Cpu(0), 10.0), (CpuDataType::Cpu(1), 20.0)],
        );
        let points = painter().generate_points(&state, &data, false, false, true);

        assert_eq!(named(&points), ["CPU0  10%"]);
    }

    #[test]
    fn single_view_names_the_selected_entry() {
        let (state, data) = setup(
            2,
            &[
                (CpuDataType::Avg, 50.0),
                (CpuDataType::Cpu(0), 10.0),
                (CpuDataType::Cpu(1), 20.0),
            ],
        );
        let points = painter().generate_points(&state, &data, true, false, true);

        assert_eq!(points.len(), 1);
        assert_eq!(named(&points), ["CPU0  10%"]);
    }

    #[test]
    fn labels_respect_the_decimal_setting() {
        let (state, data) = setup(ALL_POSITION, &[(CpuDataType::Avg, 12.44)]);
        let points = painter().generate_points(&state, &data, true, true, true);

        assert_eq!(named(&points), ["AVG 12.4%"]);
    }

    #[test]
    fn naming_is_disabled_outside_overlay_mode() {
        let (state, data) = setup(ALL_POSITION, &[(CpuDataType::Avg, 50.0)]);
        let points = painter().generate_points(&state, &data, true, false, false);

        assert!(named(&points).is_empty());
    }

    #[test]
    fn single_view_handles_diverged_point_and_entry_vectors() {
        // `cpu_entries` may have an entry while `cpu_points` does not yet; this
        // used to be an unchecked index that could panic.
        let (state, data) = setup(1, &[(CpuDataType::Avg, 50.0)]);
        let mut data = data;
        data.time_series_data.cpu.clear();

        assert!(
            painter()
                .generate_points(&state, &data, true, false, true)
                .is_empty()
        );
    }
}
