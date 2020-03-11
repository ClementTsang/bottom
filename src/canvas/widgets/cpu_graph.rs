use std::borrow::Cow;
use std::cmp::max;

use crate::{
    app::{App, WidgetPosition},
    canvas::{
        drawing_utils::{get_start_position, get_variable_intrinsic_widths},
        Painter,
    },
    constants::*,
    data_conversion::ConvertedCpuData,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
};

const CPU_SELECT_LEGEND_HEADER: [&str; 2] = ["CPU", "Show"];
const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
lazy_static! {
    static ref CPU_LEGEND_HEADER_LENS: Vec<usize> = CPU_LEGEND_HEADER
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref CPU_SELECT_LEGEND_HEADER_LENS: Vec<usize> = CPU_SELECT_LEGEND_HEADER
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
}

pub trait CpuGraphWidget {
    fn draw_cpu_graph<B: Backend>(&self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect);
    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    );
}

impl CpuGraphWidget for Painter {
    fn draw_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

        let display_time_labels = [
            format!("{}s", app_state.cpu_state.display_time / 1000),
            "0s".to_string(),
        ];

        let x_axis = if app_state.app_config_fields.hide_time
            || (app_state.app_config_fields.autohide_time
                && app_state.cpu_state.display_time_instant.is_none())
        {
            Axis::default().bounds([0.0, app_state.cpu_state.display_time as f64])
        } else if let Some(time) = app_state.cpu_state.display_time_instant {
            if std::time::Instant::now().duration_since(time).as_millis()
                < AUTOHIDE_TIMEOUT_MILLISECONDS as u128
            {
                Axis::default()
                    .bounds([0.0, app_state.cpu_state.display_time as f64])
                    .style(self.colours.graph_style)
                    .labels_style(self.colours.graph_style)
                    .labels(&display_time_labels)
            } else {
                app_state.cpu_state.display_time_instant = None;
                Axis::default().bounds([0.0, app_state.cpu_state.display_time as f64])
            }
        } else {
            Axis::default()
                .bounds([0.0, app_state.cpu_state.display_time as f64])
                .style(self.colours.graph_style)
                .labels_style(self.colours.graph_style)
                .labels(&display_time_labels)
        };

        // Note this is offset as otherwise the 0 value is not drawn!
        let y_axis = Axis::default()
            .style(self.colours.graph_style)
            .labels_style(self.colours.graph_style)
            .bounds([-0.5, 100.5])
            .labels(&["0%", "100%"]);

        let dataset_vector: Vec<Dataset<'_>> = cpu_data
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(itx, cpu)| {
                if app_state.cpu_state.core_show_vec[itx] {
                    Some(
                        Dataset::default()
                            .marker(if app_state.app_config_fields.use_dot {
                                Marker::Dot
                            } else {
                                Marker::Braille
                            })
                            .style(
                                if app_state.app_config_fields.show_average_cpu && itx == 0 {
                                    self.colours.avg_colour_style
                                } else {
                                    self.colours.cpu_colour_styles
                                        [itx % self.colours.cpu_colour_styles.len()]
                                },
                            )
                            .data(&cpu.cpu_data[..]),
                    )
                } else {
                    None
                }
            })
            .collect();

        let title = if app_state.is_expanded && !app_state.cpu_state.is_showing_tray {
            const TITLE_BASE: &str = " CPU ── Esc to go back ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title =
                format!(" CPU ─{}─ Esc to go back ", "─".repeat(repeat_num as usize));

            result_title
        } else {
            " CPU ".to_string()
        };

        let border_style = match app_state.current_widget_selected {
            WidgetPosition::Cpu => self.colours.highlighted_border_style,
            _ => self.colours.border_style,
        };

        Chart::default()
            .block(
                Block::default()
                    .title(&title)
                    .title_style(if app_state.is_expanded {
                        border_style
                    } else {
                        self.colours.widget_title_style
                    })
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
            .datasets(&dataset_vector)
            .render(f, draw_loc);
    }

    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

        let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
        let start_position = get_start_position(
            num_rows,
            &app_state.app_scroll_positions.scroll_direction,
            &mut app_state
                .app_scroll_positions
                .cpu_scroll_state
                .previous_scroll_position,
            app_state
                .app_scroll_positions
                .cpu_scroll_state
                .current_scroll_position,
            app_state.is_resized,
        );

        let sliced_cpu_data = &cpu_data[start_position as usize..];
        let mut stringified_cpu_data: Vec<Vec<Cow<'_, str>>> = Vec::new();

        for (itx, cpu) in sliced_cpu_data.iter().enumerate() {
            if app_state.cpu_state.is_showing_tray {
                stringified_cpu_data.push(vec![
                    Cow::Borrowed(&cpu.cpu_name),
                    if app_state.cpu_state.core_show_vec[itx + start_position as usize] {
                        "[*]".into()
                    } else {
                        "[ ]".into()
                    },
                ]);
            } else if app_state.app_config_fields.show_disabled_data
                || app_state.cpu_state.core_show_vec[itx]
            {
                stringified_cpu_data.push(vec![
                    Cow::Borrowed(&cpu.cpu_name),
                    Cow::Borrowed(&cpu.legend_value),
                ]);
            } else {
                stringified_cpu_data.push(Vec::new());
            }
        }

        let cpu_rows =
            stringified_cpu_data
                .iter()
                .enumerate()
                .filter_map(|(itx, cpu_string_row)| {
                    if cpu_string_row.is_empty() {
                        None
                    } else {
                        Some(Row::StyledData(
                            cpu_string_row.iter(),
                            match app_state.current_widget_selected {
                                WidgetPosition::CpuLegend => {
                                    if itx as u64
                                        == app_state
                                            .app_scroll_positions
                                            .cpu_scroll_state
                                            .current_scroll_position
                                            - start_position
                                    {
                                        self.colours.currently_selected_text_style
                                    } else if app_state.app_config_fields.show_average_cpu
                                        && itx == 0
                                    {
                                        self.colours.avg_colour_style
                                    } else {
                                        self.colours.cpu_colour_styles[itx
                                            + start_position as usize
                                                % self.colours.cpu_colour_styles.len()]
                                    }
                                }
                                _ => {
                                    if app_state.app_config_fields.show_average_cpu && itx == 0 {
                                        self.colours.avg_colour_style
                                    } else {
                                        self.colours.cpu_colour_styles[itx
                                            + start_position as usize
                                                % self.colours.cpu_colour_styles.len()]
                                    }
                                }
                            },
                        ))
                    }
                });

        // Calculate widths
        let width = f64::from(draw_loc.width);
        let width_ratios = vec![0.5, 0.5];

        let variable_intrinsic_results = get_variable_intrinsic_widths(
            width as u16,
            &width_ratios,
            if app_state.cpu_state.is_showing_tray {
                &CPU_SELECT_LEGEND_HEADER_LENS
            } else {
                &CPU_LEGEND_HEADER_LENS
            },
        );
        let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

        let title = if app_state.cpu_state.is_showing_tray {
            const TITLE_BASE: &str = " Esc to close ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title = format!("{} Esc to close ", "─".repeat(repeat_num as usize));

            result_title
        } else {
            "".to_string()
        };

        let title_and_border_style = match app_state.current_widget_selected {
            WidgetPosition::CpuLegend => self.colours.highlighted_border_style,
            _ => self.colours.border_style,
        };

        // Draw
        Table::new(
            if app_state.cpu_state.is_showing_tray {
                CPU_SELECT_LEGEND_HEADER
            } else {
                CPU_LEGEND_HEADER
            }
            .iter(),
            cpu_rows,
        )
        .block(
            Block::default()
                .title(&title)
                .title_style(title_and_border_style)
                .borders(Borders::ALL)
                .border_style(title_and_border_style),
        )
        .header_style(self.colours.table_header_style)
        .widths(
            &(intrinsic_widths
                .iter()
                .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                .collect::<Vec<_>>()),
        )
        .render(f, draw_loc);
    }
}
