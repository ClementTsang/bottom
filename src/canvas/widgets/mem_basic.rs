use crate::{
    app::App, canvas::Painter, components::tui_widget::pipe_gauge::PipeGauge, constants::*,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::Block,
};

impl Painter {
    pub fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, draw_loc: Rect, widget_id: u64,
    ) {
        let mem_data: &[(f64, f64)] = &app_state.converted_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.converted_data.swap_data;

        let margined_loc = Layout::default()
            .constraints({
                #[cfg(feature = "zfs")]
                {
                    [Constraint::Length(1); 3]
                }

                #[cfg(not(feature = "zfs"))]
                {
                    [Constraint::Length(1); 2]
                }
            })
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .split(draw_loc);

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                Block::default()
                    .borders(SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                draw_loc,
            );
        }

        let ram_ratio = if let Some(mem) = mem_data.last() {
            mem.1 / 100.0
        } else {
            0.0
        };
        let swap_ratio = if let Some(swap) = swap_data.last() {
            swap.1 / 100.0
        } else {
            0.0
        };

        const EMPTY_MEMORY_FRAC_STRING: &str = "0.0B/0.0B";

        let memory_fraction_label =
            if let Some((_, label_frac)) = &app_state.converted_data.mem_labels {
                label_frac.trim()
            } else {
                EMPTY_MEMORY_FRAC_STRING
            };

        let swap_fraction_label =
            if let Some((_, label_frac)) = &app_state.converted_data.swap_labels {
                label_frac.trim()
            } else {
                EMPTY_MEMORY_FRAC_STRING
            };

        f.render_widget(
            PipeGauge::default()
                .ratio(ram_ratio)
                .start_label("RAM")
                .inner_label(memory_fraction_label)
                .label_style(self.colours.ram_style)
                .gauge_style(self.colours.ram_style),
            margined_loc[0],
        );

        f.render_widget(
            PipeGauge::default()
                .ratio(swap_ratio)
                .start_label("SWP")
                .inner_label(swap_fraction_label)
                .label_style(self.colours.swap_style)
                .gauge_style(self.colours.swap_style),
            margined_loc[1],
        );

        #[cfg(feature = "zfs")]
        {
            let arc_data: &[(f64, f64)] = &app_state.converted_data.arc_data;
            let arc_ratio = if let Some(arc) = arc_data.last() {
                arc.1 / 100.0
            } else {
                0.0
            };
            let arc_fraction_label =
                if let Some((_, label_frac)) = &app_state.converted_data.arc_labels {
                    label_frac.trim()
                } else {
                    EMPTY_MEMORY_FRAC_STRING
                };

            f.render_widget(
                PipeGauge::default()
                    .ratio(arc_ratio)
                    .start_label("ARC")
                    .inner_label(arc_fraction_label)
                    .label_style(self.colours.arc_style)
                    .gauge_style(self.colours.arc_style),
                margined_loc[2],
            );
        }

        // Update draw loc in widget map
        if app_state.should_get_widget_bounds() {
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }
}
