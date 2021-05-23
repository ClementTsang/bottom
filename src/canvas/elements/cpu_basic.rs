use std::cmp::min;

use crate::{
    app::AppState,
    canvas::{drawing_utils::*, Painter},
    constants::*,
    data_conversion::ConvertedCpuData,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Paragraph},
};

pub trait CpuBasicWidget {
    fn draw_basic_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    );
}

impl CpuBasicWidget for Painter {
    fn draw_basic_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    ) {
        // Skip the first element, it's the "all" element
        if app_state.canvas_data.cpu_data.len() > 1 {
            let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data[1..];

            // This is a bit complicated, but basically, we want to draw SOME number
            // of columns to draw all CPUs.  Ideally, as well, we want to not have
            // to ever scroll.
            // **General logic** - count number of elements in cpu_data.  Then see how
            // many rows and columns we have in draw_loc (-2 on both sides for border?).
            // I think what we can do is try to fit in as many in one column as possible.
            // If not, then add a new column.
            // Then, from this, split the row space across ALL columns.  From there, generate
            // the desired lengths.

            if app_state.current_widget.widget_id == widget_id {
                f.render_widget(
                    Block::default()
                        .borders(*SIDE_BORDERS)
                        .border_style(self.colours.highlighted_border_style),
                    draw_loc,
                );
            }

            let num_cpus = cpu_data.len();
            let show_avg_cpu = app_state.app_config_fields.show_average_cpu;

            if draw_loc.height > 0 {
                let remaining_height = usize::from(draw_loc.height);
                const REQUIRED_COLUMNS: usize = 4;

                let chunk_vec =
                    vec![Constraint::Percentage((100 / REQUIRED_COLUMNS) as u16); REQUIRED_COLUMNS];
                let chunks = Layout::default()
                    .constraints(chunk_vec)
                    .direction(Direction::Horizontal)
                    .split(draw_loc);

                const CPU_NAME_SPACE: usize = 3;
                const BAR_BOUND_SPACE: usize = 2;
                const PERCENTAGE_SPACE: usize = 4;
                const MARGIN_SPACE: usize = 2;

                const COMBINED_SPACING: usize =
                    CPU_NAME_SPACE + BAR_BOUND_SPACE + PERCENTAGE_SPACE + MARGIN_SPACE;
                const REDUCED_SPACING: usize = CPU_NAME_SPACE + PERCENTAGE_SPACE + MARGIN_SPACE;
                let chunk_width = chunks[0].width as usize;

                // Inspired by htop.
                // We do +4 as if it's too few bars in the bar length, it's kinda pointless.
                let cpu_bars = if chunk_width >= COMBINED_SPACING + 4 {
                    let bar_length = chunk_width - COMBINED_SPACING;
                    (0..num_cpus)
                        .map(|cpu_index| {
                            let use_percentage =
                                if let Some(cpu_usage) = cpu_data[cpu_index].cpu_data.last() {
                                    cpu_usage.1
                                } else {
                                    0.0
                                };

                            let num_bars = calculate_basic_use_bars(use_percentage, bar_length);
                            format!(
                                "{:3}[{}{}{:3.0}%]",
                                if app_state.app_config_fields.show_average_cpu {
                                    if cpu_index == 0 {
                                        "AVG".to_string()
                                    } else {
                                        (cpu_index - 1).to_string()
                                    }
                                } else {
                                    cpu_index.to_string()
                                },
                                "|".repeat(num_bars),
                                " ".repeat(bar_length - num_bars),
                                use_percentage.round(),
                            )
                        })
                        .collect::<Vec<_>>()
                } else if chunk_width >= REDUCED_SPACING {
                    (0..num_cpus)
                        .map(|cpu_index| {
                            let use_percentage =
                                if let Some(cpu_usage) = cpu_data[cpu_index].cpu_data.last() {
                                    cpu_usage.1
                                } else {
                                    0.0
                                };

                            format!(
                                "{:3} {:3.0}%",
                                if app_state.app_config_fields.show_average_cpu {
                                    if cpu_index == 0 {
                                        "AVG".to_string()
                                    } else {
                                        (cpu_index - 1).to_string()
                                    }
                                } else {
                                    cpu_index.to_string()
                                },
                                use_percentage.round(),
                            )
                        })
                        .collect::<Vec<_>>()
                } else {
                    (0..num_cpus)
                        .map(|cpu_index| {
                            let use_percentage =
                                if let Some(cpu_usage) = cpu_data[cpu_index].cpu_data.last() {
                                    cpu_usage.1
                                } else {
                                    0.0
                                };

                            format!("{:3.0}%", use_percentage.round(),)
                        })
                        .collect::<Vec<_>>()
                };

                let mut row_counter = num_cpus;
                let mut start_index = 0;
                for (itx, chunk) in chunks.iter().enumerate() {
                    // Explicitly check... don't want an accidental DBZ or underflow, this ensures
                    // to_divide is > 0
                    if REQUIRED_COLUMNS > itx {
                        let to_divide = REQUIRED_COLUMNS - itx;
                        let how_many_cpus = min(
                            remaining_height,
                            (row_counter / to_divide)
                                + (if row_counter % to_divide == 0 { 0 } else { 1 }),
                        );
                        row_counter -= how_many_cpus;
                        let end_index = min(start_index + how_many_cpus, num_cpus);

                        let cpu_column = (start_index..end_index)
                            .map(|itx| {
                                Spans::from(Span {
                                    content: (&cpu_bars[itx]).into(),
                                    style: if show_avg_cpu {
                                        if itx == 0 {
                                            self.colours.avg_colour_style
                                        } else {
                                            self.colours.cpu_colour_styles
                                                [(itx - 1) % self.colours.cpu_colour_styles.len()]
                                        }
                                    } else {
                                        self.colours.cpu_colour_styles
                                            [itx % self.colours.cpu_colour_styles.len()]
                                    },
                                })
                            })
                            .collect::<Vec<_>>();

                        start_index += how_many_cpus;

                        let margined_loc = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(100)])
                            .horizontal_margin(1)
                            .split(*chunk)[0];

                        f.render_widget(
                            Paragraph::new(cpu_column).block(Block::default()),
                            margined_loc,
                        );
                    }
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
