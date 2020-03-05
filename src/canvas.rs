use std::cmp::{max, min};
use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::Frame,
    widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Paragraph, Row, Table, Text, Widget},
    Terminal,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use canvas_colours::*;
use drawing_utils::*;

use crate::{
    app::{self, data_harvester::processes::ProcessHarvest, WidgetPosition},
    constants::*,
    data_conversion::{ConvertedCpuData, ConvertedProcessData},
    utils::error,
};

mod canvas_colours;
mod drawing_utils;

// Headers
const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
const CPU_SELECT_LEGEND_HEADER: [&str; 2] = ["CPU", "Show (Space)"];
const DISK_HEADERS: [&str; 7] = ["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"];
const TEMP_HEADERS: [&str; 2] = ["Sensor", "Temp"];
const MEM_HEADERS: [&str; 3] = ["Mem", "Usage", "Use%"];
const NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "Total RX", "Total TX"];
const FORCE_MIN_THRESHOLD: usize = 5;

lazy_static! {
    static ref SIDE_BORDERS: Borders = Borders::from_bits_truncate(20);
    static ref DEFAULT_TEXT_STYLE: Style = Style::default().fg(Color::Gray);
    static ref DEFAULT_HEADER_STYLE: Style = Style::default().fg(Color::LightBlue);
    static ref DISK_HEADERS_LENS: Vec<usize> = DISK_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref CPU_LEGEND_HEADER_LENS: Vec<usize> = CPU_LEGEND_HEADER
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref CPU_SELECT_LEGEND_HEADER_LENS: Vec<usize> = CPU_SELECT_LEGEND_HEADER
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref TEMP_HEADERS_LENS: Vec<usize> = TEMP_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref MEM_HEADERS_LENS: Vec<usize> = MEM_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
    static ref NETWORK_HEADERS_LENS: Vec<usize> = NETWORK_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
}

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
                // Only for the help
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(60),
                            Constraint::Percentage(20),
                        ]
                            .as_ref(),
                    )
                    .split(f.size());

                let middle_dialog_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(60),
                            Constraint::Percentage(20),
                        ]
                            .as_ref(),
                    )
                    .split(vertical_dialog_chunk[1]);

                const HELP_BASE: &str =
                    " Help ── 1: General ─── 2: Processes ─── 3: Search ─── Esc to close ";
                let repeat_num = max(
                    0,
                    middle_dialog_chunk[1].width as i32 - HELP_BASE.chars().count() as i32 - 2,
                );
                let help_title = format!(
                    " Help ─{}─ 1: General ─── 2: Processes ─── 3: Search ─── Esc to close ",
                    "─".repeat(repeat_num as usize)
                );

                Paragraph::new(
                    match app_state.help_dialog_state.current_category {
                        app::AppHelpCategory::General => &self.styled_general_help_text,
                        app::AppHelpCategory::Process => &self.styled_process_help_text,
                        app::AppHelpCategory::Search => &self.styled_search_help_text,
                    }
                        .iter(),
                )
                    .block(
                        Block::default()
                            .title(&help_title)
                            .title_style(self.colours.border_style)
                            .style(self.colours.border_style)
                            .borders(Borders::ALL)
                            .border_style(self.colours.border_style),
                    )
                    .style(self.colours.text_style)
                    .alignment(Alignment::Left)
                    .wrap(true)
                    .render(&mut f, middle_dialog_chunk[1]);
            } else if app_state.delete_dialog_state.is_showing_dd {
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Percentage(35),
                            Constraint::Percentage(30),
                            Constraint::Percentage(35),
                        ]
                            .as_ref(),
                    )
                    .split(f.size());

                let middle_dialog_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Percentage(25),
                            Constraint::Percentage(50),
                            Constraint::Percentage(25),
                        ]
                            .as_ref(),
                    )
                    .split(vertical_dialog_chunk[1]);

                if let Some(dd_err) = &app_state.dd_err {
                    let dd_text = [Text::raw(format!(
                        "\nFailure to properly kill the process - {}",
                        dd_err
                    ))];

                    const ERROR_BASE: &str = " Error ── Esc to close ";
                    let repeat_num = max(
                        0,
                        middle_dialog_chunk[1].width as i32 - ERROR_BASE.chars().count() as i32 - 2,
                    );
                    let error_title =
                        format!(" Error ─{}─ Esc to close ", "─".repeat(repeat_num as usize));

                    Paragraph::new(dd_text.iter())
                        .block(
                            Block::default()
                                .title(&error_title)
                                .title_style(self.colours.border_style)
                                .style(self.colours.border_style)
                                .borders(Borders::ALL)
                                .border_style(self.colours.border_style),
                        )
                        .style(self.colours.text_style)
                        .alignment(Alignment::Center)
                        .wrap(true)
                        .render(&mut f, middle_dialog_chunk[1]);
                } else if let Some(to_kill_processes) = app_state.get_to_delete_processes() {
                    if let Some(first_pid) = to_kill_processes.1.first() {
                        let dd_text = vec![
                            if app_state.is_grouped() {
                                if to_kill_processes.1.len() != 1 {
                                    Text::raw(format!(
                                        "\nAre you sure you want to kill {} processes with the name {}?",
                                        to_kill_processes.1.len(), to_kill_processes.0
                                    ))
                                } else {
                                    Text::raw(format!(
                                        "\nAre you sure you want to kill {} process with the name {}?",
                                        to_kill_processes.1.len(), to_kill_processes.0
                                    ))
                                }
                            } else {
                                Text::raw(format!(
                                    "\nAre you sure you want to kill process {} with PID {}?",
                                    to_kill_processes.0, first_pid
                                ))
                            },
                            Text::raw("\nNote that if bottom is frozen, it must be unfrozen for changes to be shown.\n\n\n"),
                            if app_state.delete_dialog_state.is_on_yes {
                                Text::styled("Yes", self.colours.currently_selected_text_style)
                            } else {
                                Text::raw("Yes")
                            },
                            Text::raw("                 "),
                            if app_state.delete_dialog_state.is_on_yes {
                                Text::raw("No")
                            } else {
                                Text::styled("No", self.colours.currently_selected_text_style)
                            },
                        ];

                        const DD_BASE: &str = " Confirm Kill Process ── Esc to close ";
                        let repeat_num = max(
                            0,
                            middle_dialog_chunk[1].width as i32
                                - DD_BASE.chars().count() as i32 - 2,
                        );
                        let dd_title = format!(
                            " Confirm Kill Process ─{}─ Esc to close ",
                            "─".repeat(repeat_num as usize)
                        );

                        Paragraph::new(dd_text.iter())
                            .block(
                                Block::default()
                                    .title(&dd_title)
                                    .title_style(self.colours.border_style)
                                    .style(self.colours.border_style)
                                    .borders(Borders::ALL)
                                    .border_style(self.colours.border_style),
                            )
                            .style(self.colours.text_style)
                            .alignment(Alignment::Center)
                            .wrap(true)
                            .render(&mut f, middle_dialog_chunk[1]);
                    } else {
                        // This is a bit nasty, but it works well... I guess.
                        app_state.delete_dialog_state.is_showing_dd = false;
                    }
                } else {
                    // This is a bit nasty, but it works well... I guess.
                    app_state.delete_dialog_state.is_showing_dd = false;
                }
            } else if app_state.is_expanded {
                // TODO: [REF] we should combine this with normal drawing tbh

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
                    + (
                    if app_state.canvas_data.cpu_data.len() % 4 == 0 {
                        0
                    } else {
                        1
                    }
                );
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(cpu_height),
                        Constraint::Length(1),
                        Constraint::Length(2),
                        Constraint::Length(2),
                        Constraint::Min(5),
                    ].as_ref())
                    .split(f.size());

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref())
                    .split(vertical_chunks[2]);
                self.draw_basic_cpu(&mut f, app_state, vertical_chunks[0]);
                self.draw_basic_memory(&mut f, app_state, middle_chunks[0]);
                self.draw_basic_network(&mut f, app_state, middle_chunks[1]);
                self.draw_basic_table_arrows(&mut f, app_state, vertical_chunks[3]);
                if app_state.current_widget_selected.is_widget_table() {
                    self.draw_specific_table(&mut f, app_state, vertical_chunks[4], false, app_state.current_widget_selected);
                } else {
                    self.draw_specific_table(&mut f, app_state, vertical_chunks[4], false, app_state.previous_basic_table_selected);
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

    fn draw_process_and_search<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
    ) {
        let search_width = if draw_border { 5 } else { 3 };

        if app_state.is_searching() {
            let processes_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(search_width)].as_ref())
                .split(draw_loc);

            self.draw_processes_table(f, app_state, processes_chunk[0], draw_border);
            self.draw_search_field(f, app_state, processes_chunk[1], draw_border);
        } else {
            self.draw_processes_table(f, app_state, draw_loc, draw_border);
        }
    }

    fn draw_cpu_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &app::App, draw_loc: Rect,
    ) {
        let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

        // CPU usage graph
        let x_axis: Axis<'_, String> = Axis::default().bounds([0.0, TIME_STARTS_FROM as f64]);
        let y_axis = Axis::default()
            .style(self.colours.graph_style)
            .labels_style(self.colours.graph_style)
            .bounds([-0.5, 100.5])
            .labels(&["0%", "100%"]);

        let dataset_vector: Vec<Dataset<'_>> = cpu_data
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(itx, cpu)| {
                if app_state.cpu_state.core_show_vec[itx] {
                    Some(
                        Dataset::default()
                            .marker(if app_state.app_config_fields.use_dot {
                                Marker::Dot
                            } else {
                                Marker::Braille
                            })
                            .style(
                                if app_state.app_config_fields.show_average_cpu && itx == 0 {
                                    self.colours.avg_colour_style
                                } else {
                                    self.colours.cpu_colour_styles
                                        [itx % self.colours.cpu_colour_styles.len()]
                                },
                            )
                            .data(&cpu.cpu_data[..]),
                    )
                } else {
                    None
                }
            })
            .collect();

        let title = if app_state.is_expanded && !app_state.cpu_state.is_showing_tray {
            const TITLE_BASE: &str = " CPU ── Esc to go back ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title =
                format!(" CPU ─{}─ Esc to go back ", "─".repeat(repeat_num as usize));

            result_title
        } else {
            " CPU ".to_string()
        };

        Chart::default()
            .block(
                Block::default()
                    .title(&title)
                    .title_style(if app_state.is_expanded {
                        self.colours.highlighted_border_style
                    } else {
                        self.colours.widget_title_style
                    })
                    .borders(Borders::ALL)
                    .border_style(match app_state.current_widget_selected {
                        WidgetPosition::Cpu | WidgetPosition::BasicCpu => {
                            self.colours.highlighted_border_style
                        }
                        _ => self.colours.border_style,
                    }),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
            .datasets(&dataset_vector)
            .render(f, draw_loc);
    }

    fn draw_cpu_legend<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect,
    ) {
        let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

        let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
        let start_position = get_start_position(
            num_rows,
            &app_state.app_scroll_positions.scroll_direction,
            &mut app_state
                .app_scroll_positions
                .cpu_scroll_state
                .previous_scroll_position,
            app_state
                .app_scroll_positions
                .cpu_scroll_state
                .current_scroll_position,
            app_state.is_resized,
        );

        let sliced_cpu_data = &cpu_data[start_position as usize..];
        let mut stringified_cpu_data: Vec<Vec<String>> = Vec::new();

        for (itx, cpu) in sliced_cpu_data.iter().enumerate() {
            if app_state.cpu_state.is_showing_tray {
                stringified_cpu_data.push(vec![
                    cpu.cpu_name.clone(),
                    if app_state.cpu_state.core_show_vec[itx + start_position as usize] {
                        "[*]".to_string()
                    } else {
                        "[ ]".to_string()
                    },
                ]);
            } else if let Some(cpu_data) = cpu.cpu_data.last() {
                if app_state.app_config_fields.show_disabled_data
                    || app_state.cpu_state.core_show_vec[itx]
                {
                    stringified_cpu_data.push(vec![
                        cpu.cpu_name.clone(),
                        format!("{:.0}%", cpu_data.1.round()),
                    ]);
                }
            }
        }

        let cpu_rows = stringified_cpu_data
            .iter()
            .enumerate()
            .map(|(itx, cpu_string_row)| {
                Row::StyledData(
                    cpu_string_row.iter(),
                    match app_state.current_widget_selected {
                        WidgetPosition::Cpu => {
                            if itx as u64
                                == app_state
                                    .app_scroll_positions
                                    .cpu_scroll_state
                                    .current_scroll_position
                                    - start_position
                            {
                                self.colours.currently_selected_text_style
                            } else if app_state.app_config_fields.show_average_cpu && itx == 0 {
                                self.colours.avg_colour_style
                            } else {
                                self.colours.cpu_colour_styles[itx
                                    + start_position as usize
                                        % self.colours.cpu_colour_styles.len()]
                            }
                        }
                        _ => {
                            if app_state.app_config_fields.show_average_cpu && itx == 0 {
                                self.colours.avg_colour_style
                            } else {
                                self.colours.cpu_colour_styles[itx
                                    + start_position as usize
                                        % self.colours.cpu_colour_styles.len()]
                            }
                        }
                    },
                )
            });

        // Calculate widths
        let width = f64::from(draw_loc.width);
        let width_ratios = vec![0.5, 0.5];

        let variable_intrinsic_results = get_variable_intrinsic_widths(
            width as u16,
            &width_ratios,
            if app_state.cpu_state.is_showing_tray {
                &CPU_SELECT_LEGEND_HEADER_LENS
            } else {
                &CPU_LEGEND_HEADER_LENS
            },
        );
        let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

        let title = if app_state.cpu_state.is_showing_tray {
            const TITLE_BASE: &str = " Esc to close ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title = format!("{} Esc to close ", "─".repeat(repeat_num as usize));

            result_title
        } else {
            "".to_string()
        };

        // Draw
        Table::new(
            if app_state.cpu_state.is_showing_tray {
                CPU_SELECT_LEGEND_HEADER
            } else {
                CPU_LEGEND_HEADER
            }
            .iter(),
            cpu_rows,
        )
        .block(
            Block::default()
                .title(&title)
                .title_style(if app_state.is_expanded {
                    self.colours.highlighted_border_style
                } else {
                    match app_state.current_widget_selected {
                        WidgetPosition::Cpu => self.colours.highlighted_border_style,
                        _ => self.colours.border_style,
                    }
                })
                .borders(Borders::ALL)
                .border_style(match app_state.current_widget_selected {
                    WidgetPosition::Cpu => self.colours.highlighted_border_style,
                    _ => self.colours.border_style,
                }),
        )
        .header_style(self.colours.table_header_style)
        .widths(
            &(intrinsic_widths
                .iter()
                .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                .collect::<Vec<_>>()),
        )
        .render(f, draw_loc);
    }

    fn draw_memory_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &app::App, draw_loc: Rect,
    ) {
        let mem_data: &[(f64, f64)] = &app_state.canvas_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.canvas_data.swap_data;

        let x_axis: Axis<'_, String> = Axis::default().bounds([0.0, TIME_STARTS_FROM as f64]);

        // Offset as the zero value isn't drawn otherwise...
        let y_axis: Axis<'_, &str> = Axis::default()
            .style(self.colours.graph_style)
            .labels_style(self.colours.graph_style)
            .bounds([-0.5, 100.5])
            .labels(&["0%", "100%"]);

        let mem_canvas_vec: Vec<Dataset<'_>> = vec![
            Dataset::default()
                .name(&app_state.canvas_data.mem_label)
                .marker(if app_state.app_config_fields.use_dot {
                    Marker::Dot
                } else {
                    Marker::Braille
                })
                .style(self.colours.ram_style)
                .data(&mem_data),
            Dataset::default()
                .name(&app_state.canvas_data.swap_label)
                .marker(if app_state.app_config_fields.use_dot {
                    Marker::Dot
                } else {
                    Marker::Braille
                })
                .style(self.colours.swap_style)
                .data(&swap_data),
        ];

        let title = if app_state.is_expanded {
            const TITLE_BASE: &str = " Memory ── Esc to go back ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title = format!(
                " Memory ─{}─ Esc to go back ",
                "─".repeat(repeat_num as usize)
            );

            result_title
        } else {
            " Memory ".to_string()
        };

        Chart::default()
            .block(
                Block::default()
                    .title(&title)
                    .title_style(if app_state.is_expanded {
                        self.colours.highlighted_border_style
                    } else {
                        self.colours.widget_title_style
                    })
                    .borders(Borders::ALL)
                    .border_style(match app_state.current_widget_selected {
                        WidgetPosition::Mem | WidgetPosition::BasicMem => {
                            self.colours.highlighted_border_style
                        }
                        _ => self.colours.border_style,
                    }),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
            .datasets(&mem_canvas_vec)
            .render(f, draw_loc);
    }

    fn draw_network_graph<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &app::App, draw_loc: Rect,
    ) {
        let network_data_rx: &[(f64, f64)] = &app_state.canvas_data.network_data_rx;
        let network_data_tx: &[(f64, f64)] = &app_state.canvas_data.network_data_tx;

        let x_axis: Axis<'_, String> = Axis::default().bounds([0.0, 60_000.0]);
        let y_axis: Axis<'_, &str> = Axis::default()
            .style(self.colours.graph_style)
            .labels_style(self.colours.graph_style)
            .bounds([-0.5, 30_f64])
            .labels(&["0B", "1KiB", "1MiB", "1GiB"]);

        let title = if app_state.is_expanded {
            const TITLE_BASE: &str = " Network ── Esc to go back ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title = format!(
                " Network ─{}─ Esc to go back ",
                "─".repeat(repeat_num as usize)
            );

            result_title
        } else {
            " Network ".to_string()
        };

        Chart::default()
            .block(
                Block::default()
                    .title(&title)
                    .title_style(if app_state.is_expanded {
                        self.colours.highlighted_border_style
                    } else {
                        self.colours.widget_title_style
                    })
                    .borders(Borders::ALL)
                    .border_style(match app_state.current_widget_selected {
                        WidgetPosition::Network | WidgetPosition::BasicNet => {
                            self.colours.highlighted_border_style
                        }
                        _ => self.colours.border_style,
                    }),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
            .datasets(&[
                Dataset::default()
                    .name(&format!("RX: {:7}", app_state.canvas_data.rx_display))
                    .marker(if app_state.app_config_fields.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(self.colours.rx_style)
                    .data(&network_data_rx),
                Dataset::default()
                    .name(&format!("TX: {:7}", app_state.canvas_data.tx_display))
                    .marker(if app_state.app_config_fields.use_dot {
                        Marker::Dot
                    } else {
                        Marker::Braille
                    })
                    .style(self.colours.tx_style)
                    .data(&network_data_tx),
                Dataset::default()
                    .name(&format!(
                        "Total RX: {:7}",
                        app_state.canvas_data.total_rx_display
                    ))
                    .style(self.colours.total_rx_style),
                Dataset::default()
                    .name(&format!(
                        "Total TX: {:7}",
                        app_state.canvas_data.total_tx_display
                    ))
                    .style(self.colours.total_tx_style),
            ])
            .render(f, draw_loc);
    }

    fn draw_network_labels<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect,
    ) {
        let rx_display = &app_state.canvas_data.rx_display;
        let tx_display = &app_state.canvas_data.tx_display;
        let total_rx_display = &app_state.canvas_data.total_rx_display;
        let total_tx_display = &app_state.canvas_data.total_tx_display;

        // Gross but I need it to work...
        let total_network = vec![vec![
            rx_display,
            tx_display,
            total_rx_display,
            total_tx_display,
        ]];
        let mapped_network = total_network
            .iter()
            .map(|val| Row::StyledData(val.iter(), self.colours.text_style));

        // Calculate widths
        let width_ratios: Vec<f64> = vec![0.25, 0.25, 0.25, 0.25];
        let lens: &[usize] = &NETWORK_HEADERS_LENS;
        let width = f64::from(draw_loc.width);

        let variable_intrinsic_results =
            get_variable_intrinsic_widths(width as u16, &width_ratios, lens);
        let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

        // Draw
        Table::new(NETWORK_HEADERS.iter(), mapped_network)
            .block(Block::default().borders(Borders::ALL).border_style(
                match app_state.current_widget_selected {
                    WidgetPosition::Network => self.colours.highlighted_border_style,
                    _ => self.colours.border_style,
                },
            ))
            .header_style(self.colours.table_header_style)
            .style(self.colours.text_style)
            .widths(
                &(intrinsic_widths
                    .iter()
                    .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                    .collect::<Vec<_>>()),
            )
            .render(f, draw_loc);
    }

    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
    ) {
        let temp_sensor_data: &[Vec<String>] = &app_state.canvas_data.temp_sensor_data;

        let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
        let start_position = get_start_position(
            num_rows,
            &app_state.app_scroll_positions.scroll_direction,
            &mut app_state
                .app_scroll_positions
                .temp_scroll_state
                .previous_scroll_position,
            app_state
                .app_scroll_positions
                .temp_scroll_state
                .current_scroll_position,
            app_state.is_resized,
        );

        let sliced_vec = &temp_sensor_data[start_position as usize..];
        let mut temp_row_counter: i64 = 0;

        let temperature_rows = sliced_vec.iter().map(|temp_row| {
            Row::StyledData(
                temp_row.iter(),
                match app_state.current_widget_selected {
                    WidgetPosition::Temp => {
                        if temp_row_counter as u64
                            == app_state
                                .app_scroll_positions
                                .temp_scroll_state
                                .current_scroll_position
                                - start_position
                        {
                            temp_row_counter = -1;
                            self.colours.currently_selected_text_style
                        } else {
                            if temp_row_counter >= 0 {
                                temp_row_counter += 1;
                            }
                            self.colours.text_style
                        }
                    }
                    _ => self.colours.text_style,
                },
            )
        });

        // Calculate widths
        let width = f64::from(draw_loc.width);
        let width_ratios = [0.5, 0.5];
        let variable_intrinsic_results =
            get_variable_intrinsic_widths(width as u16, &width_ratios, &TEMP_HEADERS_LENS);
        let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

        let title = if app_state.is_expanded {
            const TITLE_BASE: &str = " Temperatures ── Esc to go back ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title = format!(
                " Temperatures ─{}─ Esc to go back ",
                "─".repeat(repeat_num as usize)
            );

            result_title
        } else if app_state.app_config_fields.use_basic_mode {
            String::new()
        } else {
            " Temperatures ".to_string()
        };

        let temp_block = if draw_border {
            Block::default()
                .title(&title)
                .title_style(if app_state.is_expanded {
                    match app_state.current_widget_selected {
                        WidgetPosition::Temp => self.colours.highlighted_border_style,
                        _ => self.colours.border_style,
                    }
                } else {
                    self.colours.widget_title_style
                })
                .borders(Borders::ALL)
                .border_style(match app_state.current_widget_selected {
                    WidgetPosition::Temp => self.colours.highlighted_border_style,
                    _ => self.colours.border_style,
                })
        } else {
            match app_state.current_widget_selected {
                WidgetPosition::Temp => Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                _ => Block::default().borders(Borders::NONE),
            }
        };

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(match app_state.current_widget_selected {
                WidgetPosition::Temp => 0,
                _ if !draw_border => 1,
                _ => 0,
            })
            .direction(Direction::Horizontal)
            .split(draw_loc);

        // Draw
        Table::new(TEMP_HEADERS.iter(), temperature_rows)
            .block(temp_block)
            .header_style(self.colours.table_header_style)
            .widths(
                &(intrinsic_widths
                    .iter()
                    .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                    .collect::<Vec<_>>()),
            )
            .render(f, margined_draw_loc[0]);
    }

    fn draw_disk_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
    ) {
        let disk_data: &[Vec<String>] = &app_state.canvas_data.disk_data;
        let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
        let start_position = get_start_position(
            num_rows,
            &app_state.app_scroll_positions.scroll_direction,
            &mut app_state
                .app_scroll_positions
                .disk_scroll_state
                .previous_scroll_position,
            app_state
                .app_scroll_positions
                .disk_scroll_state
                .current_scroll_position,
            app_state.is_resized,
        );

        let sliced_vec = &disk_data[start_position as usize..];
        let mut disk_counter: i64 = 0;

        let disk_rows = sliced_vec.iter().map(|disk| {
            Row::StyledData(
                disk.iter(),
                match app_state.current_widget_selected {
                    WidgetPosition::Disk => {
                        if disk_counter as u64
                            == app_state
                                .app_scroll_positions
                                .disk_scroll_state
                                .current_scroll_position
                                - start_position
                        {
                            disk_counter = -1;
                            self.colours.currently_selected_text_style
                        } else {
                            if disk_counter >= 0 {
                                disk_counter += 1;
                            }
                            self.colours.text_style
                        }
                    }
                    _ => self.colours.text_style,
                },
            )
        });

        // Calculate widths
        // TODO: [PRETTY] Ellipsis on strings?
        let width = f64::from(draw_loc.width);
        let width_ratios = [0.2, 0.15, 0.13, 0.13, 0.13, 0.13, 0.13];
        let variable_intrinsic_results =
            get_variable_intrinsic_widths(width as u16, &width_ratios, &DISK_HEADERS_LENS);
        let intrinsic_widths = &variable_intrinsic_results.0[0..variable_intrinsic_results.1];

        let title = if app_state.is_expanded {
            const TITLE_BASE: &str = " Disk ── Esc to go back ";
            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            let result_title = format!(
                " Disk ─{}─ Esc to go back ",
                "─".repeat(repeat_num as usize)
            );
            result_title
        } else if app_state.app_config_fields.use_basic_mode {
            String::new()
        } else {
            " Disk ".to_string()
        };

        let disk_block = if draw_border {
            Block::default()
                .title(&title)
                .title_style(if app_state.is_expanded {
                    match app_state.current_widget_selected {
                        WidgetPosition::Disk => self.colours.highlighted_border_style,
                        _ => self.colours.border_style,
                    }
                } else {
                    self.colours.widget_title_style
                })
                .borders(Borders::ALL)
                .border_style(match app_state.current_widget_selected {
                    WidgetPosition::Disk => self.colours.highlighted_border_style,
                    _ => self.colours.border_style,
                })
        } else {
            match app_state.current_widget_selected {
                WidgetPosition::Disk => Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                _ => Block::default().borders(Borders::NONE),
            }
        };

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(match app_state.current_widget_selected {
                WidgetPosition::Disk => 0,
                _ if !draw_border => 1,
                _ => 0,
            })
            .direction(Direction::Horizontal)
            .split(draw_loc);

        // Draw!
        Table::new(DISK_HEADERS.iter(), disk_rows)
            .block(disk_block)
            .header_style(self.colours.table_header_style)
            .widths(
                &(intrinsic_widths
                    .iter()
                    .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                    .collect::<Vec<_>>()),
            )
            .render(f, margined_draw_loc[0]);
    }

    fn draw_search_field<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
    ) {
        let width = max(0, draw_loc.width as i64 - 34) as u64; // TODO: [REFACTOR] Hard coding this is terrible.
        let cursor_position = app_state.get_cursor_position();
        let char_cursor_position = app_state.get_char_cursor_position();

        let start_position: usize = get_search_start_position(
            width as usize,
            &app_state.process_search_state.search_state.cursor_direction,
            &mut app_state.process_search_state.search_state.cursor_bar,
            char_cursor_position,
            app_state.is_resized,
        );

        let query = app_state.get_current_search_query().as_str();
        let grapheme_indices = UnicodeSegmentation::grapheme_indices(query, true);
        let mut current_grapheme_posn = 0;
        let query_with_cursor: Vec<Text<'_>> =
            if let WidgetPosition::ProcessSearch = app_state.current_widget_selected {
                let mut res = grapheme_indices
                    .filter_map(|grapheme| {
                        current_grapheme_posn += UnicodeWidthStr::width(grapheme.1);

                        if current_grapheme_posn <= start_position {
                            None
                        } else {
                            let styled = if grapheme.0 == cursor_position {
                                Text::styled(grapheme.1, self.colours.currently_selected_text_style)
                            } else {
                                Text::styled(grapheme.1, self.colours.text_style)
                            };
                            Some(styled)
                        }
                    })
                    .collect::<Vec<_>>();

                if cursor_position >= query.len() {
                    res.push(Text::styled(
                        " ",
                        self.colours.currently_selected_text_style,
                    ))
                }

                res
            } else {
                // This is easier - we just need to get a range of graphemes, rather than
                // dealing with possibly inserting a cursor (as none is shown!)
                grapheme_indices
                    .filter_map(|grapheme| {
                        current_grapheme_posn += UnicodeWidthStr::width(grapheme.1);
                        if current_grapheme_posn <= start_position {
                            None
                        } else {
                            let styled = Text::styled(grapheme.1, self.colours.text_style);
                            Some(styled)
                        }
                    })
                    .collect::<Vec<_>>()
            };

        let mut search_text = vec![if app_state.is_grouped() {
            Text::styled("Search by Name: ", self.colours.table_header_style)
        } else if app_state.process_search_state.is_searching_with_pid {
            Text::styled(
                "Search by PID (Tab for Name): ",
                self.colours.table_header_style,
            )
        } else {
            Text::styled(
                "Search by Name (Tab for PID): ",
                self.colours.table_header_style,
            )
        }];

        // Text options shamelessly stolen from VS Code.
        let mut option_text = vec![];
        let case_style = if !app_state.process_search_state.is_ignoring_case {
            self.colours.currently_selected_text_style
        } else {
            self.colours.text_style
        };

        let whole_word_style = if app_state.process_search_state.is_searching_whole_word {
            self.colours.currently_selected_text_style
        } else {
            self.colours.text_style
        };

        let regex_style = if app_state.process_search_state.is_searching_with_regex {
            self.colours.currently_selected_text_style
        } else {
            self.colours.text_style
        };

        let case_text = format!(
            "Match Case ({})[{}]",
            if self.is_mac_os { "F1" } else { "Alt+C" },
            if !app_state.process_search_state.is_ignoring_case {
                "*"
            } else {
                " "
            }
        );

        let whole_text = format!(
            "Match Whole Word ({})[{}]",
            if self.is_mac_os { "F2" } else { "Alt+W" },
            if app_state.process_search_state.is_searching_whole_word {
                "*"
            } else {
                " "
            }
        );

        let regex_text = format!(
            "Use Regex ({})[{}]",
            if self.is_mac_os { "F3" } else { "Alt+R" },
            if app_state.process_search_state.is_searching_with_regex {
                "*"
            } else {
                " "
            }
        );

        let option_row = vec![
            Text::raw("\n\n"),
            Text::styled(&case_text, case_style),
            Text::raw("     "),
            Text::styled(&whole_text, whole_word_style),
            Text::raw("     "),
            Text::styled(&regex_text, regex_style),
        ];
        option_text.extend(option_row);

        search_text.extend(query_with_cursor);
        search_text.extend(option_text);

        let current_border_style: Style = if app_state
            .process_search_state
            .search_state
            .is_invalid_search
        {
            Style::default().fg(Color::Rgb(255, 0, 0))
        } else {
            match app_state.current_widget_selected {
                WidgetPosition::ProcessSearch => self.colours.highlighted_border_style,
                _ => self.colours.border_style,
            }
        };

        let title = if draw_border {
            const TITLE_BASE: &str = " Esc to close ";

            let repeat_num = max(
                0,
                draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
            );
            format!("{} Esc to close ", "─".repeat(repeat_num as usize))
        } else {
            String::new()
        };

        let process_search_block = if draw_border {
            Block::default()
                .title(&title)
                .title_style(current_border_style)
                .borders(Borders::ALL)
                .border_style(current_border_style)
        } else {
            match app_state.current_widget_selected {
                WidgetPosition::ProcessSearch => Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(current_border_style),
                _ => Block::default().borders(Borders::NONE),
            }
        };

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(match app_state.current_widget_selected {
                WidgetPosition::ProcessSearch => 0,
                _ if !draw_border => 1,
                _ => 0,
            })
            .direction(Direction::Horizontal)
            .split(draw_loc);

        Paragraph::new(search_text.iter())
            .block(process_search_block)
            .style(self.colours.text_style)
            .alignment(Alignment::Left)
            .wrap(false)
            .render(f, margined_draw_loc[0]);
    }

    fn draw_processes_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
    ) {
        let process_data: &[ConvertedProcessData] = &app_state.canvas_data.finalized_process_data;

        // Admittedly this is kinda a hack... but we need to:
        // * Scroll
        // * Show/hide elements based on scroll position
        //
        // As such, we use a process_counter to know when we've
        // hit the process we've currently scrolled to.
        // We also need to move the list - we can
        // do so by hiding some elements!
        let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;

        let position = get_start_position(
            num_rows,
            &app_state.app_scroll_positions.scroll_direction,
            &mut app_state
                .app_scroll_positions
                .process_scroll_state
                .previous_scroll_position,
            app_state
                .app_scroll_positions
                .process_scroll_state
                .current_scroll_position,
            app_state.is_resized,
        );

        // Sanity check
        let start_position = if position >= process_data.len() as u64 {
            std::cmp::max(0, process_data.len() as i64 - 1) as u64
        } else {
            position
        };

        let sliced_vec = &process_data[start_position as usize..];
        let mut process_counter: i64 = 0;

        // Draw!
        let process_rows = sliced_vec.iter().map(|process| {
            let stringified_process_vec: Vec<String> = vec![
                if app_state.is_grouped() {
                    process.group_pids.len().to_string()
                } else {
                    process.pid.to_string()
                },
                process.name.clone(),
                format!("{:.1}%", process.cpu_usage),
                format!("{:.1}%", process.mem_usage),
            ];
            Row::StyledData(
                stringified_process_vec.into_iter(),
                match app_state.current_widget_selected {
                    WidgetPosition::Process => {
                        if process_counter as u64
                            == app_state
                                .app_scroll_positions
                                .process_scroll_state
                                .current_scroll_position
                                - start_position
                        {
                            process_counter = -1;
                            self.colours.currently_selected_text_style
                        } else {
                            if process_counter >= 0 {
                                process_counter += 1;
                            }
                            self.colours.text_style
                        }
                    }
                    _ => self.colours.text_style,
                },
            )
        });

        use app::data_harvester::processes::ProcessSorting;
        let mut pid_or_name = if app_state.is_grouped() {
            "Count"
        } else {
            "PID(p)"
        }
        .to_string();
        let mut name = "Name(n)".to_string();
        let mut cpu = "CPU%(c)".to_string();
        let mut mem = "Mem%(m)".to_string();

        let direction_val = if app_state.process_sorting_reverse {
            "▼".to_string()
        } else {
            "▲".to_string()
        };

        match app_state.process_sorting_type {
            ProcessSorting::CPU => cpu += &direction_val,
            ProcessSorting::MEM => mem += &direction_val,
            ProcessSorting::PID => pid_or_name += &direction_val,
            ProcessSorting::NAME => name += &direction_val,
        };

        let process_headers = [pid_or_name, name, cpu, mem];
        let process_headers_lens: Vec<usize> = process_headers
            .iter()
            .map(|entry| entry.len())
            .collect::<Vec<_>>();

        // Calculate widths
        let width = f64::from(draw_loc.width);
        let width_ratios = [0.2, 0.4, 0.2, 0.2];
        let variable_intrinsic_results =
            get_variable_intrinsic_widths(width as u16, &width_ratios, &process_headers_lens);
        let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

        let title = if draw_border {
            if app_state.is_expanded && !app_state.process_search_state.search_state.is_enabled {
                const TITLE_BASE: &str = " Processes ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!(
                    " Processes ─{}─ Esc to go back ",
                    "─".repeat(repeat_num as usize)
                );

                result_title
            } else {
                " Processes ".to_string()
            }
        } else {
            String::default()
        };

        let process_block = if draw_border {
            Block::default()
                .title(&title)
                .title_style(if app_state.is_expanded {
                    match app_state.current_widget_selected {
                        WidgetPosition::Process => self.colours.highlighted_border_style,
                        _ => self.colours.border_style,
                    }
                } else {
                    self.colours.widget_title_style
                })
                .borders(Borders::ALL)
                .border_style(match app_state.current_widget_selected {
                    WidgetPosition::Process => self.colours.highlighted_border_style,
                    _ => self.colours.border_style,
                })
        } else {
            match app_state.current_widget_selected {
                WidgetPosition::Process => Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style),
                _ => Block::default().borders(Borders::NONE),
            }
        };

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(match app_state.current_widget_selected {
                WidgetPosition::Process => 0,
                _ if !draw_border => 1,
                _ => 0,
            })
            .direction(Direction::Horizontal)
            .split(draw_loc);

        Table::new(process_headers.iter(), process_rows)
            .block(process_block)
            .header_style(self.colours.table_header_style)
            .widths(
                &(intrinsic_widths
                    .iter()
                    .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                    .collect::<Vec<_>>()),
            )
            .render(f, margined_draw_loc[0]);
    }

    fn draw_basic_cpu<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect,
    ) {
        let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

        // This is a bit complicated, but basically, we want to draw SOME number
        // of columns to draw all CPUs.  Ideally, as well, we want to not have
        // to ever scroll.
        // **General logic** - count number of elements in cpu_data.  Then see how
        // many rows and columns we have in draw_loc (-2 on both sides for border?).
        // I think what we can do is try to fit in as many in one column as possible.
        // If not, then add a new column.
        // Then, from this, split the row space across ALL columns.  From there, generate
        // the desired lengths.

        if let WidgetPosition::BasicCpu = app_state.current_widget_selected {
            Block::default()
                .borders(*SIDE_BORDERS)
                .border_style(self.colours.highlighted_border_style)
                .render(f, draw_loc);
        }

        let num_cpus = cpu_data.len();
        if draw_loc.height > 0 {
            let remaining_height = draw_loc.height as usize;
            const REQUIRED_COLUMNS: usize = 4;

            let chunk_vec =
                vec![Constraint::Percentage((100 / REQUIRED_COLUMNS) as u16); REQUIRED_COLUMNS];
            let chunks = Layout::default()
                .constraints(chunk_vec.as_ref())
                .direction(Direction::Horizontal)
                .split(draw_loc);

            // +9 due to 3 + 4 + 2 columns for the name & space + percentage + bar bounds
            let margin_space = 2;
            let remaining_width = max(
                0,
                draw_loc.width as i64 - ((9 + margin_space) * REQUIRED_COLUMNS) as i64,
            ) as usize;

            let bar_length = remaining_width / REQUIRED_COLUMNS;

            // CPU (and RAM) percent bars are, uh, "heavily" inspired from htop.
            let cpu_bars = (0..num_cpus)
                .map(|cpu_index| {
                    let use_percentage =
                        if let Some(cpu_usage) = cpu_data[cpu_index].cpu_data.last() {
                            cpu_usage.1
                        } else {
                            0.0
                        };

                    let num_bars = calculate_basic_use_bars(use_percentage, bar_length);
                    format!(
                        "{:3}[{}{}{:3.0}%]\n",
                        if app_state.app_config_fields.show_average_cpu {
                            if cpu_index == 0 {
                                "AVG".to_string()
                            } else {
                                (cpu_index - 1).to_string()
                            }
                        } else {
                            cpu_index.to_string()
                        },
                        "|".repeat(num_bars),
                        " ".repeat(bar_length - num_bars),
                        use_percentage.round(),
                    )
                })
                .collect::<Vec<_>>();

            let mut row_counter = num_cpus;
            let mut start_index = 0;
            for (itx, chunk) in chunks.iter().enumerate() {
                let to_divide = REQUIRED_COLUMNS - itx;
                let how_many_cpus = min(
                    remaining_height,
                    (row_counter / to_divide) + (if row_counter % to_divide == 0 { 0 } else { 1 }),
                );
                row_counter -= how_many_cpus;
                let end_index = min(start_index + how_many_cpus, num_cpus);
                let cpu_column: Vec<Text<'_>> = (start_index..end_index)
                    .map(|cpu_index| {
                        Text::Styled(
                            (&cpu_bars[cpu_index]).into(),
                            self.colours.cpu_colour_styles
                                [cpu_index as usize % self.colours.cpu_colour_styles.len()],
                        )
                    })
                    .collect::<Vec<_>>();

                start_index += how_many_cpus;

                let margined_loc = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .horizontal_margin(1)
                    .split(*chunk);

                Paragraph::new(cpu_column.iter())
                    .block(Block::default())
                    .render(f, margined_loc[0]);
            }
        }
    }

    fn draw_basic_memory<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect,
    ) {
        let mem_data: &[(f64, f64)] = &app_state.canvas_data.mem_data;
        let swap_data: &[(f64, f64)] = &app_state.canvas_data.swap_data;

        let margined_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(draw_loc);

        if let WidgetPosition::BasicMem = app_state.current_widget_selected {
            Block::default()
                .borders(*SIDE_BORDERS)
                .border_style(self.colours.highlighted_border_style)
                .render(f, draw_loc);
        }

        // +9 due to 3 + 4 + 2 + 2 columns for the name & space + percentage + bar bounds + margin spacing
        let bar_length = max(0, draw_loc.width as i64 - 11) as usize;
        let ram_use_percentage = if let Some(mem) = mem_data.last() {
            mem.1
        } else {
            0.0
        };
        let swap_use_percentage = if let Some(swap) = swap_data.last() {
            swap.1
        } else {
            0.0
        };
        let num_bars_ram = calculate_basic_use_bars(ram_use_percentage, bar_length);
        let num_bars_swap = calculate_basic_use_bars(swap_use_percentage, bar_length);
        let mem_label = format!(
            "RAM[{}{}{:3.0}%]\n",
            "|".repeat(num_bars_ram),
            " ".repeat(bar_length - num_bars_ram),
            ram_use_percentage.round(),
        );
        let swap_label = format!(
            "SWP[{}{}{:3.0}%]",
            "|".repeat(num_bars_swap),
            " ".repeat(bar_length - num_bars_swap),
            swap_use_percentage.round(),
        );

        let mem_text: Vec<Text<'_>> = vec![
            Text::Styled(mem_label.into(), self.colours.ram_style),
            Text::Styled(swap_label.into(), self.colours.swap_style),
        ];

        Paragraph::new(mem_text.iter())
            .block(Block::default())
            .render(f, margined_loc[0]);
    }

    fn draw_basic_network<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect,
    ) {
        let divided_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(draw_loc);

        let net_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(divided_loc[0]);

        let total_loc = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(divided_loc[1]);

        if let WidgetPosition::BasicNet = app_state.current_widget_selected {
            Block::default()
                .borders(*SIDE_BORDERS)
                .border_style(self.colours.highlighted_border_style)
                .render(f, draw_loc);
        }

        let rx_label = format!("RX: {}\n", &app_state.canvas_data.rx_display);
        let tx_label = format!("TX: {}", &app_state.canvas_data.tx_display);
        let total_rx_label = format!("Total RX: {}\n", &app_state.canvas_data.total_rx_display);
        let total_tx_label = format!("Total TX: {}", &app_state.canvas_data.total_tx_display);

        let net_text = vec![
            Text::Styled(rx_label.into(), self.colours.rx_style),
            Text::Styled(tx_label.into(), self.colours.tx_style),
        ];

        let total_net_text = vec![
            Text::Styled(total_rx_label.into(), self.colours.total_rx_style),
            Text::Styled(total_tx_label.into(), self.colours.total_tx_style),
        ];

        Paragraph::new(net_text.iter())
            .block(Block::default())
            .render(f, net_loc[0]);

        Paragraph::new(total_net_text.iter())
            .block(Block::default())
            .render(f, total_loc[0]);
    }

    fn draw_basic_table_arrows<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect,
    ) {
        // Effectively a paragraph with a ton of spacing

        // TODO: [MODULARITY] This is hard coded.  Gross.
        let (left_table, right_table) = if app_state.current_widget_selected.is_widget_table() {
            match app_state.current_widget_selected {
                WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                    (WidgetPosition::Temp, WidgetPosition::Disk)
                }
                WidgetPosition::Disk => (WidgetPosition::Process, WidgetPosition::Temp),
                WidgetPosition::Temp => (WidgetPosition::Disk, WidgetPosition::Process),
                _ => (WidgetPosition::Disk, WidgetPosition::Temp),
            }
        } else {
            match app_state.previous_basic_table_selected {
                WidgetPosition::Process | WidgetPosition::ProcessSearch => {
                    (WidgetPosition::Temp, WidgetPosition::Disk)
                }
                WidgetPosition::Disk => (WidgetPosition::Process, WidgetPosition::Temp),
                WidgetPosition::Temp => (WidgetPosition::Disk, WidgetPosition::Process),
                _ => (WidgetPosition::Disk, WidgetPosition::Temp),
            }
        };

        let left_name = left_table.get_pretty_name();
        let right_name = right_table.get_pretty_name();

        let num_spaces = max(
            0,
            draw_loc.width as i64 - 2 - 4 - (left_name.len() + right_name.len()) as i64,
        ) as usize;

        let arrow_text = vec![
            Text::Styled(
                format!("\n◄ {}", right_name).into(),
                self.colours.text_style,
            ),
            Text::Raw(" ".repeat(num_spaces).into()),
            Text::Styled(format!("{} ►", left_name).into(), self.colours.text_style),
        ];

        let margined_draw_loc = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .horizontal_margin(1)
            .split(draw_loc);

        Paragraph::new(arrow_text.iter())
            .block(Block::default())
            .render(f, margined_draw_loc[0]);
    }
}
