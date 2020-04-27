use std::cmp::max;

use crate::{
    app::{layout_manager::BottomWidgetType, App},
    canvas::Painter,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Paragraph, Text},
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
        let (left_table, right_table) = (
            {
                app_state
                    .current_widget
                    .left_neighbour
                    .and_then(|left_widget_id| {
                        Some(
                            app_state
                                .widget_map
                                .get(&left_widget_id)
                                .and_then(|left_widget| Some(&left_widget.widget_type))
                                .unwrap_or_else(|| &BottomWidgetType::Temp),
                        )
                    })
                    .unwrap_or_else(|| &BottomWidgetType::Temp)
            },
            {
                app_state
                    .current_widget
                    .right_neighbour
                    .and_then(|right_widget_id| {
                        Some(
                            app_state
                                .widget_map
                                .get(&right_widget_id)
                                .and_then(|right_widget| Some(&right_widget.widget_type))
                                .unwrap_or_else(|| &BottomWidgetType::Disk),
                        )
                    })
                    .unwrap_or_else(|| &BottomWidgetType::Disk)
            },
        );

        let left_name = left_table.get_pretty_name();
        let right_name = right_table.get_pretty_name();

        let num_spaces = max(
            0,
            draw_loc.width as i64 - 2 - 4 - (left_name.len() + right_name.len()) as i64,
        ) as usize;

        let arrow_text = vec![
            Text::Styled(format!("\n◄ {}", left_name).into(), self.colours.text_style),
            Text::Raw(" ".repeat(num_spaces).into()),
            Text::Styled(format!("{} ►", right_name).into(), self.colours.text_style),
        ];

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(draw_loc);

        f.render_widget(
            Paragraph::new(arrow_text.iter()).block(Block::default()),
            margined_draw_loc[0],
        );
    }
}
