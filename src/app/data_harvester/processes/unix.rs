//! Unix-specific parts of process collection.

mod user_table;
use cfg_if::cfg_if;
pub use user_table::*;

cfg_if! {
    if #[cfg(all(target_family = "unix", not(target_os = "linux")))] {
        mod process_ext;
        pub(crate) use process_ext::*;

        use sysinfo::System;
        use super::ProcessHarvest;

        use crate::utils::error;

        pub fn get_process_data(
            sys: &System, use_current_cpu_total: bool, unnormalized_cpu: bool, total_memory: u64,
            user_table: &mut UserTable,
        ) -> error::Result<Vec<ProcessHarvest>> {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    crate::app::data_harvester::processes::MacOSProcessExt::get_process_data(sys, use_current_cpu_total, unnormalized_cpu, total_memory, user_table)
                } else if #[cfg(target_os = "freebsd")] {
                    crate::app::data_harvester::processes::FreeBSDProcessExt::get_process_data(sys, use_current_cpu_total, unnormalized_cpu, total_memory, user_table)
                } else {
                    struct GenericProcessExt;
                    impl UnixProcessExt for GenericProcessExt {}

                    GenericProcessExt::get_process_data(sys, use_current_cpu_total, unnormalized_cpu, total_memory, user_table)
                }
            }
        }

    }
}
