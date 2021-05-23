use crate::{
    app::{layout_manager::BottomWidgetType, AppState},
    canvas::Painter,
};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::Span,
    text::Spans,
    widgets::{Block, Paragraph},
};

pub trait BasicTableArrows {
    fn draw_basic_table_arrows<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    );
}

impl BasicTableArrows for Painter {
    fn draw_basic_table_arrows<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut AppState, draw_loc: Rect, widget_id: u64,
    ) {
        if let Some(current_table) = app_state.widget_map.get(&widget_id) {
            let current_table = if let BottomWidgetType::ProcSort = current_table.widget_type {
                current_table
                    .right_neighbour
                    .map(|id| app_state.widget_map.get(&id).unwrap())
                    .unwrap()
            } else {
                current_table
            };

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
                                                    |left_left_widget| {
                                                        &left_left_widget.widget_type
                                                    },
                                                )
                                            })
                                            .unwrap_or(Some(&BottomWidgetType::Temp))
                                            .unwrap_or(&BottomWidgetType::Temp)
                                    } else {
                                        &left_widget.widget_type
                                    }
                                })
                                .unwrap_or(&BottomWidgetType::Temp)
                        })
                        .unwrap_or(&BottomWidgetType::Temp)
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
                                                app_state
                                                    .widget_map
                                                    .get(&right_right_widget_id)
                                                    .map(|right_right_widget| {
                                                        &right_right_widget.widget_type
                                                    })
                                            })
                                            .unwrap_or(Some(&BottomWidgetType::Disk))
                                            .unwrap_or(&BottomWidgetType::Disk)
                                    } else {
                                        &right_widget.widget_type
                                    }
                                })
                                .unwrap_or(&BottomWidgetType::Disk)
                        })
                        .unwrap_or(&BottomWidgetType::Disk)
                },
            );

            let left_name = left_table.get_pretty_name();
            let right_name = right_table.get_pretty_name();

            let num_spaces =
                usize::from(draw_loc.width).saturating_sub(6 + left_name.len() + right_name.len());

            let left_arrow_text = vec![
                Spans::default(),
                Spans::from(Span::styled(
                    format!("◄ {}", left_name),
                    self.colours.text_style,
                )),
            ];

            let right_arrow_text = vec![
                Spans::default(),
                Spans::from(Span::styled(
                    format!("{} ►", right_name),
                    self.colours.text_style,
                )),
            ];

            let margined_draw_loc = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(2 + left_name.len() as u16),
                    Constraint::Length(num_spaces as u16),
                    Constraint::Length(2 + right_name.len() as u16),
                ])
                .horizontal_margin(1)
                .split(draw_loc);

            f.render_widget(
                Paragraph::new(left_arrow_text).block(Block::default()),
                margined_draw_loc[0],
            );
            f.render_widget(
                Paragraph::new(right_arrow_text)
                    .block(Block::default())
                    .alignment(Alignment::Right),
                margined_draw_loc[2],
            );

            if app_state.should_get_widget_bounds() {
                // Some explanations for future readers:
                // - The "height" as of writing of this entire widget is 2.  If it's 1, it occasionally doesn't draw.
                // - As such, the buttons are only on the lower part of this 2-high widget.
                // - So, we want to only check at one location, the `draw_loc.y + 1`, and that's it.
                // - But why is it "+2" then?  Well, it's because I have a REALLY ugly hack
                //   for mouse button checking, since most button checks are of the form `(draw_loc.y + draw_loc.height)`,
                //   and the same for the x and width.  Unfortunately, if you check using >= and <=, the outer bound is
                //   actually too large - so, we assume all of them are one too big and check via < (see
                //   https://github.com/ClementTsang/bottom/pull/459 for details).
                // - So in other words, to make it simple, we keep this to a standard and overshoot by one here.
                if let Some(basic_table) = &mut app_state.basic_table_widget_state {
                    basic_table.left_tlc =
                        Some((margined_draw_loc[0].x, margined_draw_loc[0].y + 1));
                    basic_table.left_brc = Some((
                        margined_draw_loc[0].x + margined_draw_loc[0].width,
                        margined_draw_loc[0].y + 2,
                    ));
                    basic_table.right_tlc =
                        Some((margined_draw_loc[2].x, margined_draw_loc[2].y + 1));
                    basic_table.right_brc = Some((
                        margined_draw_loc[2].x + margined_draw_loc[2].width,
                        margined_draw_loc[2].y + 2,
                    ));
                }
            }
        }
    }
}
