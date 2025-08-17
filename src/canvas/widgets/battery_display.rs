use std::cmp::min;

use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, Tabs},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::App,
    canvas::{Painter, drawing_utils::widget_block},
    collection::batteries::BatteryState,
    constants::*,
};

/// Calculate how many bars are to be drawn within basic mode's components.
fn calculate_basic_use_bars(use_percentage: f64, num_bars_available: usize) -> usize {
    min(
        (num_bars_available as f64 * use_percentage / 100.0).round() as usize,
        num_bars_available,
    )
}

impl Painter {
    pub fn draw_battery(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(battery_widget_state) = app_state
            .states
            .battery_state
            .widget_states
            .get_mut(&widget_id)
        {
            let is_selected = widget_id == app_state.current_widget.widget_id;
            let border_style = if is_selected {
                self.styles.highlighted_border_style
            } else {
                self.styles.border_style
            };
            let table_gap = if draw_loc.height < TABLE_GAP_HEIGHT_LIMIT {
                0
            } else {
                app_state.app_config_fields.table_gap
            };

            let block = {
                let mut block = widget_block(
                    app_state.app_config_fields.use_basic_mode,
                    is_selected,
                    self.styles.border_type,
                )
                .border_style(border_style)
                .title_top(Line::styled(" Battery ", self.styles.widget_title_style));

                if app_state.is_expanded {
                    block = block.title_top(
                        Line::styled(" Esc to go back ", self.styles.widget_title_style)
                            .right_aligned(),
                    )
                }

                block
            };

            let battery_harvest = &(app_state.data_store.get_data().battery_harvest);
            if battery_harvest.len() > 1 {
                let battery_names = battery_harvest
                    .iter()
                    .enumerate()
                    .map(|(itx, _)| format!("Battery {itx}"))
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
                            .map(|name| Line::from((*name).clone()))
                            .collect::<Vec<_>>(),
                    )
                    .divider(tui::symbols::line::VERTICAL)
                    .style(self.styles.text_style)
                    .highlight_style(self.styles.selected_text_style)
                    .select(battery_widget_state.currently_selected_battery_index),
                    tab_draw_loc,
                );

                if should_get_widget_bounds {
                    let mut current_x = tab_draw_loc.x;
                    let current_y = tab_draw_loc.y;
                    let mut tab_click_locs: Vec<((u16, u16), (u16, u16))> = vec![];
                    for battery in battery_names {
                        // +1 because there's a space after the tab label.
                        let width = UnicodeWidthStr::width(battery.as_str()) as u16;
                        tab_click_locs
                            .push(((current_x, current_y), (current_x + width, current_y)));

                        // +4 because we want to go one space, then one space past to get to the
                        // '|', then 2 more to start at the blank space
                        // before the tab label.
                        current_x += width + 4;
                    }
                    battery_widget_state.tab_click_locs = Some(tab_click_locs);
                }
            }

            let is_basic = app_state.app_config_fields.use_basic_mode;

            let [margined_draw_loc] = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(u16::from(is_basic && !is_selected))
                .direction(Direction::Horizontal)
                .areas(draw_loc);

            if let Some(battery_details) =
                battery_harvest.get(battery_widget_state.currently_selected_battery_index)
            {
                let full_width = draw_loc.width.saturating_sub(2);
                let bar_length = usize::from(full_width.saturating_sub(6));
                let charge_percent = battery_details.charge_percent;

                let num_bars = calculate_basic_use_bars(charge_percent, bar_length);
                let bars = format!(
                    "[{}{}{:3.0}%]",
                    "|".repeat(num_bars),
                    " ".repeat(bar_length - num_bars),
                    charge_percent,
                );

                let mut battery_charge_rows = Vec::with_capacity(2);
                battery_charge_rows.push(Row::new([
                    Cell::from("Charge").style(self.styles.text_style)
                ]));
                battery_charge_rows.push(Row::new([Cell::from(bars).style(
                    if charge_percent < 10.0 {
                        self.styles.low_battery
                    } else if charge_percent < 50.0 {
                        self.styles.medium_battery
                    } else {
                        self.styles.high_battery
                    },
                )]));

                let mut battery_rows = Vec::with_capacity(3);
                let watt_consumption = battery_details.watt_consumption();
                let health = battery_details.health();

                battery_rows.push(Row::new([""]).bottom_margin(table_gap + 1));
                battery_rows
                    .push(Row::new(["Rate", &watt_consumption]).style(self.styles.text_style));

                battery_rows.push(
                    Row::new(["State", battery_details.state.as_str()])
                        .style(self.styles.text_style),
                );

                let mut time: String; // Keep string lifetime in scope.
                {
                    let style = self.styles.text_style;
                    let time_width = (full_width / 2) as usize;

                    match &battery_details.state {
                        BatteryState::Charging {
                            time_to_full: Some(secs),
                        } => {
                            time = long_time(*secs);

                            if time_width >= time.len() {
                                battery_rows.push(Row::new(["Time to full", &time]).style(style));
                            } else {
                                time = short_time(*secs);
                                battery_rows.push(Row::new(["To full", &time]).style(style));
                            }
                        }
                        BatteryState::Discharging {
                            time_to_empty: Some(secs),
                        } => {
                            time = long_time(*secs);

                            if time_width >= time.len() {
                                battery_rows.push(Row::new(["Time to empty", &time]).style(style));
                            } else {
                                time = short_time(*secs);
                                battery_rows.push(Row::new(["To empty", &time]).style(style));
                            }
                        }
                        _ => {}
                    }
                }

                battery_rows.push(Row::new(["Health", &health]).style(self.styles.text_style));

                let header = if battery_harvest.len() > 1 {
                    Row::new([""]).bottom_margin(table_gap)
                } else {
                    Row::default()
                };

                // Draw bar
                f.render_widget(
                    Table::new(battery_charge_rows, [Constraint::Percentage(100)])
                        .block(block.clone())
                        .header(header.clone()),
                    margined_draw_loc,
                );

                // Draw info
                f.render_widget(
                    Table::new(
                        battery_rows,
                        [Constraint::Percentage(50), Constraint::Percentage(50)],
                    )
                    .block(block)
                    .header(header),
                    margined_draw_loc,
                );
            } else {
                let mut contents = vec![Line::default(); table_gap.into()];

                contents.push(Line::from(Span::styled(
                    "No data found for this battery",
                    self.styles.text_style,
                )));

                f.render_widget(Paragraph::new(contents).block(block), margined_draw_loc);
            }

            if should_get_widget_bounds {
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

fn get_hms(secs: u32) -> (u32, u32, u32) {
    let hours = secs / (60 * 60);
    let minutes = (secs / 60) - hours * 60;
    let seconds = secs - minutes * 60 - hours * 60 * 60;

    (hours, minutes, seconds)
}

fn long_time(secs: u32) -> String {
    let (hours, minutes, seconds) = get_hms(secs);

    if hours > 0 {
        let h = if hours == 1 { "hour" } else { "hours" };
        let m = if minutes == 1 { "minute" } else { "minutes" };
        let s = if seconds == 1 { "second" } else { "seconds" };

        format!("{hours} {h}, {minutes} {m}, {seconds} {s}")
    } else {
        let m = if minutes == 1 { "minute" } else { "minutes" };
        let s = if seconds == 1 { "second" } else { "seconds" };

        format!("{minutes} {m}, {seconds} {s}")
    }
}

fn short_time(secs: u32) -> String {
    let (hours, minutes, seconds) = get_hms(secs);

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else {
        format!("{minutes}m {seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hms() {
        assert_eq!(get_hms(10), (0, 0, 10));
        assert_eq!(get_hms(60), (0, 1, 0));
        assert_eq!(get_hms(61), (0, 1, 1));
        assert_eq!(get_hms(3600), (1, 0, 0));
        assert_eq!(get_hms(3601), (1, 0, 1));
        assert_eq!(get_hms(3661), (1, 1, 1));
    }

    #[test]
    fn test_long_time() {
        assert_eq!(long_time(1), "0 minutes, 1 second".to_string());
        assert_eq!(long_time(10), "0 minutes, 10 seconds".to_string());
        assert_eq!(long_time(60), "1 minute, 0 seconds".to_string());
        assert_eq!(long_time(61), "1 minute, 1 second".to_string());
        assert_eq!(long_time(3600), "1 hour, 0 minutes, 0 seconds".to_string());
        assert_eq!(long_time(3601), "1 hour, 0 minutes, 1 second".to_string());
        assert_eq!(long_time(3661), "1 hour, 1 minute, 1 second".to_string());
    }

    #[test]
    fn test_short_time() {
        assert_eq!(short_time(1), "0m 1s".to_string());
        assert_eq!(short_time(10), "0m 10s".to_string());
        assert_eq!(short_time(60), "1m 0s".to_string());
        assert_eq!(short_time(61), "1m 1s".to_string());
        assert_eq!(short_time(3600), "1h 0m 0s".to_string());
        assert_eq!(short_time(3601), "1h 0m 1s".to_string());
        assert_eq!(short_time(3661), "1h 1m 1s".to_string());
    }

    #[test]
    fn test_calculate_basic_use_bars() {
        // Testing various breakpoints and edge cases.
        assert_eq!(calculate_basic_use_bars(0.0, 15), 0);
        assert_eq!(calculate_basic_use_bars(1.0, 15), 0);
        assert_eq!(calculate_basic_use_bars(5.0, 15), 1);
        assert_eq!(calculate_basic_use_bars(10.0, 15), 2);
        assert_eq!(calculate_basic_use_bars(40.0, 15), 6);
        assert_eq!(calculate_basic_use_bars(45.0, 15), 7);
        assert_eq!(calculate_basic_use_bars(50.0, 15), 8);
        assert_eq!(calculate_basic_use_bars(100.0, 15), 15);
        assert_eq!(calculate_basic_use_bars(150.0, 15), 15);
    }
}
