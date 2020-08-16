use crate::{
    app::{
        layout_manager::{BottomWidget, BottomWidgetType},
        App,
    },
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
        &self, f: &mut Frame<'_, B>, app_state: &App, draw_loc: Rect, current_table: &BottomWidget,
    );
}

impl BasicTableArrows for Painter {
    fn draw_basic_table_arrows<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &App, draw_loc: Rect, current_table: &BottomWidget,
    ) {
        let current_table = if let BottomWidgetType::ProcSort = current_table.widget_type {
            current_table
                .right_neighbour
                .map(|id| app_state.widget_map.get(&id).unwrap())
                .unwrap()
        } else {
            current_table
        };

        // Effectively a paragraph with a ton of spacing
        let (left_table, right_table) = (
            {
                current_table
                    .left_neighbour
                    .map(|left_widget_id| {
                        app_state
                            .widget_map
                            .get(&left_widget_id)
                            .map(|left_widget| {
                                if left_widget.widget_type == BottomWidgetType::ProcSort {
                                    left_widget
                                        .left_neighbour
                                        .map(|left_left_widget_id| {
                                            app_state.widget_map.get(&left_left_widget_id).map(
                                                |left_left_widget| &left_left_widget.widget_type,
                                            )
                                        })
                                        .unwrap_or_else(|| Some(&BottomWidgetType::Temp))
                                        .unwrap_or_else(|| &BottomWidgetType::Temp)
                                } else {
                                    &left_widget.widget_type
                                }
                            })
                            .unwrap_or_else(|| &BottomWidgetType::Temp)
                    })
                    .unwrap_or_else(|| &BottomWidgetType::Temp)
            },
            {
                current_table
                    .right_neighbour
                    .map(|right_widget_id| {
                        app_state
                            .widget_map
                            .get(&right_widget_id)
                            .map(|right_widget| {
                                if right_widget.widget_type == BottomWidgetType::ProcSort {
                                    right_widget
                                        .right_neighbour
                                        .map(|right_right_widget_id| {
                                            app_state.widget_map.get(&right_right_widget_id).map(
                                                |right_right_widget| {
                                                    &right_right_widget.widget_type
                                                },
                                            )
                                        })
                                        .unwrap_or_else(|| Some(&BottomWidgetType::Disk))
                                        .unwrap_or_else(|| &BottomWidgetType::Disk)
                                } else {
                                    &right_widget.widget_type
                                }
                            })
                            .unwrap_or_else(|| &BottomWidgetType::Disk)
                    })
                    .unwrap_or_else(|| &BottomWidgetType::Disk)
            },
        );

        let left_name = left_table.get_pretty_name();
        let right_name = right_table.get_pretty_name();

        let num_spaces =
            usize::from(draw_loc.width).saturating_sub(6 + left_name.len() + right_name.len());

        let arrow_text = vec![
            Text::raw("\n"),
            Text::styled(format!("◄ {}", left_name), self.colours.text_style),
            Text::raw(" ".repeat(num_spaces)),
            Text::styled(format!("{} ►", right_name), self.colours.text_style),
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
