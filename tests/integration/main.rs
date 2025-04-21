//! Integration tests for bottom.

mod util;

mod arg_tests;
mod invalid_config_tests;
mod layout_movement_tests;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod valid_config_tests;
