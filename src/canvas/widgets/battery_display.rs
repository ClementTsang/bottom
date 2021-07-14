use crate::{
    app::AppState,
    canvas::{drawing_utils::calculate_basic_use_bars, Painter},
    constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
};
use unicode_segmentation::UnicodeSegmentation;

pub trait BatteryDisplayWidget {
    fn draw_battery_display<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl BatteryDisplayWidget for Painter {
    fn draw_battery_display<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(battery_widget_state) =
            app_state.battery_state.widget_states.get_mut(&widget_id)
        {
            let is_on_widget = widget_id == app_state.current_widget.widget_id;
            let border_style = if is_on_widget {
                self.colours.highlighted_border_style
            } else {
                self.colours.border_style
            };
            let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                app_state.app_config_fields.table_gap
            };

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Battery ── Esc to go back ";
                Spans::from(vec![
                    Span::styled(" Battery ".to_string(), self.colours.widget_title_style),
                    Span::styled(
                        format!(
                            "─{}─ Esc to go back ",
                            "─".repeat(usize::from(draw_loc.width).saturating_sub(
                                UnicodeSegmentation::graphemes(TITLE_BASE, true).count() + 2
                            ))
                        ),
                        border_style,
                    ),
                ])
            } else {
                Spans::from(Span::styled(
                    " Battery ".to_string(),
                    self.colours.widget_title_style,
                ))
            };

            let battery_block = if draw_border {
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            } else if is_on_widget {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            let battery_names = app_state
                .canvas_data
                .battery_data
                .iter()
                .map(|battery| &battery.battery_name)
                .collect::<Vec<_>>();

            let tab_draw_loc = Layout::default()
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Min(0),
                ])
                .direction(Direction::Vertical)
                .split(draw_loc)[1];

            f.render_widget(
                Tabs::new(
                    battery_names
                        .iter()
                        .map(|name| Spans::from((*name).clone()))
                        .collect::<Vec<_>>(),
                )
                .block(Block::default())
                .divider(tui::symbols::line::VERTICAL)
                .style(self.colours.text_style)
                .highlight_style(self.colours.currently_selected_text_style)
                .select(battery_widget_state.currently_selected_battery_index),
                tab_draw_loc,
            );

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(if is_on_widget || draw_border { 0 } else { 1 })
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

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

                let battery_rows = vec![
                    Row::new(vec![
                        Cell::from("Charge %").style(self.colours.text_style),
                        Cell::from(bars).style(if charge_percentage < 10.0 {
                            self.colours.low_battery_colour
                        } else if charge_percentage < 50.0 {
                            self.colours.medium_battery_colour
                        } else {
                            self.colours.high_battery_colour
                        }),
                    ]),
                    Row::new(vec!["Consumption", &battery_details.watt_consumption])
                        .style(self.colours.text_style),
                    if let Some(duration_until_full) = &battery_details.duration_until_full {
                        Row::new(vec!["Time to full", duration_until_full])
                            .style(self.colours.text_style)
                    } else if let Some(duration_until_empty) = &battery_details.duration_until_empty
                    {
                        Row::new(vec!["Time to empty", duration_until_empty])
                            .style(self.colours.text_style)
                    } else {
                        Row::new(vec!["Time to full/empty", "N/A"]).style(self.colours.text_style)
                    },
                    Row::new(vec!["Health %", &battery_details.health])
                        .style(self.colours.text_style),
                ];

                // Draw
                f.render_widget(
                    Table::new(battery_rows)
                        .block(battery_block)
                        .header(Row::new(vec![""]).bottom_margin(table_gap))
                        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]),
                    margined_draw_loc,
                );
            } else {
                let mut contents = vec![Spans::default(); table_gap as usize];

                contents.push(Spans::from(Span::styled(
                    "No data found for this battery",
                    self.colours.text_style,
                )));

                f.render_widget(
                    Paragraph::new(contents).block(battery_block),
                    margined_draw_loc,
                );
            }

            if should_get_widget_bounds {
                // Tab wizardry
                if !battery_names.is_empty() {
                    let mut current_x = tab_draw_loc.x;
                    let current_y = tab_draw_loc.y;
                    let mut tab_click_locs: Vec<((u16, u16), (u16, u16))> = vec![];
                    for battery in battery_names {
                        // +1 because there's a space after the tab label.
                        let width = unicode_width::UnicodeWidthStr::width(battery.as_str()) as u16;
                        tab_click_locs
                            .push(((current_x, current_y), (current_x + width, current_y)));

                        // +4 because we want to go one space, then one space past to get to the '|', then 2 more
                        // to start at the blank space before the tab label.
                        current_x += width + 4;
                    }
                    battery_widget_state.tab_click_locs = Some(tab_click_locs);
                }

                // Update draw loc in widget map
                if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                    widget.top_left_corner = Some((margined_draw_loc.x, margined_draw_loc.y));
                    widget.bottom_right_corner = Some((
                        margined_draw_loc.x + margined_draw_loc.width,
                        margined_draw_loc.y + margined_draw_loc.height,
                    ));
                }
            }
        }
    }
}
