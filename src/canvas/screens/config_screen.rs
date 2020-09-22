use crate::{app::App, canvas::Painter, constants};
use tui::{
    backend::Backend,
    layout::Constraint,
    layout::Direction,
    layout::Layout,
    layout::{Alignment, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph},
};

pub trait ConfigScreen {
    fn draw_config_screen<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    );
}

impl ConfigScreen for Painter {
    fn draw_config_screen<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect,
    ) {
        let config_block = Block::default()
            .title(&" Config ")
            .title_style(self.colours.border_style)
            .style(self.colours.border_style)
            .borders(Borders::ALL)
            .border_style(self.colours.border_style);

        f.render_widget(config_block, draw_loc);

        // let margined_draw_locs = Layout::default()
        //     .margin(2)
        //     .direction(Direction::Horizontal)
        //     .constraints(
        //         [
        //             Constraint::Percentage(33),
        //             Constraint::Percentage(34),
        //             Constraint::Percentage(33),
        //         ]
        //         .as_ref(),
        //     )
        //     .split(draw_loc)
        //     .into_iter()
        //     .map(|loc| {
        //         // Required to properly margin in *between* the rectangles.
        //         Layout::default()
        //             .horizontal_margin(1)
        //             .constraints([Constraint::Percentage(100)].as_ref())
        //             .split(loc)[0]
        //     })
        //     .collect::<Vec<Rect>>();

        // for dl in margined_draw_locs {
        //     f.render_widget(Block::default().borders(Borders::ALL), dl);
        // }
    }
}
