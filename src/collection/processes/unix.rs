//! Unix-specific parts of process collection.

mod user_table;

pub use user_table::*;

cfg_select! {
    all(target_family = "unix", not(target_os = "linux")) => {
        mod process_ext;
        pub(crate) use process_ext::*;

        use super::ProcessHarvest;
        use crate::collection::{DataCollector, error::CollectionResult, processes::*};

        pub fn sysinfo_process_data(
            collector: &mut DataCollector,
        ) -> CollectionResult<Vec<ProcessHarvest>> {
            cfg_select! {
                target_os = "macos" => MacOSProcessExt::sysinfo_process_data(collector),
                target_os = "freebsd" => FreeBSDProcessExt::sysinfo_process_data(collector),
                _ => GenericProcessExt::sysinfo_process_data(collector),
            }
        }
    }
    _ => {}
}
