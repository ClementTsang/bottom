use std::cmp::max;

use crate::{
    app::{App, WidgetPosition},
    canvas::Painter,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Paragraph, Text, Widget},
};

pub trait BasicTableArrows {
    fn draw_basic_table_arrows<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    );
}

impl BasicTableArrows for Painter {
    fn draw_basic_table_arrows<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        // Effectively a paragraph with a ton of spacing

        // TODO: [MODULARITY] This is hard coded.  Gross.
        let (left_table, right_table) = if app_state.current_widget_selected.is_widget_table() {
            match app_state.current_widget_selected {
                WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                    (WidgetPosition::Temp, WidgetPosition::Disk)
                }
                WidgetPosition::Disk => (WidgetPosition::Process, WidgetPosition::Temp),
                WidgetPosition::Temp => (WidgetPosition::Disk, WidgetPosition::Process),
                _ => (WidgetPosition::Disk, WidgetPosition::Temp),
            }
        } else {
            match app_state.previous_basic_table_selected {
                WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                    (WidgetPosition::Temp, WidgetPosition::Disk)
                }
                WidgetPosition::Disk => (WidgetPosition::Process, WidgetPosition::Temp),
                WidgetPosition::Temp => (WidgetPosition::Disk, WidgetPosition::Process),
                _ => (WidgetPosition::Disk, WidgetPosition::Temp),
            }
        };

        let left_name = left_table.get_pretty_name();
        let right_name = right_table.get_pretty_name();

        let num_spaces = max(
            0,
            draw_loc.width as i64 - 2 - 4 - (left_name.len() + right_name.len()) as i64,
        ) as usize;

        let arrow_text = vec![
            Text::Styled(
                format!("\n◄ {}", right_name).into(),
                self.colours.text_style,
            ),
            Text::Raw(" ".repeat(num_spaces).into()),
            Text::Styled(format!("{} ►", left_name).into(), self.colours.text_style),
        ];

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(draw_loc);

        Paragraph::new(arrow_text.iter())
            .block(Block::default())
            .render(f, margined_draw_loc[0]);
    }
}
