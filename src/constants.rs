// TODO: Store like three minutes of data, then change how much is shown based on scaling!
pub const STALE_MAX_MILLISECONDS: u64 = 180 * 1000; // We wish to store at most 60 seconds worth of data.  This may change in the future, or be configurable.
pub const TIME_STARTS_FROM: u64 = 60 * 1000; // TODO: Fix this
pub const TICK_RATE_IN_MILLISECONDS: u64 = 200; // We use this as it's a good value to work with.
pub const DEFAULT_REFRESH_RATE_IN_MILLISECONDS: u128 = 1000;
pub const MAX_KEY_TIMEOUT_IN_MILLISECONDS: u128 = 1000;
pub const NUM_COLOURS: i32 = 256;
