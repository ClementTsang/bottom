use std::str::FromStr;

use indextree::{Arena, NodeId};
use rustc_hash::FxHashMap;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::Paragraph,
    Frame, Terminal,
};

use canvas_colours::*;

use crate::{
    app::{
        self,
        layout_manager::{generate_layout, ColLayout, LayoutNode, RowLayout},
        widgets::{Component, Widget},
        BottomWidget,
    },
    constants::*,
    options::Config,
    utils::error,
    utils::error::BottomError,
};

mod canvas_colours;

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
            let (draw_area, frozen_draw_loc) = if false {
                let split_loc = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(f.size());
                (split_loc[0], Some(split_loc[1]))
            } else {
                (f.size(), None)
            };
            let terminal_height = draw_area.height;
            let terminal_width = draw_area.width;

            if false {
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
                    lookup_map: &mut FxHashMap<NodeId, BottomWidget>, painter: &Painter,
                    selected_id: NodeId, offset_x: u16, offset_y: u16,
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
                                    if let BottomWidget::Carousel(carousel) = widget {
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
                                                    selected_id == node,
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
                let selected_id = app_state.selected_widget;

                generate_layout(*root, arena, draw_area, &app_state.widget_lookup_map);

                let lookup_map = &mut app_state.widget_lookup_map;
                traverse_and_draw_tree(*root, arena, f, lookup_map, self, selected_id, 0, 0);
            }
        })?;

        Ok(())
    }
}
