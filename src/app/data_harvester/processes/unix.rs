//! Unix-specific parts of process collection.

mod user_table;
pub use user_table::*;

cfg_if::cfg_if! {
    if #[cfg(all(target_family = "unix", not(target_os = "linux")))] {
        mod process_data;
        pub use process_data::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_family = "unix", all(not(target_os = "linux"), not(target_os = "macos"), not(target_os = "freebsd"))))] {
        use sysinfo::{System};
        use super::ProcessHarvest;

        pub fn get_process_data(
            sys: &System, use_current_cpu_total: bool, unnormalized_cpu: bool, mem_total: u64,
            user_table: &mut UserTable,
        ) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
            process_data_wrapper(
                sys,
                use_current_cpu_total,
                unnormalized_cpu,
                mem_total,
                user_table,
            )
        }
    }
}
