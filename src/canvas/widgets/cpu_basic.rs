use std::cmp::min;

use itertools::Itertools;
use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    app::App,
    canvas::{
        Painter,
        components::pipe_gauge::{LabelLimit, PipeGauge},
        drawing_utils::widget_block,
    },
    collection::cpu::{CpuData, CpuDataType},
};

impl Painter {
    /// Inspired by htop.
    pub fn draw_basic_cpu(
        &self, f: &mut Frame<'_>, app_state: &mut App, mut draw_loc: Rect, widget_id: u64,
    ) {
        let cpu_data = &app_state.data_store.get_data().cpu_harvest;

        // This is a bit complicated, but basically, we want to draw SOME number
        // of columns to draw all CPUs. Ideally, as well, we want to not have
        // to ever scroll.
        //
        // **General logic** - count number of elements in cpu_data.  Then see how
        // many rows and columns we have in draw_loc (-2 on both sides for border?).
        // I think what we can do is try to fit in as many in one column as possible.
        // If not, then add a new column. Then, from this, split the row space across ALL columns.
        // From there, generate the desired lengths.

        if app_state.current_widget.widget_id == widget_id {
            f.render_widget(
                widget_block(true, true, self.styles.border_type)
                    .border_style(self.styles.highlighted_border_style),
                draw_loc,
            );
        }

        // TODO: This is pretty ugly. Is there a better way of doing it?
        let mut avg_index = cpu_data.len() + 1;
        let mut avg_row_count = 0;
        if app_state.app_config_fields.dedicated_average_row
            && app_state.app_config_fields.show_average_cpu
        {
            if let Some((index, avg)) = cpu_data
                .iter()
                .find_position(|&datum| matches!(datum.data_type, CpuDataType::Avg))
            {
                let (outer, inner, ratio, style) = self.cpu_info(avg);
                let [cores_loc, mut avg_loc] =
                    Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(draw_loc);

                // The cores section all have horizontal margin, so to line up with the cores we
                // need to add some margin ourselves.
                avg_loc.x += 1;
                avg_loc.width -= 2;

                f.render_widget(
                    PipeGauge::default()
                        .gauge_style(style)
                        .label_style(style)
                        .inner_label(inner)
                        .start_label(outer)
                        .ratio(ratio.into()),
                    avg_loc,
                );
                avg_row_count += 1;
                avg_index = index;
                draw_loc = cores_loc;
            }
        }

        if draw_loc.height > 0 {
            let remaining_height = usize::from(draw_loc.height);
            const REQUIRED_COLUMNS: usize = 4;

            let col_constraints =
                vec![Constraint::Percentage((100 / REQUIRED_COLUMNS) as u16); REQUIRED_COLUMNS];
            let columns = Layout::default()
                .constraints(col_constraints)
                .direction(Direction::Horizontal)
                .split(draw_loc);

            let mut gauge_info = cpu_data.iter().enumerate().filter_map(|(index, cpu)| {
                if index == avg_index {
                    None
                } else {
                    Some(self.cpu_info(cpu))
                }
            });

            // Very ugly way to sync the gauge limit across all gauges.
            let hide_parts = columns
                .first()
                .map(|col| {
                    if col.width >= 12 {
                        LabelLimit::None
                    } else if col.width >= 10 {
                        LabelLimit::Bars
                    } else {
                        LabelLimit::StartLabel
                    }
                })
                .unwrap_or_default();

            let num_entries = cpu_data.len() - avg_row_count;
            let mut row_counter = num_entries;
            for (itx, column) in columns.iter().enumerate() {
                if REQUIRED_COLUMNS > itx {
                    let to_divide = REQUIRED_COLUMNS - itx;
                    let num_taken = min(
                        remaining_height,
                        (row_counter / to_divide) + usize::from(row_counter % to_divide != 0),
                    );
                    row_counter -= num_taken;
                    let chunk = (&mut gauge_info).take(num_taken);

                    let rows = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(vec![Constraint::Length(1); remaining_height])
                        .horizontal_margin(1)
                        .split(*column);

                    for ((start_label, inner_label, ratio, style), row) in chunk.zip(rows.iter()) {
                        f.render_widget(
                            PipeGauge::default()
                                .gauge_style(style)
                                .label_style(style)
                                .inner_label(inner_label)
                                .start_label(start_label)
                                .ratio(ratio.into())
                                .hide_parts(hide_parts),
                            *row,
                        );
                    }
                }
            }
        }

        if app_state.should_get_widget_bounds() {
            // Update draw loc in widget map
            if let Some(widget) = app_state.widget_map.get_mut(&widget_id) {
                widget.top_left_corner = Some((draw_loc.x, draw_loc.y));
                widget.bottom_right_corner =
                    Some((draw_loc.x + draw_loc.width, draw_loc.y + draw_loc.height));
            }
        }
    }

    #[inline]
    fn cpu_info(&self, data: &CpuData) -> (String, String, f32, tui::style::Style) {
        let (outer, style) = match data.data_type {
            CpuDataType::Avg => ("AVG".to_string(), self.styles.avg_cpu_colour),
            CpuDataType::Cpu(index) => (
                format!("{index:<3}",),
                self.styles.cpu_colour_styles
                    [(index as usize) % self.styles.cpu_colour_styles.len()],
            ),
        };

        let inner = format!("{:>3.0}%", data.usage.round());
        let ratio = data.usage / 100.0;

        (outer, inner, ratio, style)
    }
}
