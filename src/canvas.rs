use std::{collections::HashMap, str::FromStr};

use fxhash::FxHashMap;
use indextree::{Arena, NodeId};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    widgets::Paragraph,
    Frame, Terminal,
};

use canvas_colours::*;
use dialogs::*;

use crate::{
    app::{
        self,
        layout_manager::{generate_layout, ColLayout, LayoutNode, RowLayout},
        text_table::TextTableData,
        widgets::{Component, Widget},
        DialogState, TmpBottomWidget,
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
    pub disk_data: TextTableData,
    pub temp_sensor_data: TextTableData,
    pub single_process_data: HashMap<Pid, ConvertedProcessData>, // Contains single process data, key is PID
    pub stringified_process_data_map: HashMap<NodeId, Vec<(Vec<(String, Option<String>)>, bool)>>, // Represents the row and whether it is disabled, key is the widget ID

    pub mem_labels: Option<(String, String)>,
    pub swap_labels: Option<(String, String)>,
    pub mem_data: Vec<Point>,
    pub swap_data: Vec<Point>,

    pub load_avg_data: [f32; 3],
    pub cpu_data: Vec<ConvertedCpuData>,
    pub battery_data: Vec<ConvertedBatteryData>,
}

#[derive(Debug)]
pub enum ColourScheme {
    Default,
    DefaultLight,
    Gruvbox,
    GruvboxLight,
    Nord,
    NordLight,
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
            "nord" => Ok(ColourScheme::Nord),
            "nord-light" => Ok(ColourScheme::NordLight),
            _ => Err(BottomError::ConfigError(format!(
                "\"{}\" is an invalid built-in color scheme.",
                s
            ))),
        }
    }
}

/// Handles the canvas' state.
pub struct Painter {
    pub colours: CanvasColours,
}

impl Painter {
    pub fn init(config: &Config, colour_scheme: ColourScheme) -> anyhow::Result<Self> {
        let mut painter = Painter {
            colours: CanvasColours::default(),
        };

        if let ColourScheme::Custom = colour_scheme {
            painter.generate_config_colours(config)?;
        } else {
            painter.generate_colour_scheme(colour_scheme)?;
        }

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
            ColourScheme::Nord => {
                self.colours
                    .set_colours_from_palette(&*NORD_COLOUR_PALETTE)?;
            }
            ColourScheme::NordLight => {
                self.colours
                    .set_colours_from_palette(&*NORD_LIGHT_COLOUR_PALETTE)?;
            }
            ColourScheme::Custom => {
                // This case should never occur, just do nothing.
            }
        }

        Ok(())
    }

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
        &mut self, terminal: &mut Terminal<B>, app_state: &mut app::AppState,
    ) -> error::Result<()> {
        terminal.draw(|mut f| {
            let (draw_area, frozen_draw_loc) = if app_state.is_frozen() {
                let split_loc = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(f.size());
                (split_loc[0], Some(split_loc[1]))
            } else {
                (f.size(), None)
            };
            let terminal_height = draw_area.height;
            let terminal_width = draw_area.width;

            if let DialogState::Shown(help_dialog) = &mut app_state.help_dialog {
                let gen_help_len = GENERAL_HELP_TEXT.len() as u16 + 3;
                let border_len = terminal_height.saturating_sub(gen_help_len) / 2;
                let vertical_dialog_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(border_len),
                        Constraint::Length(gen_help_len),
                        Constraint::Length(border_len),
                    ])
                    .split(draw_area);

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

                help_dialog.draw_help(&self, f, middle_dialog_chunk[1]);
            } else if app_state.delete_dialog_state.is_showing_dd {
                // TODO: This needs the paragraph wrap feature from tui-rs to be pushed to complete... but for now it's pretty close!
                // The main problem right now is that I cannot properly calculate the height offset since
                // line-wrapping is NOT the same as taking the width of the text and dividing by width.
                // So, I need the height AFTER wrapping.
                // See: https://github.com/fdehau/tui-rs/pull/349.  Land this after this pushes to release.

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
                    .split(draw_area);

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

                if let Some(current_widget) = app_state
                    .widget_lookup_map
                    .get_mut(&app_state.selected_widget)
                {
                    current_widget.set_bounds(draw_area);
                    current_widget.draw(self, f, draw_area, true, true);
                }
            } else {
                /// A simple traversal through the `arena`, drawing all leaf elements.
                fn traverse_and_draw_tree<B: Backend>(
                    node: NodeId, arena: &Arena<LayoutNode>, f: &mut Frame<'_, B>,
                    lookup_map: &mut FxHashMap<NodeId, TmpBottomWidget>, painter: &Painter,
                    canvas_data: &DisplayableData, selected_id: NodeId, offset_x: u16,
                    offset_y: u16,
                ) {
                    if let Some(layout_node) = arena.get(node).map(|n| n.get()) {
                        match layout_node {
                            LayoutNode::Row(RowLayout { bound, .. })
                            | LayoutNode::Col(ColLayout { bound, .. }) => {
                                for child in node.children(arena) {
                                    traverse_and_draw_tree(
                                        child,
                                        arena,
                                        f,
                                        lookup_map,
                                        painter,
                                        canvas_data,
                                        selected_id,
                                        offset_x + bound.x,
                                        offset_y + bound.y,
                                    );
                                }
                            }
                            LayoutNode::Widget(widget_layout) => {
                                let bound = widget_layout.bound;
                                let area = Rect::new(
                                    bound.x + offset_x,
                                    bound.y + offset_y,
                                    bound.width,
                                    bound.height,
                                );

                                if let Some(widget) = lookup_map.get_mut(&node) {
                                    // debug!(
                                    //     "Original bound: {:?}, offset_x: {}, offset_y: {}, area: {:?}, widget: {}",
                                    //     bound,
                                    //     offset_x,
                                    //     offset_y,
                                    //     area,
                                    //     widget.get_pretty_name()
                                    // );

                                    if let TmpBottomWidget::Carousel(carousel) = widget {
                                        let remaining_area: Rect =
                                            carousel.draw_carousel(painter, f, area);
                                        if let Some(to_draw_node) =
                                            carousel.get_currently_selected()
                                        {
                                            if let Some(child_widget) =
                                                lookup_map.get_mut(&to_draw_node)
                                            {
                                                child_widget.set_bounds(remaining_area);
                                                child_widget.draw(
                                                    painter,
                                                    f,
                                                    remaining_area,
                                                    selected_id == to_draw_node,
                                                    false,
                                                );
                                            }
                                        }
                                    } else {
                                        widget.set_bounds(area);
                                        widget.draw(painter, f, area, selected_id == node, false);
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(frozen_draw_loc) = frozen_draw_loc {
                    self.draw_frozen_indicator(&mut f, frozen_draw_loc);
                }

                let root = &app_state.layout_tree_root;
                let arena = &mut app_state.layout_tree;
                let canvas_data = &app_state.canvas_data;
                let selected_id = app_state.selected_widget;

                generate_layout(*root, arena, draw_area, &app_state.widget_lookup_map);

                let lookup_map = &mut app_state.widget_lookup_map;
                traverse_and_draw_tree(
                    *root,
                    arena,
                    f,
                    lookup_map,
                    self,
                    canvas_data,
                    selected_id,
                    0,
                    0,
                );
            }
        })?;

        Ok(())
    }
}
