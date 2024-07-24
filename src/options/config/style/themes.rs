//! A set of pre-defined themes.

pub(super) mod default;
pub(super) mod gruvbox;
pub(super) mod nord;

macro_rules! color {
    ($value:expr) => {
        tui::style::Style::new().fg($value)
    };
}

macro_rules! hex {
    ($value:literal) => {};
}

pub(super) use {color, hex};
