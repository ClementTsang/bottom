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
}
