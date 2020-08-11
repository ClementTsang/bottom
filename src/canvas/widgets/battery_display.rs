use crate::{
    app::App,
    canvas::{drawing_utils::calculate_basic_use_bars, Painter},
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
};

pub trait BatteryDisplayWidget {
    fn draw_battery_display<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl BatteryDisplayWidget for Painter {
    fn draw_battery_display<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(battery_widget_state) =
            app_state.battery_state.widget_states.get_mut(&widget_id)
        {
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let border_and_title_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Battery ── Esc to go back ";
                Span::styled(
                    format!(
                        " Battery ─{}─ Esc to go back ",
                        "─".repeat(
                            usize::from(draw_loc.width)
                                .saturating_sub(TITLE_BASE.chars().count() + 2)
                        )
                    ),
                    border_and_title_style,
                )
            } else {
                Span::styled(" Battery ".to_string(), self.colours.widget_title_style)
            };

            let battery_block = if draw_border {
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_and_title_style)
            } else if is_on_widget {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            if let Some(battery_details) = app_state
                .canvas_data
                .battery_data
                .get(battery_widget_state.currently_selected_battery_index)
            {
                // Assuming a 50/50 split in width
                let bar_length =
                    usize::from((draw_loc.width.saturating_sub(2) / 2).saturating_sub(8));
                let charge_percentage = battery_details.charge_percentage;
                let num_bars = calculate_basic_use_bars(charge_percentage, bar_length);
                let bars = format!(
                    "[{}{}{:3.0}%]",
                    "|".repeat(num_bars),
                    " ".repeat(bar_length - num_bars),
                    charge_percentage,
                );

                let battery_items = vec![
                    ["Charge %", &bars],
                    ["Consumption", &battery_details.watt_consumption],
                    if let Some(duration_until_full) = &battery_details.duration_until_full {
                        ["Time to full", duration_until_full]
                    } else if let Some(duration_until_empty) = &battery_details.duration_until_empty
                    {
                        ["Time to empty", duration_until_empty]
                    } else {
                        ["Time to full/empty", "N/A"]
                    },
                    ["Health %", &battery_details.health],
                ];

                let battery_rows = battery_items.iter().enumerate().map(|(itx, item)| {
                    Row::StyledData(
                        item.iter(),
                        if itx == 0 {
                            let colour_index = ((charge_percentage
                                * self.colours.battery_bar_styles.len() as f64
                                - 1.0)
                                / 100.0)
                                .floor() as usize;
                            *self
                                .colours
                                .battery_bar_styles
                                .get(colour_index)
                                .unwrap_or(&self.colours.text_style)
                        } else {
                            self.colours.text_style
                        },
                    )
                });

                // Draw
                f.render_widget(
                    Table::new([""].iter(), battery_rows)
                        .block(battery_block.clone())
                        .header_style(self.colours.table_header_style)
                        .widths([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref()),
                    draw_loc,
                );
            } else {
                f.render_widget(
                    Paragraph::new(Spans::from(vec![Span::styled(
                        "No data found for this battery",
                        self.colours.text_style,
                    )]))
                    .block(battery_block.clone()),
                    draw_loc,
                );
            }
            f.render_widget(
                Tabs::new(
                    (app_state
                        .canvas_data
                        .battery_data
                        .iter()
                        .map(|battery| Spans::from((&battery).battery_name.clone())))
                    .collect::<Vec<_>>(),
                )
                .block(battery_block)
                .divider(tui::symbols::line::VERTICAL)
                .style(self.colours.text_style)
                .highlight_style(self.colours.currently_selected_text_style)
                .select(battery_widget_state.currently_selected_battery_index),
                draw_loc,
            );
        }
    }
}
