pub const STALE_MAX_MILLISECONDS: u128 = 60 * 1000; // How long to store data.
pub const TIME_STARTS_FROM: u64 = 60 * 1000;
pub const TICK_RATE_IN_MILLISECONDS: u64 = 200; // How fast the screen refreshes
pub const DEFAULT_REFRESH_RATE_IN_MILLISECONDS: u128 = 1000;
pub const MAX_KEY_TIMEOUT_IN_MILLISECONDS: u128 = 1000;
pub const NUM_COLOURS: i32 = 256;

// Config and flags
pub const DEFAULT_CONFIG_FILE_PATH: &str = "~/.config/btm/btm.toml";

pub const KELVIN: &str = "kelvin";
pub const FAHRENHEIT: &str = "fahrenheit";
pub const CELSIUS: &str = "celsius";
