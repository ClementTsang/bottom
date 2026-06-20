//! A set of pre-defined themes.

pub(super) mod default;
pub(super) mod gruvbox;
pub(super) mod nord;

/// Convert a [`ratatui::style::Color`] into a [`ratatui::style::Style`] with the colour
/// as the foreground.
macro_rules! colour {
    ($value:expr) => {
        ratatui::style::Style::new().fg($value)
    };
}

/// Convert a hex string to a [`ratatui::style::Style`], where the hex string is
/// used as the foreground colour.
macro_rules! hex {
    ($value:literal) => {
        ratatui::style::Style::new().fg(crate::options::config::style::utils::try_hex_to_colour(
            $value.into(),
        )
        .expect("valid hex"))
    };
}

/// Convert a hex string to a [`ratatui::style::Color`].
macro_rules! hex_colour {
    ($value:literal) => {
        crate::options::config::style::utils::try_hex_to_colour($value.into()).expect("valid hex")
    };
}

pub(super) use colour;
pub(super) use hex;
pub(super) use hex_colour;
