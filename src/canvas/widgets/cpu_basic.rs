use std::cmp::min;

use crate::{
    app::App,
    canvas::{drawing_utils::*, Painter},
    constants::*,
    data_conversion::ConvertedCpuData,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Paragraph, Text},
};

pub trait CpuBasicWidget {
    fn draw_basic_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl CpuBasicWidget for Painter {
    fn draw_basic_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

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
        if draw_loc.height > 0 {
            let remaining_height = draw_loc.height as usize;
            const REQUIRED_COLUMNS: usize = 4;

            let chunk_vec =
                vec![Constraint::Percentage((100 / REQUIRED_COLUMNS) as u16); REQUIRED_COLUMNS];
            let chunks = Layout::default()
                .constraints(chunk_vec.as_ref())
                .direction(Direction::Horizontal)
                .split(draw_loc);

            // +9 due to 3 + 4 + 2 columns for the name & space + percentage + bar bounds
            const MARGIN_SPACE: usize = 2;
            let remaining_width = usize::from(draw_loc.width)
                .saturating_sub((9 + MARGIN_SPACE) * REQUIRED_COLUMNS - MARGIN_SPACE)
                as usize;

            let bar_length = remaining_width / REQUIRED_COLUMNS;

            // CPU (and RAM) percent bars are, uh, "heavily" inspired from htop.
            let cpu_bars = (0..num_cpus)
                .map(|cpu_index| {
                    let use_percentage =
                        if let Some(cpu_usage) = cpu_data[cpu_index].cpu_data.last() {
                            cpu_usage.1
                        } else {
                            0.0
                        };

                    let num_bars = calculate_basic_use_bars(use_percentage, bar_length);
                    format!(
                        "{:3}[{}{}{:3.0}%]\n",
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
                .collect::<Vec<_>>();

            let mut row_counter = num_cpus;
            let mut start_index = 0;
            for (itx, chunk) in chunks.iter().enumerate() {
                // Explicitly check... don't want an accidental DBZ or underflow
                if REQUIRED_COLUMNS > itx {
                    let to_divide = REQUIRED_COLUMNS - itx;
                    let how_many_cpus = min(
                        remaining_height,
                        (row_counter / to_divide)
                            + (if row_counter % to_divide == 0 { 0 } else { 1 }),
                    );
                    row_counter -= how_many_cpus;
                    let end_index = min(start_index + how_many_cpus, num_cpus);
                    let cpu_column: Vec<Text<'_>> = (start_index..end_index)
                        .map(|cpu_index| {
                            Text::Styled(
                                (&cpu_bars[cpu_index]).into(),
                                self.colours.cpu_colour_styles
                                    [cpu_index as usize % self.colours.cpu_colour_styles.len()],
                            )
                        })
                        .collect::<Vec<_>>();

                    start_index += how_many_cpus;

                    let margined_loc = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)].as_ref())
                        .horizontal_margin(1)
                        .split(*chunk);

                    f.render_widget(
                        Paragraph::new(cpu_column.iter()).block(Block::default()),
                        margined_loc[0],
                    );
                }
            }
        }
    }
}
