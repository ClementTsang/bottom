use std::cmp::max;

use crate::{
    app::App,
    canvas::{drawing_utils::calculate_basic_use_bars, Painter},
};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs, Text, Widget},
};

pub trait BatteryDisplayWidget {
    fn draw_battery_display<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    );
}

impl BatteryDisplayWidget for Painter {
    fn draw_battery_display<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(battery_widget_state) =
            app_state.battery_state.widget_states.get_mut(&widget_id)
        {
            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Battery ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!(
                    " Battery ─{}─ Esc to go back ",
                    "─".repeat(repeat_num as usize)
                );
                result_title
            } else {
                " Battery ".to_string()
            };

            let border_and_title_style = if app_state.current_widget.widget_id == widget_id {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };

            let battery_block = Block::default()
                .title(&title)
                .title_style(if app_state.is_expanded {
                    border_and_title_style
                } else {
                    self.colours.widget_title_style
                })
                .borders(Borders::ALL)
                .border_style(border_and_title_style);

            if let Some(battery_details) = app_state
                .canvas_data
                .battery_data
                .get(battery_widget_state.currently_selected_battery_index)
            {
                // Assuming a 50/50 split in width
                let bar_length = max(0, (draw_loc.width as i64 - 2) / 2 - 8) as usize;
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
                ];

                let battery_rows = battery_items.iter().enumerate().map(|(itx, item)| {
                    Row::StyledData(
                        item.iter(),
                        if itx == 0 {
                            let colour_index = ((charge_percentage
                                * self.colours.battery_bar_styles.len() as f64)
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
                Table::new([""].iter(), battery_rows)
                    .block(battery_block)
                    .header_style(self.colours.table_header_style)
                    .widths([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .render(f, draw_loc);
            } else {
                Paragraph::new(
                    [Text::Styled(
                        "No data found for this battery".into(),
                        self.colours.text_style,
                    )]
                    .iter(),
                )
                .block(battery_block)
                .render(f, draw_loc);
            }
            // if app_state.canvas_data.battery_data.len() > 1 {
            Tabs::default()
                .block(battery_block)
                .titles(
                    (app_state
                        .canvas_data
                        .battery_data
                        .iter()
                        .map(|battery| &battery.battery_name))
                    .collect::<Vec<_>>()
                    .as_ref(),
                )
                .divider(tui::symbols::line::VERTICAL)
                .style(self.colours.text_style)
                .highlight_style(self.colours.currently_selected_text_style)
                .select(battery_widget_state.currently_selected_battery_index)
                .render(f, draw_loc);
            // }
        }
    }
}
