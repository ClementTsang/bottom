use nvml_wrapper::Nvml;

use crate::new_data_collection::sources::memory::MemData;

/// Returns GPU memory usage per device name.
pub(crate) fn get_gpu_memory_usage(nvml: &Nvml) -> Vec<(String, MemData)> {
    let Ok(num_gpu) = nvml.device_count() else {
        return vec![];
    };

    (0..num_gpu)
        .filter_map(|i| nvml.device_by_index(i).ok())
        .filter_map(|device| match device.name() {
            Ok(name) => {
                match device.memory_info() {
                    Ok(mem_info) => Some((
                        name,
                        MemData {
                            used_bytes: mem_info.used,
                            total_bytes: mem_info.total,
                        },
                    )),
                    Err(_) => {
                        // TODO: Maybe we should still return something here if it errors out.
                        None
                    }
                }
            }
            Err(_) => None,
        })
        .collect()
}
