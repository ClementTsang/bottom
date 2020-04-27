use itertools::izip;
use std::cmp::max;
use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Text,
    Frame, Terminal,
};

use canvas_colours::*;
use dialogs::*;
use widgets::*;

use crate::{
    app::{
        self,
        data_harvester::processes::ProcessHarvest,
        layout_manager::{BottomColRow, BottomLayout, BottomWidgetType},
        App,
    },
    constants::*,
    data_conversion::{ConvertedBatteryData, ConvertedCpuData, ConvertedProcessData},
    utils::error,
};

mod canvas_colours;
mod dialogs;
mod drawing_utils;
mod widgets;

#[derive(Default)]
pub struct DisplayableData {
    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: String,
    pub total_tx_display: String,
    pub network_data_rx: Vec<(f64, f64)>,
    pub network_data_tx: Vec<(f64, f64)>,
    pub disk_data: Vec<Vec<String>>,
    pub temp_sensor_data: Vec<Vec<String>>,
    // Not the final value
    pub process_data: HashMap<u32, ProcessHarvest>,
    // Not the final value
    pub grouped_process_data: Vec<ConvertedProcessData>,
    // What's actually displayed
    pub finalized_process_data_map: HashMap<u64, Vec<ConvertedProcessData>>,
    pub mem_label: String,
    pub swap_label: String,
    pub mem_data: Vec<(f64, f64)>,
    pub swap_data: Vec<(f64, f64)>,
    pub cpu_data: Vec<ConvertedCpuData>,
    pub battery_data: Vec<ConvertedBatteryData>,
}

/// Handles the canvas' state.  TODO: [OPT] implement this.
pub struct Painter {
    pub colours: CanvasColours,
    height: u16,
    width: u16,
    styled_help_text: Vec<Text<'static>>,
    is_mac_os: bool,
    row_constraints: Vec<Constraint>,
    col_constraints: Vec<Vec<Constraint>>,
    col_row_constraints: Vec<Vec<Vec<Constraint>>>,
    layout_constraints: Vec<Vec<Vec<Vec<Constraint>>>>,
    widget_layout: BottomLayout,
    derived_widget_draw_locs: Vec<Vec<Vec<Vec<Rect>>>>,
    table_height_offset: i64,
}

impl Painter {
    pub fn init(widget_layout: BottomLayout, table_gap: u16) -> Self {
        // Now for modularity; we have to also initialize the base layouts!
        // We want to do this ONCE and reuse; after this we can just construct
        // based on the console size.

        let mut row_constraints = Vec::new();
        let mut col_constraints = Vec::new();
        let mut col_row_constraints = Vec::new();
        let mut layout_constraints = Vec::new();

        widget_layout.rows.iter().for_each(|row| {
            if row.canvas_handle_height {
                row_constraints.push(Constraint::Length(0));
            } else {
                row_constraints.push(Constraint::Ratio(
                    row.row_height_ratio,
                    widget_layout.total_row_height_ratio,
                ));
            }

            let mut new_col_constraints = Vec::new();
            let mut new_widget_constraints = Vec::new();
            let mut new_col_row_constraints = Vec::new();
            row.children.iter().for_each(|col| {
                if col.canvas_handle_width {
                    new_col_constraints.push(Constraint::Length(0));
                } else {
                    new_col_constraints
                        .push(Constraint::Ratio(col.col_width_ratio, row.total_col_ratio));
                }

                let mut new_new_col_row_constraints = Vec::new();
                let mut new_new_widget_constraints = Vec::new();
                col.children.iter().for_each(|col_row| {
                    if col_row.canvas_handle_height {
                        new_new_col_row_constraints.push(Constraint::Length(0));
                    } else if col_row.flex_grow {
                        new_new_col_row_constraints.push(Constraint::Min(0));
                    } else {
                        new_new_col_row_constraints.push(Constraint::Ratio(
                            col_row.col_row_height_ratio,
                            col.total_col_row_ratio,
                        ));
                    }

                    let mut new_new_new_widget_constraints = Vec::new();
                    col_row.children.iter().for_each(|widget| {
                        if widget.canvas_handle_width {
                            new_new_new_widget_constraints.push(Constraint::Length(0));
                        } else if widget.flex_grow {
                            new_new_new_widget_constraints.push(Constraint::Min(0));
                        } else {
                            new_new_new_widget_constraints.push(Constraint::Ratio(
                                widget.width_ratio,
                                col_row.total_widget_ratio,
                            ));
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

        Painter {
            colours: CanvasColours::default(),
            height: 0,
            width: 0,
            styled_help_text: Vec::new(),
            is_mac_os: false,
            row_constraints,
            col_constraints,
            col_row_constraints,
            layout_constraints,
            widget_layout,
            derived_widget_draw_locs: Vec::new(),
            table_height_offset: 4 + table_gap as i64,
        }
    }

    /// Must be run once before drawing, but after setting colours.
    /// This is to set some remaining styles and text.
    pub fn complete_painter_init(&mut self) {
        self.is_mac_os = cfg!(target_os = "macos");

        // Init help text:
        (*HELP_TEXT).iter().enumerate().for_each(|(itx, section)| {
            if itx == 0 {
                self.styled_help_text.extend(
                    section
                        .iter()
                        .map(|&text| Text::Styled(text.into(), self.colours.text_style))
                        .collect::<Vec<_>>(),
                );
            } else {
                // Not required check but it runs only a few times... so whatever ig, prevents me from
                // being dumb and leaving a help text section only one line long.
                if section.len() > 1 {
                    self.styled_help_text.push(Text::Raw("\n\n".into()));
                    self.styled_help_text.push(Text::Styled(
                        section[0].into(),
                        self.colours.table_header_style,
                    ));
                    self.styled_help_text.extend(
                        section[1..]
                            .iter()
                            .map(|&text| Text::Styled(text.into(), self.colours.text_style))
                            .collect::<Vec<_>>(),
                    );
                }
            }
        });
    }

    pub fn draw_data<B: Backend>(
        &mut self, terminal: &mut Terminal<B>, app_state: &mut app::App,
    ) -> error::Result<()> {
        use BottomWidgetType::*;

        let terminal_size = terminal.size()?;
        let current_height = terminal_size.height;
        let current_width = terminal_size.width;

        if (self.height == 0 && self.width == 0)
            || (self.height != current_height || self.width != current_width)
        {
            app_state.is_force_redraw = true;
            self.height = current_height;
            self.width = current_width;
        }

        terminal.autoresize()?;
        terminal.draw(|mut f| {
            if app_state.help_dialog_state.is_showing_help {
                let gen_help_len = GENERAL_HELP_TEXT.len() as u16 + 3;
                let border_len = (max(0, f.size().height as i64 - gen_help_len as i64)) as u16 / 2;
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(border_len),
                            Constraint::Length(gen_help_len),
                            Constraint::Length(border_len),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let middle_dialog_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        if f.size().width < 100 {
                            // TODO: [REFACTOR] The point we start changing size at currently hard-coded in.
                            [
                                Constraint::Percentage(0),
                                Constraint::Percentage(100),
                                Constraint::Percentage(0),
                            ]
                        } else {
                            [
                                Constraint::Percentage(20),
                                Constraint::Percentage(60),
                                Constraint::Percentage(20),
                            ]
                        }
                        .as_ref(),
                    )
                    .split(vertical_dialog_chunk[1]);

                self.draw_help_dialog(&mut f, app_state, middle_dialog_chunk[1]);
            } else if app_state.delete_dialog_state.is_showing_dd {
                let bordering = (max(0, f.size().height as i64 - 7) as u16) / 2;
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(bordering),
                            Constraint::Length(7),
                            Constraint::Length(bordering),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let middle_dialog_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        if f.size().width < 100 {
                            // TODO: [REFACTOR] The point we start changing size at currently hard-coded in.
                            [
                                Constraint::Percentage(5),
                                Constraint::Percentage(90),
                                Constraint::Percentage(5),
                            ]
                        } else {
                            [
                                Constraint::Percentage(30),
                                Constraint::Percentage(40),
                                Constraint::Percentage(30),
                            ]
                        }
                        .as_ref(),
                    )
                    .split(vertical_dialog_chunk[1]);

                if let Some(dd_err) = &app_state.dd_err {
                    self.draw_dd_error_dialog(&mut f, dd_err, middle_dialog_chunk[1]);
                } else {
                    // This is a bit nasty, but it works well... I guess.
                    app_state.delete_dialog_state.is_showing_dd =
                        self.draw_dd_dialog(&mut f, app_state, middle_dialog_chunk[1]);
                }
            } else if app_state.is_expanded {
                let rect = Layout::default()
                    .margin(0)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());
                match &app_state.current_widget.widget_type {
                    Cpu => self.draw_cpu(
                        &mut f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                    ),
                    CpuLegend => self.draw_cpu(
                        &mut f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id - 1,
                    ),
                    Mem | BasicMem => self.draw_memory_graph(
                        &mut f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                    ),
                    Disk => self.draw_disk_table(
                        &mut f,
                        app_state,
                        rect[0],
                        true,
                        app_state.current_widget.widget_id,
                    ),
                    Temp => self.draw_temp_table(
                        &mut f,
                        app_state,
                        rect[0],
                        true,
                        app_state.current_widget.widget_id,
                    ),
                    Net => self.draw_network_graph(
                        &mut f,
                        app_state,
                        rect[0],
                        app_state.current_widget.widget_id,
                        false,
                    ),
                    Proc => self.draw_process_and_search(
                        &mut f,
                        app_state,
                        rect[0],
                        true,
                        app_state.current_widget.widget_id,
                    ),
                    ProcSearch => self.draw_process_and_search(
                        &mut f,
                        app_state,
                        rect[0],
                        true,
                        app_state.current_widget.widget_id - 1,
                    ),
                    Battery => self.draw_battery_display(
                        &mut f,
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

                let cpu_height = (app_state.canvas_data.cpu_data.len() / 4) as u16
                    + (if app_state.canvas_data.cpu_data.len() % 4 == 0 {
                        0
                    } else {
                        1
                    });
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(cpu_height),
                            Constraint::Length(1),
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Min(5),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(vertical_chunks[2]);
                self.draw_basic_cpu(&mut f, app_state, vertical_chunks[0], 1);
                self.draw_basic_memory(&mut f, app_state, middle_chunks[0], 2);
                self.draw_basic_network(&mut f, app_state, middle_chunks[1], 3);
                self.draw_basic_table_arrows(&mut f, app_state, vertical_chunks[3]);
                if let Some(basic_table_widget_state) = &app_state.basic_table_widget_state {
                    let widget_id = basic_table_widget_state.currently_displayed_widget_id;
                    match basic_table_widget_state.currently_displayed_widget_type {
                        Disk => self.draw_disk_table(
                            &mut f,
                            app_state,
                            vertical_chunks[4],
                            false,
                            widget_id,
                        ),
                        Proc => self.draw_process_and_search(
                            &mut f,
                            app_state,
                            vertical_chunks[4],
                            false,
                            widget_id,
                        ),
                        Temp => self.draw_temp_table(
                            &mut f,
                            app_state,
                            vertical_chunks[4],
                            false,
                            widget_id,
                        ),
                        Battery => self.draw_battery_display(
                            &mut f,
                            app_state,
                            vertical_chunks[4],
                            false,
                            widget_id,
                        ),
                        _ => {}
                    }
                }
            } else {
                // Draws using the passed in (or default) layout.  NOT basic so far.
                if self.derived_widget_draw_locs.is_empty() || app_state.is_force_redraw {
                    let row_draw_locs = Layout::default()
                        .margin(0)
                        .constraints(self.row_constraints.as_ref())
                        .direction(Direction::Vertical)
                        .split(f.size());
                    let col_draw_locs = self
                        .col_constraints
                        .iter()
                        .zip(&row_draw_locs)
                        .map(|(col_constraint, row_draw_loc)| {
                            Layout::default()
                                .constraints(col_constraint.as_ref())
                                .direction(Direction::Horizontal)
                                .split(*row_draw_loc)
                        })
                        .collect::<Vec<_>>();
                    let col_row_draw_locs = self
                        .col_row_constraints
                        .iter()
                        .zip(&col_draw_locs)
                        .map(|(col_row_constraints, row_draw_loc)| {
                            col_row_constraints
                                .iter()
                                .zip(row_draw_loc)
                                .map(|(col_row_constraint, col_draw_loc)| {
                                    Layout::default()
                                        .constraints(col_row_constraint.as_ref())
                                        .direction(Direction::Vertical)
                                        .split(*col_draw_loc)
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();

                    // Now... draw!
                    let mut new_derived_widget_draw_locs = Vec::new();
                    izip!(
                        &self.layout_constraints,
                        col_row_draw_locs,
                        &self.widget_layout.rows
                    )
                    .for_each(|(row_constraint_vec, row_draw_loc, cols)| {
                        let mut derived_row_draw_locs = Vec::new();
                        izip!(row_constraint_vec, row_draw_loc, &cols.children).for_each(
                            |(col_constraint_vec, col_draw_loc, col_rows)| {
                                let mut derived_col_draw_locs = Vec::new();
                                izip!(col_constraint_vec, col_draw_loc, &col_rows.children)
                                    .for_each(
                                        |(col_row_constraint_vec, col_row_draw_loc, widgets)| {
                                            // Note that col_row_constraint_vec CONTAINS the widget constraints
                                            let widget_draw_locs = Layout::default()
                                                .constraints(col_row_constraint_vec.as_ref())
                                                .direction(Direction::Horizontal)
                                                .split(col_row_draw_loc);

                                            self.draw_widgets_with_constraints(
                                                &mut f,
                                                app_state,
                                                widgets,
                                                &widget_draw_locs,
                                            );

                                            derived_col_draw_locs.push(widget_draw_locs);
                                        },
                                    );
                                derived_row_draw_locs.push(derived_col_draw_locs);
                            },
                        );
                        new_derived_widget_draw_locs.push(derived_row_draw_locs);
                    });
                    self.derived_widget_draw_locs = new_derived_widget_draw_locs;
                } else {
                    self.widget_layout
                        .rows
                        .iter()
                        .zip(&self.derived_widget_draw_locs)
                        .for_each(|(cols, row_layout)| {
                            cols.children.iter().zip(row_layout).for_each(
                                |(col_rows, col_row_layout)| {
                                    col_rows.children.iter().zip(col_row_layout).for_each(
                                        |(widgets, widget_draw_locs)| {
                                            self.draw_widgets_with_constraints(
                                                &mut f,
                                                app_state,
                                                widgets,
                                                &widget_draw_locs,
                                            );
                                        },
                                    );
                                },
                            );
                        });
                }
            }
        })?;

        app_state.is_force_redraw = false;

        Ok(())
    }

    fn draw_widgets_with_constraints<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut App, widgets: &BottomColRow,
        widget_draw_locs: &[Rect],
    ) {
        use BottomWidgetType::*;
        for (widget, widget_draw_loc) in widgets.children.iter().zip(widget_draw_locs) {
            match &widget.widget_type {
                Empty => {}
                Cpu => self.draw_cpu(f, app_state, *widget_draw_loc, widget.widget_id),
                Mem => self.draw_memory_graph(f, app_state, *widget_draw_loc, widget.widget_id),
                Net => self.draw_network(f, app_state, *widget_draw_loc, widget.widget_id),
                Temp => {
                    self.draw_temp_table(f, app_state, *widget_draw_loc, true, widget.widget_id)
                }
                Disk => {
                    self.draw_disk_table(f, app_state, *widget_draw_loc, true, widget.widget_id)
                }
                Proc => self.draw_process_and_search(
                    f,
                    app_state,
                    *widget_draw_loc,
                    true,
                    widget.widget_id,
                ),
                Battery => self.draw_battery_display(
                    f,
                    app_state,
                    *widget_draw_loc,
                    true,
                    widget.widget_id,
                ),
                _ => {}
            }
        }
    }
}
