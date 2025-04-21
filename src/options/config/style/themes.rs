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
    ($value:literal) => {
        tui::style::Style::new()
            .fg(crate::options::config::style::utils::convert_hex_to_color($value.into()).unwrap())
    };
}

pub(super) use color;
pub(super) use hex;
