use std::cmp::max;

use crate::{app::App, canvas::Painter};

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table, Tabs, Widget},
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

            let mut battery_block = Block::default()
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
                let battery_items = vec![
                    vec!["Charge Percent", &battery_details.charge_percentage],
                    vec!["Consumption", &battery_details.watt_consumption],
                    if let Some(duration_until_full) = &battery_details.duration_until_full {
                        vec!["Time to full", duration_until_full]
                    } else if let Some(duration_until_empty) = &battery_details.duration_until_empty
                    {
                        vec!["Time to empty", duration_until_empty]
                    } else {
                        vec!["Time to full", "N/A"]
                    },
                ];

                let battery_rows = battery_items.iter().map(|item| Row::Data(item.iter()));

                // Draw
                Table::new([""].iter(), battery_rows)
                    .block(battery_block)
                    .header_style(self.colours.table_header_style)
                    .widths([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .render(f, draw_loc);

                if app_state.canvas_data.battery_data.len() > 1 {
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
                }
            } else {
                battery_block.render(f, draw_loc);
            }
        }
    }
}
