use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    app::App,
    canvas::{drawing_utils::calculate_basic_use_bars, Painter},
    constants::*,
    data_conversion::BatteryDuration,
};

impl Painter {
    pub fn draw_battery(
        &self, f: &mut Frame<'_>, app_state: &mut App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        let should_get_widget_bounds = app_state.should_get_widget_bounds();
        if let Some(battery_widget_state) = app_state
            .states
            .battery_state
            .widget_states
            .get_mut(&widget_id)
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
                Line::from(vec![
                    Span::styled(" Battery ", self.colours.widget_title_style),
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
                Line::from(Span::styled(" Battery ", self.colours.widget_title_style))
            };

            let battery_block = if draw_border {
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            } else if is_on_widget {
                Block::default()
                    .borders(SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            if app_state.converted_data.battery_data.len() > 1 {
                let battery_names = app_state
                    .converted_data
                    .battery_data
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
                    .style(self.colours.text_style)
                    .highlight_style(self.colours.selected_text_style)
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

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .horizontal_margin(u16::from(!(is_on_widget || draw_border)))
                .direction(Direction::Horizontal)
                .split(draw_loc)[0];

            if let Some(battery_details) = app_state
                .converted_data
                .battery_data
                .get(battery_widget_state.currently_selected_battery_index)
            {
                let full_width = draw_loc.width.saturating_sub(2);
                let bar_length = usize::from(full_width.saturating_sub(6));
                let charge_percentage = battery_details.charge_percentage;
                let num_bars = calculate_basic_use_bars(charge_percentage, bar_length);
                let bars = format!(
                    "[{}{}{:3.0}%]",
                    "|".repeat(num_bars),
                    " ".repeat(bar_length - num_bars),
                    charge_percentage,
                );

                let mut battery_charge_rows = Vec::with_capacity(2);
                battery_charge_rows.push(Row::new([
                    Cell::from("Charge").style(self.colours.text_style)
                ]));
                battery_charge_rows.push(Row::new([Cell::from(bars).style(
                    if charge_percentage < 10.0 {
                        self.colours.low_battery
                    } else if charge_percentage < 50.0 {
                        self.colours.medium_battery
                    } else {
                        self.colours.high_battery
                    },
                )]));

                let mut battery_rows = Vec::with_capacity(3);
                battery_rows.push(Row::new([""]).bottom_margin(table_gap + 1));
                battery_rows.push(
                    Row::new(["Rate", &battery_details.watt_consumption])
                        .style(self.colours.text_style),
                );

                battery_rows.push(
                    Row::new(["State", &battery_details.state]).style(self.colours.text_style),
                );

                let mut time: String; // Keep string lifetime in scope.
                {
                    let style = self.colours.text_style;
                    match &battery_details.battery_duration {
                        BatteryDuration::ToEmpty(secs) => {
                            time = long_time(*secs);

                            if full_width as usize > time.len() {
                                battery_rows.push(Row::new(["Time to empty", &time]).style(style));
                            } else {
                                time = short_time(*secs);
                                battery_rows.push(Row::new(["To empty", &time]).style(style));
                            }
                        }
                        BatteryDuration::ToFull(secs) => {
                            time = long_time(*secs);

                            if full_width as usize > time.len() {
                                battery_rows.push(Row::new(["Time to full", &time]).style(style));
                            } else {
                                time = short_time(*secs);
                                battery_rows.push(Row::new(["To full", &time]).style(style));
                            }
                        }
                        BatteryDuration::Empty
                        | BatteryDuration::Full
                        | BatteryDuration::Unknown => {}
                    }
                }

                battery_rows.push(
                    Row::new(["Health", &battery_details.health]).style(self.colours.text_style),
                );

                let header = if app_state.converted_data.battery_data.len() > 1 {
                    Row::new([""]).bottom_margin(table_gap)
                } else {
                    Row::default()
                };

                // Draw bar
                f.render_widget(
                    Table::new(battery_charge_rows, [Constraint::Percentage(100)])
                        .block(battery_block.clone())
                        .header(header.clone()),
                    margined_draw_loc,
                );

                // Draw info
                f.render_widget(
                    Table::new(
                        battery_rows,
                        [Constraint::Percentage(50), Constraint::Percentage(50)],
                    )
                    .block(battery_block)
                    .header(header),
                    margined_draw_loc,
                );
            } else {
                let mut contents = vec![Line::default(); table_gap.into()];

                contents.push(Line::from(Span::styled(
                    "No data found for this battery",
                    self.colours.text_style,
                )));

                f.render_widget(
                    Paragraph::new(contents).block(battery_block),
                    margined_draw_loc,
                );
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

#[inline]
fn get_hms(secs: i64) -> (i64, i64, i64) {
    let hours = secs / (60 * 60);
    let minutes = (secs / 60) - hours * 60;
    let seconds = secs - minutes * 60 - hours * 60 * 60;

    (hours, minutes, seconds)
}

fn long_time(secs: i64) -> String {
    let (hours, minutes, seconds) = get_hms(secs);

    if hours > 0 {
        format!(
            "{} hour{}, {} minute{}, {} second{}",
            hours,
            if hours == 1 { "" } else { "s" },
            minutes,
            if minutes == 1 { "" } else { "s" },
            seconds,
            if seconds == 1 { "" } else { "s" },
        )
    } else {
        format!(
            "{} minute{}, {} second{}",
            minutes,
            if minutes == 1 { "" } else { "s" },
            seconds,
            if seconds == 1 { "" } else { "s" },
        )
    }
}

fn short_time(secs: i64) -> String {
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
}
