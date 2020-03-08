use std::cmp::max;
use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::Text,
    Terminal,
};

use canvas_colours::*;
use dialogs::*;
use widgets::*;

use crate::{
    app::{self, data_harvester::processes::ProcessHarvest, WidgetPosition},
    constants::*,
    data_conversion::{ConvertedCpuData, ConvertedProcessData},
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
    pub process_data: HashMap<u32, ProcessHarvest>,
    // Not the final value
    pub grouped_process_data: Vec<ConvertedProcessData>,
    // Not the final value
    pub finalized_process_data: Vec<ConvertedProcessData>,
    // What's actually displayed
    pub mem_label: String,
    pub swap_label: String,
    pub mem_data: Vec<(f64, f64)>,
    pub swap_data: Vec<(f64, f64)>,
    pub cpu_data: Vec<ConvertedCpuData>,
}

#[allow(dead_code)]
#[derive(Default)]
/// Handles the canvas' state.  TODO: [OPT] implement this.
pub struct Painter {
    height: u16,
    width: u16,
    vertical_dialog_chunk: Vec<Rect>,
    middle_dialog_chunk: Vec<Rect>,
    vertical_chunks: Vec<Rect>,
    middle_chunks: Vec<Rect>,
    middle_divided_chunk_2: Vec<Rect>,
    bottom_chunks: Vec<Rect>,
    cpu_chunk: Vec<Rect>,
    network_chunk: Vec<Rect>,
    pub colours: CanvasColours,
    pub styled_general_help_text: Vec<Text<'static>>,
    pub styled_process_help_text: Vec<Text<'static>>,
    pub styled_search_help_text: Vec<Text<'static>>,
    is_mac_os: bool,
}

impl Painter {
    /// Must be run once before drawing, but after setting colours.
    /// This is to set some remaining styles and text.
    /// This bypasses some logic checks (size > 2, for example) but this
    /// assumes that you, the programmer, are sane and do not do stupid things.
    /// RIGHT?
    pub fn initialize(&mut self) {
        self.is_mac_os = cfg!(target_os = "macos");

        self.styled_general_help_text.push(Text::Styled(
            GENERAL_HELP_TEXT[0].into(),
            self.colours.table_header_style,
        ));
        self.styled_general_help_text.extend(
            GENERAL_HELP_TEXT[1..]
                .iter()
                .map(|&text| Text::Styled(text.into(), self.colours.text_style))
                .collect::<Vec<_>>(),
        );

        self.styled_process_help_text.push(Text::Styled(
            PROCESS_HELP_TEXT[0].into(),
            self.colours.table_header_style,
        ));
        self.styled_process_help_text.extend(
            PROCESS_HELP_TEXT[1..]
                .iter()
                .map(|&text| Text::Styled(text.into(), self.colours.text_style))
                .collect::<Vec<_>>(),
        );

        self.styled_search_help_text.push(Text::Styled(
            SEARCH_HELP_TEXT[0].into(),
            self.colours.table_header_style,
        ));
        self.styled_search_help_text.extend(
            SEARCH_HELP_TEXT[1..]
                .iter()
                .map(|&text| Text::Styled(text.into(), self.colours.text_style))
                .collect::<Vec<_>>(),
        );
    }

    pub fn draw_specific_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_selected: WidgetPosition,
    ) {
        match widget_selected {
            WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                self.draw_process_and_search(f, app_state, draw_loc, draw_border)
            }
            WidgetPosition::Temp => self.draw_temp_table(f, app_state, draw_loc, draw_border),
            WidgetPosition::Disk => self.draw_disk_table(f, app_state, draw_loc, draw_border),
            _ => {}
        }
    }

    // TODO: [REFACTOR] We should clean this up tbh
    // TODO: [FEATURE] Auto-resizing dialog sizes.
    #[allow(clippy::cognitive_complexity)]
    pub fn draw_data<B: Backend>(
        &mut self, terminal: &mut Terminal<B>, app_state: &mut app::App,
    ) -> error::Result<()> {
        let terminal_size = terminal.size()?;
        let current_height = terminal_size.height;
        let current_width = terminal_size.width;

        // TODO: [OPT] we might be able to add an argument s.t. if there is
        // no resize AND it's not a data update (or process refresh/search/etc.)
        // then just... don't draw again!
        if self.height == 0 && self.width == 0 {
            self.height = current_height;
            self.width = current_width;
        } else if self.height != current_height || self.width != current_width {
            app_state.is_resized = true;
        }

        terminal.autoresize()?;
        terminal.draw(|mut f| {
            if app_state.help_dialog_state.is_showing_help {
                // TODO: [RESIZE] Scrolling dialog boxes is ideal.  This is currently VERY temporary!
                // The width is currently not good and can wrap... causing this to not go so well!
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
                    .margin(1)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());
                match &app_state.current_widget_selected {
                    WidgetPosition::Cpu | WidgetPosition::BasicCpu => {
                        let cpu_chunk = Layout::default()
                            .direction(Direction::Horizontal)
                            .margin(0)
                            .constraints(
                                if app_state.app_config_fields.left_legend {
                                    [Constraint::Percentage(15), Constraint::Percentage(85)]
                                } else {
                                    [Constraint::Percentage(85), Constraint::Percentage(15)]
                                }
                                .as_ref(),
                            )
                            .split(rect[0]);

                        let legend_index = if app_state.app_config_fields.left_legend {
                            0
                        } else {
                            1
                        };
                        let graph_index = if app_state.app_config_fields.left_legend {
                            1
                        } else {
                            0
                        };

                        self.draw_cpu_graph(&mut f, &app_state, cpu_chunk[graph_index]);
                        self.draw_cpu_legend(&mut f, app_state, cpu_chunk[legend_index]);
                    }
                    WidgetPosition::Mem | WidgetPosition::BasicMem => {
                        self.draw_memory_graph(&mut f, &app_state, rect[0]);
                    }
                    WidgetPosition::Disk => {
                        self.draw_disk_table(&mut f, app_state, rect[0], true);
                    }
                    WidgetPosition::Temp => {
                        self.draw_temp_table(&mut f, app_state, rect[0], true);
                    }
                    WidgetPosition::Network | WidgetPosition::BasicNet => {
                        self.draw_network_graph(&mut f, &app_state, rect[0]);
                    }
                    WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                        self.draw_process_and_search(&mut f, app_state, rect[0], true);
                    }
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
                self.draw_basic_cpu(&mut f, app_state, vertical_chunks[0]);
                self.draw_basic_memory(&mut f, app_state, middle_chunks[0]);
                self.draw_basic_network(&mut f, app_state, middle_chunks[1]);
                self.draw_basic_table_arrows(&mut f, app_state, vertical_chunks[3]);
                if app_state.current_widget_selected.is_widget_table() {
                    self.draw_specific_table(
                        &mut f,
                        app_state,
                        vertical_chunks[4],
                        false,
                        app_state.current_widget_selected,
                    );
                } else {
                    self.draw_specific_table(
                        &mut f,
                        app_state,
                        vertical_chunks[4],
                        false,
                        app_state.previous_basic_table_selected,
                    );
                }
            } else {
                // TODO: [TUI] Change this back to a more even 33/33/34 when TUI releases
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Percentage(30),
                            Constraint::Percentage(37),
                            Constraint::Percentage(33),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(vertical_chunks[1]);

                let middle_divided_chunk_2 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(middle_chunks[1]);

                let bottom_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(vertical_chunks[2]);

                // Component specific chunks
                let cpu_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        if app_state.app_config_fields.left_legend {
                            [Constraint::Percentage(15), Constraint::Percentage(85)]
                        } else {
                            [Constraint::Percentage(85), Constraint::Percentage(15)]
                        }
                        .as_ref(),
                    )
                    .split(vertical_chunks[0]);

                let network_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        if (bottom_chunks[0].height as f64 * 0.25) as u16 >= 4 {
                            [Constraint::Percentage(75), Constraint::Percentage(25)]
                        } else {
                            let required = if bottom_chunks[0].height < 10 {
                                bottom_chunks[0].height / 2
                            } else {
                                5
                            };
                            let remaining = bottom_chunks[0].height - required;
                            [Constraint::Length(remaining), Constraint::Length(required)]
                        }
                        .as_ref(),
                    )
                    .split(bottom_chunks[0]);

                // Default chunk index based on left or right legend setting
                let legend_index = if app_state.app_config_fields.left_legend {
                    0
                } else {
                    1
                };
                let graph_index = if app_state.app_config_fields.left_legend {
                    1
                } else {
                    0
                };

                self.draw_cpu_graph(&mut f, &app_state, cpu_chunk[graph_index]);
                self.draw_cpu_legend(&mut f, app_state, cpu_chunk[legend_index]);
                self.draw_memory_graph(&mut f, &app_state, middle_chunks[0]);
                self.draw_network_graph(&mut f, &app_state, network_chunk[0]);
                self.draw_network_labels(&mut f, app_state, network_chunk[1]);
                self.draw_temp_table(&mut f, app_state, middle_divided_chunk_2[0], true);
                self.draw_disk_table(&mut f, app_state, middle_divided_chunk_2[1], true);
                self.draw_process_and_search(&mut f, app_state, bottom_chunks[1], true);
            }
        })?;

        app_state.is_resized = false;

        Ok(())
    }
}
