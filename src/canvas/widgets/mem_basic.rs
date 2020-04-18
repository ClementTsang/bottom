use std::cmp::max;

use crate::{
    app::App,
    canvas::{drawing_utils::*, Painter},
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Paragraph, Text},
};

pub trait MemBasicWidget {
    fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl MemBasicWidget for Painter {
    fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let mem_data: &[(f64, f64)] = &app_state.canvas_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.canvas_data.swap_data;

        let margined_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(draw_loc);

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                draw_loc,
            );
        }

        // +9 due to 3 + 4 + 2 + 2 columns for the name & space + percentage + bar bounds + margin spacing
        let bar_length = max(0, draw_loc.width as i64 - 11) as usize;
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
        let num_bars_ram = calculate_basic_use_bars(ram_use_percentage, bar_length);
        let num_bars_swap = calculate_basic_use_bars(swap_use_percentage, bar_length);
        let mem_label = format!(
            "RAM[{}{}{:3.0}%]\n",
            "|".repeat(num_bars_ram),
            " ".repeat(bar_length - num_bars_ram),
            ram_use_percentage.round(),
        );
        let swap_label = format!(
            "SWP[{}{}{:3.0}%]",
            "|".repeat(num_bars_swap),
            " ".repeat(bar_length - num_bars_swap),
            swap_use_percentage.round(),
        );

        let mem_text: Vec<Text<'_>> = vec![
            Text::Styled(mem_label.into(), self.colours.ram_style),
            Text::Styled(swap_label.into(), self.colours.swap_style),
        ];

        f.render_widget(
            Paragraph::new(mem_text.iter()).block(Block::default()),
            margined_loc[0],
        );
    }
}
