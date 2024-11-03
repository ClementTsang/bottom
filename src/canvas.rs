pub mod components;
mod dialogs;
mod drawing_utils;
mod widgets;

use itertools::izip;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    widgets::Paragraph,
    Frame, Terminal,
};

use crate::{
    app::{
        layout_manager::{BottomColRow, BottomLayout, BottomWidgetType, IntermediaryConstraint},
        App,
    },
    constants::*,
    options::config::style::ColourPalette,
};

/// Handles the canvas' state.
pub struct Painter {
    pub colours: ColourPalette,
    previous_height: u16,
    previous_width: u16,

    // TODO: Redo this entire thing.
    row_constraints: Vec<LayoutConstraint>,
    col_constraints: Vec<Vec<LayoutConstraint>>,
    col_row_constraints: Vec<Vec<Vec<LayoutConstraint>>>,
    layout_constraints: Vec<Vec<Vec<Vec<LayoutConstraint>>>>,
    derived_widget_draw_locs: Vec<Vec<Vec<Vec<Rect>>>>,
    widget_layout: BottomLayout,
}

/// The constraints of a widget relative to its parent.
///
/// This is used over ratatui's internal representation due to
/// <https://github.com/ClementTsang/bottom/issues/896>.
pub enum LayoutConstraint {
    CanvasHandled,
    Grow,
    Ratio(u32, u32),
}

impl Painter {
    pub fn init(layout: BottomLayout, styling: ColourPalette) -> anyhow::Result<Self> {
        // Now for modularity; we have to also initialize the base layouts!
        // We want to do this ONCE and reuse; after this we can just construct
        // based on the console size.

        let mut row_constraints = Vec::new();
        let mut col_constraints = Vec::new();
        let mut col_row_constraints = Vec::new();
        let mut layout_constraints = Vec::new();

        layout.rows.iter().for_each(|row| {
            match row.constraint {
                IntermediaryConstraint::PartialRatio(val) => {
                    row_constraints
                        .push(LayoutConstraint::Ratio(val, layout.total_row_height_ratio));
                }
                IntermediaryConstraint::CanvasHandled { .. } => {
                    row_constraints.push(LayoutConstraint::CanvasHandled);
                }
                IntermediaryConstraint::Grow { .. } => {
                    row_constraints.push(LayoutConstraint::Grow);
                }
            }

            let mut new_col_constraints = Vec::new();
            let mut new_widget_constraints = Vec::new();
            let mut new_col_row_constraints = Vec::new();
            row.children.iter().for_each(|col| {
                match col.constraint {
                    IntermediaryConstraint::PartialRatio(val) => {
                        new_col_constraints.push(LayoutConstraint::Ratio(val, row.total_col_ratio));
                    }
                    IntermediaryConstraint::CanvasHandled { .. } => {
                        new_col_constraints.push(LayoutConstraint::CanvasHandled);
                    }
                    IntermediaryConstraint::Grow { .. } => {
                        new_col_constraints.push(LayoutConstraint::Grow);
                    }
                }

                let mut new_new_col_row_constraints = Vec::new();
                let mut new_new_widget_constraints = Vec::new();
                col.children.iter().for_each(|col_row| {
                    match col_row.constraint {
                        IntermediaryConstraint::PartialRatio(val) => {
                            new_new_col_row_constraints
                                .push(LayoutConstraint::Ratio(val, col.total_col_row_ratio));
                        }
                        IntermediaryConstraint::CanvasHandled { .. } => {
                            new_new_col_row_constraints.push(LayoutConstraint::CanvasHandled);
                        }
                        IntermediaryConstraint::Grow { .. } => {
                            new_new_col_row_constraints.push(LayoutConstraint::Grow);
                        }
                    }

                    let mut new_new_new_widget_constraints = Vec::new();
                    col_row
                        .children
                        .iter()
                        .for_each(|widget| match widget.constraint {
                            IntermediaryConstraint::PartialRatio(val) => {
                                new_new_new_widget_constraints
                                    .push(LayoutConstraint::Ratio(val, col_row.total_widget_ratio));
                            }
                            IntermediaryConstraint::CanvasHandled { .. } => {
                                new_new_new_widget_constraints
                                    .push(LayoutConstraint::CanvasHandled);
                            }
                            IntermediaryConstraint::Grow { .. } => {
                                new_new_new_widget_constraints.push(LayoutConstraint::Grow);
                            }
                        });
                    new_new_widget_constraints.push(new_new_new_widget_constraints);
                });
                new_col_row_constraints.push(new_new_col_row_constraints);
                new_widget_constraints.push(new_new_widget_constraints);
            });
            col_row_constraints.push(new_col_row_constraints);
            layout_constraints.push(new_widget_constraints);
            col_constraints.push(new_col_constraints);
        });

        let painter = Painter {
            colours: styling,
            previous_height: 0,
            previous_width: 0,
            row_constraints,
            col_constraints,
            col_row_constraints,
            layout_constraints,
            widget_layout: layout,
            derived_widget_draw_locs: Vec::default(),
        };

        Ok(painter)
    }

    /// Determines the border style.
    pub fn get_border_style(&self, widget_id: u64, selected_widget_id: u64) -> tui::style::Style {
        let is_on_widget = widget_id == selected_widget_id;
        if is_on_widget {
            self.colours.highlighted_border_style
        } else {
            self.colours.border_style
        }
    }

    fn draw_frozen_indicator(&self, f: &mut Frame<'_>, draw_loc: Rect) {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Frozen, press 'f' to unfreeze",
                self.colours.selected_text_style,
            )),
            Layout::default()
                .horizontal_margin(1)
                .constraints([Constraint::Length(1)])
                .split(draw_loc)[0],
        )
    }

    pub fn draw_data<B: Backend>(
        &mut self, terminal: &mut Terminal<B>, app_state: &mut App,
    ) -> Result<(), std::io::Error> {
        use BottomWidgetType::*;

        terminal.draw(|f| {
            let (terminal_size, frozen_draw_loc) = if app_state.frozen_state.is_frozen() {
                // TODO: Remove built-in cache?
                let split_loc = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(f.area());
                (split_loc[0], Some(split_loc[1]))
            } else {
                (f.area(), None)
            };
            let terminal_height = terminal_size.height;
            let terminal_width = terminal_size.width;

            if (self.previous_height == 0 && self.previous_width == 0)
                || (self.previous_height != terminal_height
                    || self.previous_width != terminal_width)
            {
                app_state.is_force_redraw = true;
                self.previous_height = terminal_height;
                self.previous_width = terminal_width;
            }

            if app_state.should_get_widget_bounds() {
                // If we're force drawing, reset ALL mouse boundaries.
                for widget in app_state.widget_map.values_mut() {
                    widget.top_left_corner = None;
                    widget.bottom_right_corner = None;
                }

                // Reset dd_dialog...
                app_state.delete_dialog_state.button_positions = vec![];

                // Reset battery dialog...
                for battery_widget in app_state.states.battery_state.widget_states.values_mut() {
                    battery_widget.tab_click_locs = None;
                }
            }

            if app_state.help_dialog_state.is_showing_help {
                let gen_help_len = GENERAL_HELP_TEXT.len() as u16 + 3;
                let border_len = terminal_height.saturating_sub(gen_help_len) / 2;
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(border_len),
                        Constraint::Length(gen_help_len),
                        Constraint::Length(border_len),
                    ])
                    .split(terminal_size);

                let middle_dialog_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(if terminal_width < 100 {
                        // TODO: [REFACTOR] The point we start changing size at currently hard-coded
                        // in.
                        [
                            Constraint::Percentage(0),
                            Constraint::Percentage(100),
                            Constraint::Percentage(0),
                        ]
                    } else {
                        [
                            Constraint::Percentage(15),
                            Constraint::Percentage(70),
                            Constraint::Percentage(15),
                        ]
                    })
                    .split(vertical_dialog_chunk[1]);

                self.draw_help_dialog(f, app_state, middle_dialog_chunk[1]);
            } else if app_state.delete_dialog_state.is_showing_dd {
                let dd_text = self.get_dd_spans(app_state);

                let text_width = if terminal_width < 100 {
                    terminal_width * 90 / 100
                } else {
                    terminal_width * 50 / 100
                };

                let text_height = if cfg!(target_os = "windows")
                    || !app_state.app_config_fields.is_advanced_kill
                {
                    7
                } else {
                    22
                };

                let vertical_bordering = terminal_height.saturating_sub(text_height) / 2;
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(vertical_bordering),
                        Constraint::Length(text_height),
                        Constraint::Length(vertical_bordering),
                    ])
                    .split(terminal_size);

                let horizontal_bordering = terminal_width.saturating_sub(text_width) / 2;
                let middle_dialog_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(horizontal_bordering),
                        Constraint::Length(text_width),
                        Constraint::Length(horizontal_bordering),
                    ])
                    .split(vertical_dialog_chunk[1]);

                // This is a bit nasty, but it works well... I guess.
                app_state.delete_dialog_state.is_showing_dd =
                    self.draw_dd_dialog(f, dd_text, app_state, middle_dialog_chunk[1]);
            } else if app_state.is_expanded {
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(f, frozen_draw_loc);
                }

                let rect = Layout::default()
                    .margin(0)
                    .constraints([Constraint::Percentage(100)])
                    .split(terminal_size);
                match &app_state.current_widget.widget_type {
                    Cpu => self.draw_cpu(f, app_state, rect[0], app_state.current_widget.widget_id),
                    CpuLegend => self.draw_cpu(
                        f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id - 1,
                    ),
                    Mem | BasicMem => self.draw_memory_graph(
                        f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                    ),
                    Disk => self.draw_disk_table(
                        f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                    ),
                    Temp => self.draw_temp_table(
                        f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                    ),
                    Net => self.draw_network_graph(
                        f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                        false,
                    ),
                    Proc | ProcSearch | ProcSort => {
                        let widget_id = app_state.current_widget.widget_id
                            - match &app_state.current_widget.widget_type {
                                ProcSearch => 1,
                                ProcSort => 2,
                                _ => 0,
                            };

                        self.draw_process(f, app_state, rect[0], true, widget_id);
                    }
                    Battery => self.draw_battery(
                        f,
                        app_state,
                        rect[0],
                        true,
                        app_state.current_widget.widget_id,
                    ),
                    _ => {}
                }
            } else if app_state.app_config_fields.use_basic_mode {
                // Basic mode.  This basically removes all graphs but otherwise
                // the same info.
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(f, frozen_draw_loc);
                }

                let actual_cpu_data_len = app_state.converted_data.cpu_data.len().saturating_sub(1);

                // This fixes #397, apparently if the height is 1, it can't render the CPU
                // bars...
                let cpu_height = {
                    let c = (actual_cpu_data_len / 4) as u16
                        + u16::from(actual_cpu_data_len % 4 != 0)
                        + u16::from(
                            app_state.app_config_fields.dedicated_average_row
                                && actual_cpu_data_len.saturating_sub(1) % 4 != 0,
                        );

                    if c <= 1 {
                        1
                    } else {
                        c
                    }
                };

                let mut mem_rows = 1;

                if app_state.converted_data.swap_labels.is_some() {
                    mem_rows += 1; // add row for swap
                }

                #[cfg(feature = "zfs")]
                {
                    if app_state.converted_data.arc_labels.is_some() {
                        mem_rows += 1; // add row for arc
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    if app_state.converted_data.cache_labels.is_some() {
                        mem_rows += 1;
                    }
                }

                #[cfg(feature = "gpu")]
                {
                    if let Some(gpu_data) = &app_state.converted_data.gpu_data {
                        mem_rows += gpu_data.len() as u16; // add row(s) for gpu
                    }
                }

                if mem_rows == 1 {
                    mem_rows += 1; // need at least 2 rows for RX and TX
                }

                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([
                        Constraint::Length(cpu_height),
                        Constraint::Length(mem_rows),
                        Constraint::Length(2),
                        Constraint::Min(5),
                    ])
                    .split(terminal_size);

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(vertical_chunks[1]);

                if vertical_chunks[0].width >= 2 {
                    self.draw_basic_cpu(f, app_state, vertical_chunks[0], 1);
                }
                if middle_chunks[0].width >= 2 {
                    self.draw_basic_memory(f, app_state, middle_chunks[0], 2);
                }
                if middle_chunks[1].width >= 2 {
                    self.draw_basic_network(f, app_state, middle_chunks[1], 3);
                }

                let mut later_widget_id: Option<u64> = None;
                if let Some(basic_table_widget_state) = &app_state.states.basic_table_widget_state {
                    let widget_id = basic_table_widget_state.currently_displayed_widget_id;
                    later_widget_id = Some(widget_id);
                    if vertical_chunks[3].width >= 2 {
                        match basic_table_widget_state.currently_displayed_widget_type {
                            Disk => {
                                self.draw_disk_table(f, app_state, vertical_chunks[3], widget_id)
                            }
                            Proc | ProcSort => {
                                let wid = widget_id
                                    - match basic_table_widget_state.currently_displayed_widget_type
                                    {
                                        ProcSearch => 1,
                                        ProcSort => 2,
                                        _ => 0,
                                    };
                                self.draw_process(f, app_state, vertical_chunks[3], false, wid);
                            }
                            Temp => {
                                self.draw_temp_table(f, app_state, vertical_chunks[3], widget_id)
                            }
                            Battery => self.draw_battery(
                                f,
                                app_state,
                                vertical_chunks[3],
                                false,
                                widget_id,
                            ),
                            _ => {}
                        }
                    }
                }

                if let Some(widget_id) = later_widget_id {
                    self.draw_basic_table_arrows(f, app_state, vertical_chunks[2], widget_id);
                }
            } else {
                // Draws using the passed in (or default) layout.
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(f, frozen_draw_loc);
                }

                if self.derived_widget_draw_locs.is_empty() || app_state.is_force_redraw {
                    // TODO: Can I remove this? Does ratatui's layout constraints work properly for
                    // fixing https://github.com/ClementTsang/bottom/issues/896 now?
                    fn get_constraints(
                        direction: Direction, constraints: &[LayoutConstraint], area: Rect,
                    ) -> Vec<Rect> {
                        // Order of operations:
                        // - Ratios first + canvas-handled (which is just zero)
                        // - Then any flex-grows to take up remaining space; divide amongst
                        //   remaining hand out any remaining space

                        #[derive(Debug, Default, Clone, Copy)]
                        struct Size {
                            width: u16,
                            height: u16,
                        }

                        impl Size {
                            fn shrink_width(&mut self, amount: u16) {
                                self.width -= amount;
                            }

                            fn shrink_height(&mut self, amount: u16) {
                                self.height -= amount;
                            }
                        }

                        let mut bounds = Size {
                            width: area.width,
                            height: area.height,
                        };
                        let mut sizes = vec![Size::default(); constraints.len()];
                        let mut grow = vec![];
                        let mut num_non_ch = 0;

                        for (itx, (constraint, size)) in
                            constraints.iter().zip(sizes.iter_mut()).enumerate()
                        {
                            match constraint {
                                LayoutConstraint::Ratio(a, b) => {
                                    match direction {
                                        Direction::Horizontal => {
                                            let amount =
                                                (((area.width as u32) * (*a)) / (*b)) as u16;
                                            bounds.shrink_width(amount);
                                            size.width = amount;
                                            size.height = area.height;
                                        }
                                        Direction::Vertical => {
                                            let amount =
                                                (((area.height as u32) * (*a)) / (*b)) as u16;
                                            bounds.shrink_height(amount);
                                            size.width = area.width;
                                            size.height = amount;
                                        }
                                    }
                                    num_non_ch += 1;
                                }
                                LayoutConstraint::Grow => {
                                    // Mark it as grow in the vector and handle in second pass.
                                    grow.push(itx);
                                    num_non_ch += 1;
                                }
                                LayoutConstraint::CanvasHandled => {
                                    // Do nothing in this case. It's already 0.
                                }
                            }
                        }

                        if !grow.is_empty() {
                            match direction {
                                Direction::Horizontal => {
                                    let width = bounds.width / grow.len() as u16;
                                    bounds.shrink_width(width * grow.len() as u16);
                                    for g in grow {
                                        sizes[g] = Size {
                                            width,
                                            height: area.height,
                                        };
                                    }
                                }
                                Direction::Vertical => {
                                    let height = bounds.height / grow.len() as u16;
                                    bounds.shrink_height(height * grow.len() as u16);
                                    for g in grow {
                                        sizes[g] = Size {
                                            width: area.width,
                                            height,
                                        };
                                    }
                                }
                            }
                        }

                        if num_non_ch > 0 {
                            match direction {
                                Direction::Horizontal => {
                                    let per_item = bounds.width / num_non_ch;
                                    let mut remaining_width = bounds.width % num_non_ch;
                                    for (size, constraint) in sizes.iter_mut().zip(constraints) {
                                        match constraint {
                                            LayoutConstraint::CanvasHandled => {}
                                            LayoutConstraint::Grow
                                            | LayoutConstraint::Ratio(_, _) => {
                                                if remaining_width > 0 {
                                                    size.width += per_item + 1;
                                                    remaining_width -= 1;
                                                } else {
                                                    size.width += per_item;
                                                }
                                            }
                                        }
                                    }
                                }
                                Direction::Vertical => {
                                    let per_item = bounds.height / num_non_ch;
                                    let mut remaining_height = bounds.height % num_non_ch;
                                    for (size, constraint) in sizes.iter_mut().zip(constraints) {
                                        match constraint {
                                            LayoutConstraint::CanvasHandled => {}
                                            LayoutConstraint::Grow
                                            | LayoutConstraint::Ratio(_, _) => {
                                                if remaining_height > 0 {
                                                    size.height += per_item + 1;
                                                    remaining_height -= 1;
                                                } else {
                                                    size.height += per_item;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let mut curr_x = area.x;
                        let mut curr_y = area.y;
                        sizes
                            .into_iter()
                            .map(|size| {
                                let rect = Rect::new(curr_x, curr_y, size.width, size.height);
                                match direction {
                                    Direction::Horizontal => {
                                        curr_x += size.width;
                                    }
                                    Direction::Vertical => {
                                        curr_y += size.height;
                                    }
                                }

                                rect
                            })
                            .collect()
                    }

                    let draw_locs =
                        get_constraints(Direction::Vertical, &self.row_constraints, terminal_size);

                    self.derived_widget_draw_locs = izip!(
                        draw_locs,
                        &self.col_constraints,
                        &self.col_row_constraints,
                        &self.layout_constraints,
                        &self.widget_layout.rows
                    )
                    .map(
                        |(
                            draw_loc,
                            col_constraint,
                            col_row_constraint,
                            row_constraint_vec,
                            cols,
                        )| {
                            izip!(
                                get_constraints(Direction::Horizontal, col_constraint, draw_loc),
                                col_row_constraint,
                                row_constraint_vec,
                                &cols.children
                            )
                            .map(|(split_loc, constraint, col_constraint_vec, col_rows)| {
                                izip!(
                                    get_constraints(
                                        Direction::Vertical,
                                        constraint.as_slice(),
                                        split_loc
                                    ),
                                    col_constraint_vec,
                                    &col_rows.children
                                )
                                .map(|(draw_loc, col_row_constraint_vec, widgets)| {
                                    // Note that col_row_constraint_vec CONTAINS the widget
                                    // constraints
                                    let widget_draw_locs = get_constraints(
                                        Direction::Horizontal,
                                        col_row_constraint_vec.as_slice(),
                                        draw_loc,
                                    );

                                    // Side effect, draw here.
                                    self.draw_widgets_with_constraints(
                                        f,
                                        app_state,
                                        widgets,
                                        &widget_draw_locs,
                                    );

                                    widget_draw_locs
                                })
                                .collect()
                            })
                            .collect()
                        },
                    )
                    .collect();
                } else {
                    self.widget_layout
                        .rows
                        .iter()
                        .flat_map(|row| &row.children)
                        .flat_map(|col| &col.children)
                        .zip(self.derived_widget_draw_locs.iter().flatten().flatten())
                        .for_each(|(widgets, widget_draw_locs)| {
                            self.draw_widgets_with_constraints(
                                f,
                                app_state,
                                widgets,
                                widget_draw_locs,
                            );
                        });
                }
            }
        })?;

        if let Some(updated_current_widget) = app_state
            .widget_map
            .get(&app_state.current_widget.widget_id)
        {
            app_state.current_widget = updated_current_widget.clone();
        }

        app_state.is_force_redraw = false;
        app_state.is_determining_widget_boundary = false;

        Ok(())
    }

    fn draw_widgets_with_constraints(
        &self, f: &mut Frame<'_>, app_state: &mut App, widgets: &BottomColRow,
        widget_draw_locs: &[Rect],
    ) {
        use BottomWidgetType::*;
        for (widget, draw_loc) in widgets.children.iter().zip(widget_draw_locs) {
            if draw_loc.width >= 2 && draw_loc.height >= 2 {
                match &widget.widget_type {
                    Cpu => self.draw_cpu(f, app_state, *draw_loc, widget.widget_id),
                    Mem => self.draw_memory_graph(f, app_state, *draw_loc, widget.widget_id),
                    Net => self.draw_network(f, app_state, *draw_loc, widget.widget_id),
                    Temp => self.draw_temp_table(f, app_state, *draw_loc, widget.widget_id),
                    Disk => self.draw_disk_table(f, app_state, *draw_loc, widget.widget_id),
                    Proc => self.draw_process(f, app_state, *draw_loc, true, widget.widget_id),
                    Battery => self.draw_battery(f, app_state, *draw_loc, true, widget.widget_id),
                    _ => {}
                }
            }
        }
    }
}
