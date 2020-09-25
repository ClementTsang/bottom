use itertools::izip;
use std::{collections::HashMap, str::FromStr};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame, Terminal,
};

use canvas_colours::*;
use dialogs::*;
use screens::*;
use widgets::*;

use crate::{
    app::{
        self,
        layout_manager::{BottomColRow, BottomLayout, BottomWidgetType},
        App,
    },
    constants::*,
    data_conversion::{ConvertedBatteryData, ConvertedCpuData, ConvertedProcessData},
    options::Config,
    utils::error,
    utils::error::BottomError,
    Pid,
};

mod canvas_colours;
mod dialogs;
mod drawing_utils;
mod screens;
mod widgets;

/// Point is of time, data
type Point = (f64, f64);

#[derive(Default)]
pub struct DisplayableData {
    pub rx_display: String,
    pub tx_display: String,
    pub total_rx_display: String,
    pub total_tx_display: String,
    pub network_data_rx: Vec<Point>,
    pub network_data_tx: Vec<Point>,
    pub disk_data: Vec<Vec<String>>,
    pub temp_sensor_data: Vec<Vec<String>>,
    pub single_process_data: HashMap<Pid, ConvertedProcessData>, // Contains single process data, key is PID
    pub finalized_process_data_map: HashMap<u64, Vec<ConvertedProcessData>>, // What's actually displayed, key is the widget ID.
    pub stringified_process_data_map: HashMap<u64, Vec<(Vec<(String, Option<String>)>, bool)>>, // Represents the row and whether it is disabled, key is the widget ID
    pub mem_label_percent: String,
    pub swap_label_percent: String,
    pub mem_label_frac: String,
    pub swap_label_frac: String,
    pub mem_data: Vec<Point>,
    pub swap_data: Vec<Point>,
    pub cpu_data: Vec<ConvertedCpuData>,
    pub battery_data: Vec<ConvertedBatteryData>,
}

#[derive(Debug)]
pub enum ColourScheme {
    Default,
    DefaultLight,
    Gruvbox,
    GruvboxLight,
    // Nord,
    Custom,
}

impl FromStr for ColourScheme {
    type Err = BottomError;

    fn from_str(s: &str) -> error::Result<Self> {
        let lower_case = s.to_lowercase();
        match lower_case.as_str() {
            "default" => Ok(ColourScheme::Default),
            "default-light" => Ok(ColourScheme::DefaultLight),
            "gruvbox" => Ok(ColourScheme::Gruvbox),
            "gruvbox-light" => Ok(ColourScheme::GruvboxLight),
            // "nord" => Ok(ColourScheme::Nord),
            _ => Err(BottomError::ConfigError(format!(
                "\"{}\" is an invalid built-in color scheme.",
                s
            ))),
        }
    }
}

/// Handles the canvas' state.  TODO: [OPT] implement this.
pub struct Painter {
    pub colours: CanvasColours,
    height: u16,
    width: u16,
    styled_help_text: Vec<Spans<'static>>,
    is_mac_os: bool,
    row_constraints: Vec<Constraint>,
    col_constraints: Vec<Vec<Constraint>>,
    col_row_constraints: Vec<Vec<Vec<Constraint>>>,
    layout_constraints: Vec<Vec<Vec<Vec<Constraint>>>>,
    derived_widget_draw_locs: Vec<Vec<Vec<Vec<Rect>>>>,
    widget_layout: BottomLayout,
    table_height_offset: u16,
}

impl Painter {
    pub fn init(
        widget_layout: BottomLayout, table_gap: u16, is_basic_mode: bool, config: &Config,
        colour_scheme: ColourScheme,
    ) -> anyhow::Result<Self> {
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

        let mut painter = Painter {
            colours: CanvasColours::default(),
            height: 0,
            width: 0,
            styled_help_text: Vec::default(),
            is_mac_os: false,
            row_constraints,
            col_constraints,
            col_row_constraints,
            layout_constraints,
            widget_layout,
            derived_widget_draw_locs: Vec::default(),
            table_height_offset: if is_basic_mode { 2 } else { 4 } + table_gap,
        };

        if let ColourScheme::Custom = colour_scheme {
            painter.generate_config_colours(config)?;
        } else {
            painter.generate_colour_scheme(colour_scheme)?;
        }
        painter.complete_painter_init();

        Ok(painter)
    }

    fn generate_config_colours(&mut self, config: &Config) -> anyhow::Result<()> {
        if let Some(colours) = &config.colors {
            self.colours.set_colours_from_palette(colours)?;
        }

        Ok(())
    }

    fn generate_colour_scheme(&mut self, colour_scheme: ColourScheme) -> anyhow::Result<()> {
        match colour_scheme {
            ColourScheme::Default => {
                // Don't have to do anything.
            }
            ColourScheme::DefaultLight => {
                self.colours
                    .set_colours_from_palette(&*DEFAULT_LIGHT_MODE_COLOUR_PALETTE)?;
            }
            ColourScheme::Gruvbox => {
                self.colours
                    .set_colours_from_palette(&*GRUVBOX_COLOUR_PALETTE)?;
            }
            ColourScheme::GruvboxLight => {
                self.colours
                    .set_colours_from_palette(&*GRUVBOX_LIGHT_COLOUR_PALETTE)?;
            }
            // ColourScheme::Nord => {
            //     self.colours
            //         .set_colours_from_palette(&*NORD_COLOUR_PALETTE)?;
            // }
            ColourScheme::Custom => {
                // This case should never occur, just do nothing.
            }
        }

        Ok(())
    }

    /// Must be run once before drawing, but after setting colours.
    /// This is to set some remaining styles and text.
    fn complete_painter_init(&mut self) {
        self.is_mac_os = cfg!(target_os = "macos");
        let mut styled_help_spans = Vec::new();

        // Init help text:
        (*HELP_TEXT).iter().enumerate().for_each(|(itx, section)| {
            if itx == 0 {
                styled_help_spans.extend(
                    section
                        .iter()
                        .map(|&text| Span::styled(text, self.colours.text_style))
                        .collect::<Vec<_>>(),
                );
            } else {
                // Not required check but it runs only a few times... so whatever ig, prevents me from
                // being dumb and leaving a help text section only one line long.
                if section.len() > 1 {
                    styled_help_spans.push(Span::raw(""));
                    styled_help_spans
                        .push(Span::styled(section[0], self.colours.table_header_style));
                    styled_help_spans.extend(
                        section[1..]
                            .iter()
                            .map(|&text| Span::styled(text, self.colours.text_style))
                            .collect::<Vec<_>>(),
                    );
                }
            }
        });

        self.styled_help_text = styled_help_spans.into_iter().map(Spans::from).collect();
    }

    // FIXME: [CONFIG] write this, should call painter init and any changed colour functions...
    pub fn update_painter_colours(&mut self) {}

    fn draw_frozen_indicator<B: Backend>(&self, f: &mut Frame<'_, B>, draw_loc: Rect) {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Frozen, press 'f' to unfreeze",
                self.colours.currently_selected_text_style,
            )),
            Layout::default()
                .horizontal_margin(1)
                .constraints([Constraint::Length(1)])
                .split(draw_loc)[0],
        )
    }

    pub fn draw_data<B: Backend>(
        &mut self, terminal: &mut Terminal<B>, app_state: &mut app::App,
    ) -> error::Result<()> {
        use BottomWidgetType::*;

        terminal.draw(|mut f| {
            let (terminal_size, frozen_draw_loc) = if app_state.is_frozen {
                let split_loc = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(f.size());
                (split_loc[0], Some(split_loc[1]))
            } else {
                (f.size(), None)
            };
            let terminal_height = terminal_size.height;
            let terminal_width = terminal_size.width;

            if (self.height == 0 && self.width == 0)
                || (self.height != terminal_height || self.width != terminal_width)
            {
                app_state.is_force_redraw = true;
                self.height = terminal_height;
                self.width = terminal_width;
            }

            if app_state.should_get_widget_bounds() {
                // If we're force drawing, reset ALL mouse boundaries.
                for widget in app_state.widget_map.values_mut() {
                    widget.top_left_corner = None;
                    widget.bottom_right_corner = None;
                }

                // And reset dd_dialog...
                app_state.delete_dialog_state.yes_tlc = None;
                app_state.delete_dialog_state.yes_brc = None;
                app_state.delete_dialog_state.no_tlc = None;
                app_state.delete_dialog_state.no_brc = None;

                // And battery dialog...
                for battery_widget in app_state.battery_state.widget_states.values_mut() {
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
                    })
                    .split(vertical_dialog_chunk[1]);

                self.draw_help_dialog(&mut f, app_state, middle_dialog_chunk[1]);
            } else if app_state.delete_dialog_state.is_showing_dd {
                // TODO: This needs the paragraph wrap feature from tui-rs to be pushed to complete... but for now it's pretty close!
                // The main problem right now is that I cannot properly calculate the height offset since
                // line-wrapping is NOT the same as taking the width of the text and dividing by width.
                // So, I need the height AFTER wrapping.
                // See: https://github.com/fdehau/tui-rs/pull/349.  Land this after this pushes to release.

                let dd_text = self.get_dd_spans(app_state);

                let (text_width, text_height) = (
                    if terminal_width < 100 {
                        terminal_width * 90 / 100
                    } else {
                        terminal_width * 50 / 100
                    },
                    22,
                );
                // let (text_width, text_height) = if let Some(dd_text) = &dd_text {
                //     let width = if current_width < 100 {
                //         current_width * 90 / 100
                //     } else {
                //         let min_possible_width = (current_width * 50 / 100) as usize;
                //         let mut width = dd_text.width();

                //         // This should theoretically never allow width to be 0... we can be safe and do an extra check though.
                //         while width > (current_width as usize) && width / 2 > min_possible_width {
                //             width /= 2;
                //         }

                //         std::cmp::max(width, min_possible_width) as u16
                //     };

                //     (
                //         width,
                //         (dd_text.height() + 2 + (dd_text.width() / width as usize)) as u16,
                //     )
                // } else {
                //     // AFAIK this shouldn't happen, unless something went wrong...
                //     (
                //         if current_width < 100 {
                //             current_width * 90 / 100
                //         } else {
                //             current_width * 50 / 100
                //         },
                //         7,
                //     )
                // };

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
                    self.draw_dd_dialog(&mut f, dd_text, app_state, middle_dialog_chunk[1]);
            } else if app_state.is_expanded {
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(&mut f, frozen_draw_loc);
                }

                let rect = Layout::default()
                    .margin(0)
                    .constraints([Constraint::Percentage(100)])
                    .split(terminal_size);
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
                    Proc | ProcSearch | ProcSort => {
                        let widget_id = app_state.current_widget.widget_id
                            - match &app_state.current_widget.widget_type {
                                ProcSearch => 1,
                                ProcSort => 2,
                                _ => 0,
                            };

                        self.draw_process_features(&mut f, app_state, rect[0], true, widget_id);
                    }
                    Battery => self.draw_battery_display(
                        &mut f,
                        app_state,
                        rect[0],
                        true,
                        app_state.current_widget.widget_id,
                    ),
                    _ => {}
                }
            } else if app_state.is_config_open {
                let rect = Layout::default()
                    .margin(0)
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size())[0];

                self.draw_config_screen(&mut f, app_state, rect)
            } else if app_state.app_config_fields.use_basic_mode {
                // Basic mode.  This basically removes all graphs but otherwise
                // the same info.
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(&mut f, frozen_draw_loc);
                }

                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(
                            (app_state.canvas_data.cpu_data.len() / 4) as u16
                                + (if app_state.canvas_data.cpu_data.len() % 4 == 0 {
                                    0
                                } else {
                                    1
                                }),
                        ),
                        Constraint::Length(1),
                        Constraint::Length(2),
                        Constraint::Length(2),
                        Constraint::Min(5),
                    ])
                    .split(terminal_size);

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(vertical_chunks[2]);
                self.draw_basic_cpu(&mut f, app_state, vertical_chunks[0], 1);
                self.draw_basic_memory(&mut f, app_state, middle_chunks[0], 2);
                self.draw_basic_network(&mut f, app_state, middle_chunks[1], 3);

                let mut later_widget_id: Option<u64> = None;
                if let Some(basic_table_widget_state) = &app_state.basic_table_widget_state {
                    let widget_id = basic_table_widget_state.currently_displayed_widget_id;
                    later_widget_id = Some(widget_id);
                    match basic_table_widget_state.currently_displayed_widget_type {
                        Disk => self.draw_disk_table(
                            &mut f,
                            app_state,
                            vertical_chunks[4],
                            false,
                            widget_id,
                        ),
                        Proc | ProcSort => {
                            let wid = widget_id
                                - match basic_table_widget_state.currently_displayed_widget_type {
                                    ProcSearch => 1,
                                    ProcSort => 2,
                                    _ => 0,
                                };
                            self.draw_process_features(
                                &mut f,
                                app_state,
                                vertical_chunks[4],
                                false,
                                wid,
                            );
                        }
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

                if let Some(widget_id) = later_widget_id {
                    self.draw_basic_table_arrows(&mut f, app_state, vertical_chunks[3], widget_id);
                }
            } else {
                // Draws using the passed in (or default) layout.
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(&mut f, frozen_draw_loc);
                }

                if self.derived_widget_draw_locs.is_empty() || app_state.is_force_redraw {
                    let row_draw_locs = Layout::default()
                        .margin(0)
                        .constraints(self.row_constraints.as_ref())
                        .direction(Direction::Vertical)
                        .split(terminal_size);
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
        app_state.is_determining_widget_boundary = false;

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
                Proc => self.draw_process_features(
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
