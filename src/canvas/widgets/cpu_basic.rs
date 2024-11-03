use std::cmp::min;

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Block,
    Frame,
};

use crate::{
    app::App,
    canvas::{
        components::pipe_gauge::{LabelLimit, PipeGauge},
        Painter,
    },
    constants::*,
    data_collection::cpu::CpuDataType,
    data_conversion::CpuWidgetData,
};

impl Painter {
    /// Inspired by htop.
    pub fn draw_basic_cpu(
        &self, f: &mut Frame<'_>, app_state: &mut App, mut draw_loc: Rect, widget_id: u64,
    ) {
        // Skip the first element, it's the "all" element
        if app_state.converted_data.cpu_data.len() > 1 {
            let cpu_data: &[CpuWidgetData] = &app_state.converted_data.cpu_data[1..];

            // This is a bit complicated, but basically, we want to draw SOME number
            // of columns to draw all CPUs.  Ideally, as well, we want to not have
            // to ever scroll.
            // **General logic** - count number of elements in cpu_data.  Then see how
            // many rows and columns we have in draw_loc (-2 on both sides for border?).
            // I think what we can do is try to fit in as many in one column as possible.
            // If not, then add a new column.
            // Then, from this, split the row space across ALL columns.  From there,
            // generate the desired lengths.

            if app_state.current_widget.widget_id == widget_id {
                f.render_widget(
                    Block::default()
                        .borders(SIDE_BORDERS)
                        .border_style(self.colours.highlighted_border_style),
                    draw_loc,
                );
            }

            let (cpu_data, avg_data) =
                maybe_split_avg(cpu_data, app_state.app_config_fields.dedicated_average_row);

            if let Some(avg) = avg_data {
                let (outer, inner, ratio, style) = self.cpu_info(&avg);
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
                        .ratio(ratio),
                    avg_loc,
                );

                draw_loc = cores_loc;
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

                let mut gauge_info = cpu_data.iter().map(|cpu| self.cpu_info(cpu));

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

                let num_entries = cpu_data.len();
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

                        for ((start_label, inner_label, ratio, style), row) in
                            chunk.zip(rows.iter())
                        {
                            f.render_widget(
                                PipeGauge::default()
                                    .gauge_style(style)
                                    .label_style(style)
                                    .inner_label(inner_label)
                                    .start_label(start_label)
                                    .ratio(ratio)
                                    .hide_parts(hide_parts),
                                *row,
                            );
                        }
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

    fn cpu_info(&self, cpu: &CpuWidgetData) -> (String, String, f64, tui::style::Style) {
        let CpuWidgetData::Entry {
            data_type,
            last_entry,
            ..
        } = cpu
        else {
            unreachable!()
        };

        let (outer, style) = match data_type {
            CpuDataType::Avg => ("AVG".to_string(), self.colours.avg_cpu_colour),
            CpuDataType::Cpu(index) => (
                format!("{index:<3}",),
                self.colours.cpu_colour_styles[index % self.colours.cpu_colour_styles.len()],
            ),
        };
        let inner = format!("{:>3.0}%", last_entry.round());
        let ratio = last_entry / 100.0;

        (outer, inner, ratio, style)
    }
}

fn maybe_split_avg(
    data: &[CpuWidgetData], separate_avg: bool,
) -> (Vec<CpuWidgetData>, Option<CpuWidgetData>) {
    let mut cpu_data = vec![];
    let mut avg_data = None;

    for cpu in data {
        let CpuWidgetData::Entry {
            data_type,
            data,
            last_entry,
        } = cpu
        else {
            unreachable!()
        };

        match data_type {
            CpuDataType::Avg if separate_avg => {
                avg_data = Some(CpuWidgetData::Entry {
                    data_type: *data_type,
                    data: data.clone(),
                    last_entry: *last_entry,
                });
            }
            _ => {
                cpu_data.push(CpuWidgetData::Entry {
                    data_type: *data_type,
                    data: data.clone(),
                    last_entry: *last_entry,
                });
            }
        }
    }

    (cpu_data, avg_data)
}
