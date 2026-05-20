//! A set of pre-defined themes.

pub(super) mod default;
pub(super) mod gruvbox;
pub(super) mod nord;

/// Convert a [`tui::style::Color`] into a [`tui::style::Style`] with the colour
/// as the foreground.
macro_rules! colour {
    ($value:expr) => {
        tui::style::Style::new().fg($value)
    };
}

/// Convert a hex string to a [`tui::style::Style`], where the hex string is
/// used as the foreground colour.
macro_rules! hex {
    ($value:literal) => {
        tui::style::Style::new().fg(crate::options::config::style::utils::try_hex_to_colour(
            $value.into(),
        )
        .expect("valid hex"))
    };
}

/// Convert a hex string to a [`tui::style::Color`].
macro_rules! hex_colour {
    ($value:literal) => {
        crate::options::config::style::utils::try_hex_to_colour($value.into()).expect("valid hex")
    };
}

pub(super) use colour;
pub(super) use hex;
pub(super) use hex_colour;
