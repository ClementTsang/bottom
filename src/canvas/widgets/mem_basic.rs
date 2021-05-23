use crate::{
    app::AppState,
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

pub trait MemBasicWidget {
    fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    );
}

impl MemBasicWidget for Painter {
    fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    ) {
        let mem_data: &[(f64, f64)] = &app_state.canvas_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.canvas_data.swap_data;

        let margined_loc = Layout::default()
            .constraints([Constraint::Percentage(100)])
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
            if let Some((_label_percent, label_frac)) = &app_state.canvas_data.mem_labels {
                label_frac.trim()
            } else {
                EMPTY_MEMORY_FRAC_STRING
            };

        let trimmed_swap_frac =
            if let Some((_label_percent, label_frac)) = &app_state.canvas_data.swap_labels {
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

        let mem_text = vec![
            Spans::from(Span::styled(mem_label, self.colours.ram_style)),
            Spans::from(Span::styled(swap_label, self.colours.swap_style)),
        ];

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
