//! Unix-specific parts of process collection.

mod user_table;

pub use user_table::*;

cfg_select! {
    all(target_family = "unix", not(target_os = "linux")) => {
        mod process_ext;
        pub(crate) use process_ext::*;

        use super::ProcessHarvest;

        use crate::collection::{DataCollector, processes::*};
        use crate::collection::error::CollectionResult;

        pub fn sysinfo_process_data(collector: &mut DataCollector) -> CollectionResult<Vec<ProcessHarvest>> {
            let sys = &collector.sys.system;
            let use_current_cpu_total = collector.use_current_cpu_total;
            let unnormalized_cpu = collector.unnormalized_cpu;
            let total_memory = collector.total_memory();
            let user_table = &mut collector.user_table;

            cfg_select! {
                target_os = "macos" => {
                    MacOSProcessExt::sysinfo_process_data(sys, use_current_cpu_total, unnormalized_cpu, total_memory, user_table)
                }
                target_os = "freebsd" => {
                    FreeBSDProcessExt::sysinfo_process_data(sys, use_current_cpu_total, unnormalized_cpu, total_memory, user_table)
                }
                _ => {
                    GenericProcessExt::sysinfo_process_data(sys, use_current_cpu_total, unnormalized_cpu, total_memory, user_table)
                }
            }
        }

    }
    _ => {}
}
